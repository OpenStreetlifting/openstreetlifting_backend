use super::models as liftcontrol_models;
use super::movement_mapper::LiftControlMovementMapper;
use super::spec::CompetitionMetadata;
use crate::canonical::models as canonical;
use crate::movement_mapper::MovementMapper;
use crate::{ImporterError, Result};
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

pub struct LiftControlExporter {
    metadata: CompetitionMetadata,
    base_slug: String,
}

impl LiftControlExporter {
    pub fn new(base_slug: String, metadata: CompetitionMetadata) -> Self {
        Self {
            metadata,
            base_slug,
        }
    }

    pub fn to_canonical(
        &self,
        api_response: liftcontrol_models::ApiResponse,
    ) -> Result<canonical::CanonicalFormat> {
        Ok(canonical::CanonicalFormat {
            format_version: "1.0.0".to_string(),
            source: self.build_source_metadata(&api_response),
            competition: self.build_competition_data(),
            movements: self.build_movements(&api_response.results.movements)?,
            categories: self.build_categories(&api_response.results)?,
            liftcontrol_metadata: Some(canonical::LiftControlMetadata {
                contest_id: api_response.contest.id,
            }),
            pdf_metadata: None,
        })
    }

    fn build_source_metadata(
        &self,
        api_response: &liftcontrol_models::ApiResponse,
    ) -> canonical::SourceMetadata {
        canonical::SourceMetadata {
            r#type: canonical::SourceType::LiftControl,
            url: Some(format!(
                "https://app.liftcontrol.com/contest/{}",
                api_response.contest.slug
            )),
            extracted_at: Utc::now(),
            extractor: "liftcontrol-api-v1".to_string(),
            original_filename: None,
        }
    }

    fn build_competition_data(&self) -> canonical::CompetitionData {
        canonical::CompetitionData {
            name: self.metadata.name.clone(),
            slug: self.base_slug.clone(),
            federation: canonical::FederationData {
                name: self.metadata.federation.name.clone(),
                slug: None,
                abbreviation: Some(self.metadata.federation.abbreviation.clone()),
                country: Some(self.metadata.federation.country.clone()),
            },
            start_date: self.metadata.start_date,
            end_date: self.metadata.end_date,
            venue: self.metadata.venue.clone(),
            city: self.metadata.city.clone(),
            country: self
                .metadata
                .country
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            number_of_judges: self.metadata.number_of_judges,
            status: Some("completed".to_string()),
        }
    }

    fn build_movements(
        &self,
        movements: &HashMap<String, liftcontrol_models::Movement>,
    ) -> Result<Vec<canonical::MovementData>> {
        let mapper = LiftControlMovementMapper;
        let mut result = Vec::new();

        for movement in movements.values() {
            let canonical_name = mapper.map_movement(&movement.name).ok_or_else(|| {
                ImporterError::TransformationError(format!("Unknown movement: {}", movement.name))
            })?;

            result.push(canonical::MovementData {
                name: canonical_name.as_str().to_string(),
                order: movement.order as i16,
                is_required: Some(true),
            });
        }

        result.sort_by_key(|m| m.order);
        Ok(result)
    }

    fn build_categories(
        &self,
        results: &liftcontrol_models::ApiResults,
    ) -> Result<Vec<canonical::CategoryData>> {
        let mut categories: HashMap<String, canonical::CategoryData> = HashMap::new();

        for (category_id_str, category_info) in &results.categories {
            let parsed = parse_category_name(&category_info.name);
            let gender = map_gender(&category_info.genre);

            let category_data = categories
                .entry(parsed.weight_class.clone())
                .or_insert_with(|| canonical::CategoryData {
                    name: parsed.weight_class.clone(),
                    gender: gender.clone(),
                    weight_class_min: parsed.weight_class_min,
                    weight_class_max: parsed.weight_class_max,
                    athletes: Vec::new(),
                });

            if let Some(athletes_data) = results.results.get(category_id_str) {
                for athlete_data in athletes_data.values() {
                    category_data
                        .athletes
                        .push(self.build_athlete_data(athlete_data, &results.movements)?);
                }
            }
        }

        Ok(categories.into_values().collect())
    }

    fn build_athlete_data(
        &self,
        athlete_data: &liftcontrol_models::AthleteData,
        movements: &HashMap<String, liftcontrol_models::Movement>,
    ) -> Result<canonical::AthleteData> {
        let bodyweight = athlete_data.athlete_info.pesee.and_then(|w| {
            Decimal::from_str(&w.to_string())
                .ok()
                .filter(|d| *d > Decimal::ZERO)
        });

        let mut lifts = Vec::new();
        let mut movement_list: Vec<_> = movements.values().collect();
        movement_list.sort_by_key(|m| m.order);

        for movement in movement_list {
            if let Some(movement_results) = athlete_data.results.get(&movement.id.to_string()) {
                lifts.push(self.build_lift_data(movement, movement_results)?);
            }
        }

        Ok(canonical::AthleteData {
            first_name: athlete_data.athlete_info.first_name.clone(),
            last_name: athlete_data.athlete_info.last_name.clone(),
            gender: None,
            country: self.metadata.default_athlete_country.clone(),
            nationality: Some(self.metadata.default_athlete_nationality.clone()),
            bodyweight,
            is_disqualified: Some(athlete_data.athlete_info.is_out),
            disqualified_reason: athlete_data.athlete_info.reason_out.clone(),
            lifts,
            liftcontrol_athlete_metadata: Some(canonical::LiftControlAthleteMetadata {
                athlete_id: athlete_data.athlete_info.id,
                reglage_dips: athlete_data.athlete_info.reglage_dips.clone(),
                reglage_squat: athlete_data.athlete_info.reglage_squat.clone(),
            }),
        })
    }

    fn build_lift_data(
        &self,
        movement: &liftcontrol_models::Movement,
        movement_results: &liftcontrol_models::MovementResults,
    ) -> Result<canonical::LiftData> {
        let mapper = LiftControlMovementMapper;
        let canonical_name = mapper.map_movement(&movement.name).ok_or_else(|| {
            ImporterError::TransformationError(format!("Unknown movement: {}", movement.name))
        })?;

        let mut attempts = Vec::new();

        for attempt_num in 1..=3 {
            if let Some(Some(attempt)) = movement_results.results.get(&attempt_num.to_string()) {
                attempts.push(self.build_attempt_data(attempt)?);
            }
        }

        Ok(canonical::LiftData {
            movement: canonical_name.as_str().to_string(),
            attempts,
        })
    }

    fn build_attempt_data(
        &self,
        attempt: &liftcontrol_models::Attempt,
    ) -> Result<canonical::AttemptData> {
        let weight = Decimal::from_str(&attempt.charge.to_string()).unwrap_or_default();

        let is_successful = match &attempt.decision_rep {
            liftcontrol_models::DecisionRep::Number(n) => *n >= 2,
            liftcontrol_models::DecisionRep::String(s) if s == "validé" || s == "valide" => true,
            liftcontrol_models::DecisionRep::String(_) => false,
        };

        Ok(canonical::AttemptData {
            attempt_number: attempt.no_essai as i16,
            weight,
            is_successful,
            no_rep_reason: attempt.justification_no_rep.clone(),
        })
    }
}

struct ParsedCategory {
    weight_class: String,
    weight_class_min: Option<Decimal>,
    weight_class_max: Option<Decimal>,
}

fn parse_category_name(name: &str) -> ParsedCategory {
    let parts: Vec<&str> = name.split(" - ").collect();
    let weight_class = parts.first().unwrap_or(&"").to_string();

    let (weight_class_min, weight_class_max) = parse_weight_class(&weight_class);

    ParsedCategory {
        weight_class,
        weight_class_min,
        weight_class_max,
    }
}

fn parse_weight_class(weight_class: &str) -> (Option<Decimal>, Option<Decimal>) {
    if weight_class.starts_with('-') || weight_class.ends_with("kg") {
        let cleaned = weight_class.trim_start_matches('-').trim_end_matches("kg");
        if let Ok(max) = Decimal::from_str(cleaned) {
            return (None, Some(max));
        }
    }

    if weight_class.contains('+') {
        let cleaned = weight_class.trim_end_matches('+').trim_end_matches("kg");
        if let Ok(min) = Decimal::from_str(cleaned) {
            return (Some(min), None);
        }
    }

    if weight_class.contains('-') && !weight_class.starts_with('-') {
        let parts: Vec<&str> = weight_class.split('-').collect();
        if parts.len() == 2 {
            let min = Decimal::from_str(parts[0].trim()).ok();
            let max = Decimal::from_str(parts[1].trim().trim_end_matches("kg")).ok();
            return (min, max);
        }
    }

    (None, None)
}

fn map_gender(genre: &str) -> String {
    match genre.to_lowercase().as_str() {
        "homme" | "hommes" | "men" | "man" | "male" | "m" => "M".to_string(),
        "femme" | "femmes" | "women" | "woman" | "female" | "f" => "F".to_string(),
        _ => "M".to_string(),
    }
}
