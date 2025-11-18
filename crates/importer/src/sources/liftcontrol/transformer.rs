use super::models::*;
use crate::{ImporterError, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;

pub struct LiftControlTransformer<'a> {
    pool: &'a PgPool,
}

impl<'a> LiftControlTransformer<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn import_competition(&self, api_response: ApiResponse) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let competition_id = self
            .upsert_competition(&api_response.contest, &mut tx)
            .await?;

        for (category_id_str, category_info) in &api_response.results.categories {
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

        tx.commit().await?;
        Ok(())
    }

    async fn upsert_competition(
        &self,
        contest: &Contest,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<i32> {
        let date = extract_date_from_name(&contest.name);
        let federation_id = self.get_default_federation_id(tx).await?;

        let competition_id = sqlx::query_scalar!(
            r#"
            INSERT INTO competitions (name, slug, status, federation_id, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5, $5)
            ON CONFLICT (slug)
            DO UPDATE SET
                name = EXCLUDED.name,
                status = EXCLUDED.status
            RETURNING competition_id
            "#,
            contest.name,
            contest.slug,
            contest.status,
            federation_id,
            date
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(competition_id)
    }

    async fn get_default_federation_id(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<i32> {
        let existing = sqlx::query_scalar!(
            r#"SELECT federation_id FROM federations WHERE name = 'LiftControl'"#
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let federation_id = sqlx::query_scalar!(
            r#"
            INSERT INTO federations (name, abbreviation)
            VALUES ('LiftControl', 'LC')
            RETURNING federation_id
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
    ) -> Result<i32> {
        let gender = map_gender(&category_info.genre);

        let (weight_min, weight_max) = extract_weight_class(&category_info.name);

        let existing = sqlx::query_scalar!(
            r#"SELECT category_id FROM categories WHERE name = $1"#,
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
            RETURNING category_id
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
        competition_id: i32,
        category_id: i32,
        group_name: &str,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<i32> {
        let group_id = sqlx::query_scalar!(
            r#"
            INSERT INTO competition_groups (competition_id, category_id, name)
            VALUES ($1, $2, $3)
            ON CONFLICT (competition_id, category_id, name)
            DO UPDATE SET name = EXCLUDED.name
            RETURNING group_id
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
        group_id: i32,
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

        let bodyweight = athlete_data
            .athlete_info
            .pesee
            .map(|w| Decimal::from_f64_retain(w))
            .flatten();
        let ris_score = Decimal::from_f64_retain(athlete_data.ris);

        let participant_id = sqlx::query_scalar!(
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
            RETURNING participant_id
            "#,
            group_id,
            athlete_id,
            bodyweight,
            rank,
            athlete_data.athlete_info.is_out,
            athlete_data.athlete_info.reason_out,
            ris_score
        )
        .fetch_one(&mut **tx)
        .await?;

        let mut movement_list: Vec<_> = movements.values().collect();
        movement_list.sort_by_key(|m| m.order);

        for movement in movement_list {
            if let Some(movement_results) = athlete_data.results.get(&movement.id.to_string()) {
                self.import_lift(
                    participant_id,
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
    ) -> Result<i32> {
        let gender = map_gender(&category_info.genre);

        let existing = sqlx::query_scalar!(
            r#"
            SELECT athlete_id FROM athletes
            WHERE first_name = $1 AND last_name = $2 AND gender = $3 AND country = $4
            "#,
            athlete_info.first_name,
            athlete_info.last_name,
            gender,
            "FR"
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let athlete_id = sqlx::query_scalar!(
            r#"
            INSERT INTO athletes (first_name, last_name, gender, country)
            VALUES ($1, $2, $3, $4)
            RETURNING athlete_id
            "#,
            athlete_info.first_name,
            athlete_info.last_name,
            gender,
            "FR"
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(athlete_id)
    }

    async fn import_lift(
        &self,
        participant_id: i32,
        movement: &Movement,
        movement_results: &MovementResults,
        athlete_info: &AthleteInfo,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let movement_id = self.upsert_movement(movement, tx).await?;

        let max_weight = Decimal::from_f64_retain(movement_results.max)
            .ok_or_else(|| ImporterError::TransformationError("Invalid max_weight".to_string()))?;

        let settings = get_movement_settings(&movement.name, athlete_info);

        let lift_id = sqlx::query_scalar!(
            r#"
            INSERT INTO lifts (participant_id, movement_id, max_weight, equipment_setting)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (participant_id, movement_id)
            DO UPDATE SET
                max_weight = EXCLUDED.max_weight,
                equipment_setting = EXCLUDED.equipment_setting,
                updated_at = CURRENT_TIMESTAMP
            RETURNING lift_id
            "#,
            participant_id,
            movement_id,
            max_weight,
            settings
        )
        .fetch_one(&mut **tx)
        .await?;

        for i in 1..=3 {
            if let Some(Some(attempt)) = movement_results.results.get(&i.to_string()) {
                self.import_attempt(lift_id, attempt, tx).await?;
            }
        }

        Ok(())
    }

    async fn upsert_movement(
        &self,
        movement: &Movement,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<i32> {
        let movement_id = sqlx::query_scalar!(
            r#"
            INSERT INTO movements (name, display_order)
            VALUES ($1, $2)
            ON CONFLICT (name)
            DO UPDATE SET display_order = EXCLUDED.display_order
            RETURNING movement_id
            "#,
            movement.name,
            movement.order
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(movement_id)
    }

    async fn import_attempt(
        &self,
        lift_id: i32,
        attempt: &Attempt,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let success = match &attempt.decision_rep {
            DecisionRep::Number(n) => *n == 111 || *n == 110,
            DecisionRep::String(s) => s == "111" || s == "110",
        };

        let passing_judges = match &attempt.decision_rep {
            DecisionRep::Number(n) => count_passing_judges(*n),
            DecisionRep::String(s) => s.parse::<i32>().ok().map(count_passing_judges).flatten(),
        };

        let weight = Decimal::from_f64_retain(attempt.charge)
            .ok_or_else(|| ImporterError::TransformationError("Invalid weight".to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO attempts (lift_id, attempt_number, weight, is_successful, passing_judges, no_rep_reason)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (lift_id, attempt_number)
            DO UPDATE SET
                weight = EXCLUDED.weight,
                is_successful = EXCLUDED.is_successful,
                passing_judges = EXCLUDED.passing_judges,
                no_rep_reason = EXCLUDED.no_rep_reason
            "#,
            lift_id,
            attempt.no_essai as i16,
            weight,
            success,
            passing_judges,
            attempt.justification_no_rep
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
