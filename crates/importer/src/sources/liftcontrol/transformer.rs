use super::models::*;
use super::movement_mapper::LiftControlMovementMapper;
use crate::movement_mapper::MovementMapper;
use crate::{ImporterError, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use storage::models::NormalizedAthleteName;
use tracing::info;
use uuid::Uuid;

pub struct LiftControlTransformer<'a> {
    pool: &'a PgPool,
    base_slug: Option<String>,
}

impl<'a> LiftControlTransformer<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            base_slug: None,
        }
    }

    pub fn with_base_slug(pool: &'a PgPool, base_slug: String) -> Self {
        Self {
            pool,
            base_slug: Some(base_slug),
        }
    }

    pub async fn import_competition(&self, api_response: ApiResponse) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let competition_id = self
            .upsert_competition(&api_response.contest, &mut tx)
            .await?;

        self.upsert_competition_movements(competition_id, &api_response.results.movements, &mut tx)
            .await?;

        for (category_id_str, category_info) in &api_response.results.categories {
            info!(
                "category id : {:?}, category info : {:?}",
                category_id_str, category_info
            );
            let category_id = self.upsert_category(category_info, &mut tx).await?;

            let group_id = self
                .upsert_competition_group(
                    competition_id,
                    category_id,
                    &api_response.contest.name,
                    &mut tx,
                )
                .await?;

            if let Some(athletes_data) = api_response.results.results.get(category_id_str) {
                for athlete_data in athletes_data.values() {
                    self.import_athlete_performance(
                        athlete_data,
                        category_info,
                        group_id,
                        &api_response.results.movements,
                        &mut tx,
                    )
                    .await?;
                }
            }
        }

        tx.rollback().await?;
        Ok(())
    }

    async fn upsert_competition(
        &self,
        contest: &Contest,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Uuid> {
        let date = extract_date_from_name(&contest.name);
        let federation_id = self.get_default_federation_id(tx).await?;

        // Use the provided base slug to group all sub-slugs into one competition
        let base_slug = self.base_slug.as_ref().ok_or_else(|| {
            ImporterError::TransformationError(
                "Base slug is required for LiftControl imports".to_string(),
            )
        })?;

        let competition_id = sqlx::query_scalar!(
            r#"
            INSERT INTO competitions (name, slug, status, federation_id, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5, $5)
            ON CONFLICT (slug)
            DO UPDATE SET
                name = EXCLUDED.name,
                status = EXCLUDED.status
            RETURNING competition_id as "competition_id: Uuid"
            "#,
            contest.name,
            base_slug,
            "completed",
            federation_id,
            date
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(competition_id)
    }

    async fn upsert_competition_movements(
        &self,
        competition_id: Uuid,
        movements: &HashMap<String, Movement>,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let mapper = LiftControlMovementMapper;

        for movement in movements.values() {
            let canonical_movement = mapper.map_movement(&movement.name).ok_or_else(|| {
                ImporterError::TransformationError(format!(
                    "Unknown movement '{}' for LiftControl importer",
                    movement.name
                ))
            })?;

            let canonical_name = canonical_movement.as_str();

            // Insert into competition_movements using the movement name directly
            sqlx::query!(
                r#"
                INSERT INTO competition_movements (competition_id, movement_name, is_required, display_order)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (competition_id, movement_name)
                DO UPDATE SET
                    is_required = EXCLUDED.is_required,
                    display_order = EXCLUDED.display_order
                "#,
                competition_id,
                canonical_name,
                true, // All movements in 4Lift competitions are required
                movement.order
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    async fn get_default_federation_id(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Uuid> {
        let existing = sqlx::query_scalar!(
            r#"SELECT federation_id as "federation_id: Uuid" FROM federations WHERE name = '4Lift'"#
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let federation_id = sqlx::query_scalar!(
            r#"
            INSERT INTO federations (name, abbreviation, country)
            VALUES ('4Lift', '4L', 'FR')
            RETURNING federation_id as "federation_id: Uuid"
            "#
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            ImporterError::TransformationError(format!("Failed to get federation: {}", e))
        })?;

        Ok(federation_id)
    }

    async fn upsert_category(
        &self,
        category_info: &CategoryInfo,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Uuid> {
        let gender = map_gender(&category_info.genre);

        let (weight_min, weight_max) = extract_weight_class(&category_info.name);

        let existing = sqlx::query_scalar!(
            r#"SELECT category_id as "category_id: Uuid" FROM categories WHERE name = $1"#,
            category_info.name
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let weight_min_decimal = weight_min
            .as_deref()
            .map(Decimal::from_str)
            .transpose()
            .map_err(|e| {
                ImporterError::TransformationError(format!("Invalid weight_min: {}", e))
            })?;

        let weight_max_decimal = weight_max
            .as_deref()
            .map(Decimal::from_str)
            .transpose()
            .map_err(|e| {
                ImporterError::TransformationError(format!("Invalid weight_max: {}", e))
            })?;

        let category_id = sqlx::query_scalar!(
            r#"
            INSERT INTO categories (name, gender, weight_class_min, weight_class_max)
            VALUES ($1, $2, $3, $4)
            RETURNING category_id as "category_id: Uuid"
            "#,
            category_info.name,
            gender,
            weight_min_decimal,
            weight_max_decimal
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(category_id)
    }

    async fn upsert_competition_group(
        &self,
        competition_id: Uuid,
        category_id: Uuid,
        group_name: &str,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Uuid> {
        let group_id = sqlx::query_scalar!(
            r#"
            INSERT INTO competition_groups (competition_id, category_id, name)
            VALUES ($1, $2, $3)
            ON CONFLICT (competition_id, category_id, name)
            DO UPDATE SET name = EXCLUDED.name
            RETURNING group_id as "group_id: Uuid"
            "#,
            competition_id,
            category_id,
            group_name
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(group_id)
    }

    async fn import_athlete_performance(
        &self,
        athlete_data: &AthleteData,
        category_info: &CategoryInfo,
        group_id: Uuid,
        movements: &HashMap<String, Movement>,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let athlete_id = self
            .upsert_athlete(&athlete_data.athlete_info, category_info, tx)
            .await?;

        let rank = match &athlete_data.rank {
            AthleteRank::Position(p) => Some(*p as i32),
            AthleteRank::Disqualified(_) => None,
        };

        let bodyweight = athlete_data.athlete_info.pesee.and_then(convert_weight);
        let ris_score = convert_weight(athlete_data.ris);

        sqlx::query!(
            r#"
            INSERT INTO competition_participants
                (group_id, athlete_id, bodyweight, rank, is_disqualified, disqualified_reason, ris_score)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (group_id, athlete_id)
            DO UPDATE SET
                bodyweight = EXCLUDED.bodyweight,
                rank = EXCLUDED.rank,
                is_disqualified = EXCLUDED.is_disqualified,
                disqualified_reason = EXCLUDED.disqualified_reason,
                ris_score = EXCLUDED.ris_score
            "#,
            group_id,
            athlete_id,
            bodyweight,
            rank,
            athlete_data.athlete_info.is_out,
            athlete_data.athlete_info.reason_out,
            ris_score
        )
        .execute(&mut **tx)
        .await?;

        let mut movement_list: Vec<_> = movements.values().collect();
        movement_list.sort_by_key(|m| m.order);

        for movement in movement_list {
            if let Some(movement_results) = athlete_data.results.get(&movement.id.to_string()) {
                self.import_lift(
                    group_id,
                    athlete_id,
                    movement,
                    movement_results,
                    &athlete_data.athlete_info,
                    tx,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn upsert_athlete(
        &self,
        athlete_info: &AthleteInfo,
        category_info: &CategoryInfo,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Uuid> {
        let gender = map_gender(&category_info.genre);

        // Create normalized name to ensure consistent ordering and prevent duplicates
        // like "John Smith" and "Smith John" from being treated as different athletes
        let normalized_name =
            NormalizedAthleteName::new(&athlete_info.first_name, &athlete_info.last_name);
        let (db_first_name, db_last_name) = normalized_name.as_database_tuple();

        // Check using normalized names
        let existing = sqlx::query_scalar!(
            r#"
            SELECT athlete_id as "athlete_id: Uuid" FROM athletes
            WHERE first_name = $1 AND last_name = $2 AND gender = $3 AND country = $4
            "#,
            db_first_name,
            db_last_name,
            gender,
            "FR"
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        // Insert using normalized names
        // LiftControl/4lift competitions are French, so all athletes are French nationals
        let athlete_id = sqlx::query_scalar!(
            r#"
            INSERT INTO athletes (first_name, last_name, gender, country, nationality)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING athlete_id as "athlete_id: Uuid"
            "#,
            db_first_name,
            db_last_name,
            gender,
            "FR",
            "French"
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(athlete_id)
    }

    async fn import_lift(
        &self,
        group_id: Uuid,
        athlete_id: Uuid,
        movement: &Movement,
        movement_results: &MovementResults,
        athlete_info: &AthleteInfo,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let mapper = LiftControlMovementMapper;
        let canonical_movement = mapper.map_movement(&movement.name).ok_or_else(|| {
            ImporterError::TransformationError(format!(
                "Unknown movement '{}' for LiftControl importer",
                movement.name
            ))
        })?;

        let movement_name = canonical_movement.as_str();
        let max_weight = convert_weight(movement_results.max);
        let settings = get_movement_settings(&movement.name, athlete_info);

        // Get the participant_id from competition_participants
        let participant = sqlx::query!(
            r#"
            SELECT participant_id
            FROM competition_participants
            WHERE group_id = $1 AND athlete_id = $2
            "#,
            group_id,
            athlete_id
        )
        .fetch_one(&mut **tx)
        .await?;

        // Insert or update the lift
        sqlx::query!(
            r#"
            INSERT INTO lifts (participant_id, movement_name, max_weight, equipment_setting)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (participant_id, movement_name)
            DO UPDATE SET
                max_weight = EXCLUDED.max_weight,
                equipment_setting = EXCLUDED.equipment_setting,
                updated_at = CURRENT_TIMESTAMP
            "#,
            participant.participant_id,
            movement_name,
            max_weight,
            settings
        )
        .execute(&mut **tx)
        .await?;

        for i in 1..=3 {
            if let Some(Some(attempt)) = movement_results.results.get(&i.to_string()) {
                self.import_attempt(participant.participant_id, movement_name, attempt, tx)
                    .await?;
            }
        }

        Ok(())
    }

    async fn import_attempt(
        &self,
        participant_id: Uuid,
        movement_name: &str,
        attempt: &Attempt,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let success = match &attempt.decision_rep {
            DecisionRep::Number(n) => *n == 111 || *n == 110,
            DecisionRep::String(s) => s == "111" || s == "110",
        };

        let passing_judges = match &attempt.decision_rep {
            DecisionRep::Number(n) => count_passing_judges(*n),
            DecisionRep::String(s) => s.parse::<i32>().ok().and_then(count_passing_judges),
        };

        let weight = convert_weight(attempt.charge);

        // Get the lift_id
        let lift = sqlx::query!(
            r#"
            SELECT lift_id
            FROM lifts
            WHERE participant_id = $1 AND movement_name = $2
            "#,
            participant_id,
            movement_name
        )
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO attempts (lift_id, attempt_number, weight, is_successful, passing_judges, no_rep_reason, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (lift_id, attempt_number)
            DO UPDATE SET
                weight = EXCLUDED.weight,
                is_successful = EXCLUDED.is_successful,
                passing_judges = EXCLUDED.passing_judges,
                no_rep_reason = EXCLUDED.no_rep_reason,
                created_by = EXCLUDED.created_by
            "#,
            lift.lift_id,
            attempt.no_essai as i16,
            weight,
            success,
            passing_judges,
            attempt.justification_no_rep,
            "Adrien Pelfresne"
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

fn extract_date_from_name(name: &str) -> NaiveDate {
    for year in 2020..=2030 {
        if name.contains(&year.to_string()) {
            return NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        }
    }
    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
}

fn map_gender(genre: &str) -> String {
    match genre.to_lowercase().as_str() {
        "homme" | "hommes" | "men" | "male" | "m" => "M".to_string(),
        "femme" | "femmes" | "women" | "female" | "f" => "F".to_string(),
        _ => "MX".to_string(),
    }
}

fn extract_weight_class(category_name: &str) -> (Option<String>, Option<String>) {
    if category_name.to_lowercase().contains("open") {
        return (None, None);
    }

    (None, None)
}

fn get_movement_settings(movement_name: &str, athlete_info: &AthleteInfo) -> Option<String> {
    let lower_name = movement_name.to_lowercase();
    if lower_name.contains("dips") {
        athlete_info.reglage_dips.clone()
    } else if lower_name.contains("squat") {
        athlete_info.reglage_squat.clone()
    } else {
        None
    }
}

fn count_passing_judges(decision: i32) -> Option<i16> {
    match decision {
        111 => Some(3),
        110 | 101 | 11 => Some(2),
        100 | 10 | 1 => Some(1),
        0 => Some(0),
        _ => None,
    }
}

/// Converts f64 to Decimal, rounds to 2 decimal places, and treats 0.0 as NULL
fn convert_weight(value: f64) -> Option<Decimal> {
    Decimal::from_f64_retain(value)
        .map(|d| d.round_dp(2))
        .filter(|d| !d.is_zero())
}
