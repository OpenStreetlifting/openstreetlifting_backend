use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::models::{RisFormulaVersion, RisScoreHistory};

pub struct RisRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> RisRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_formula_by_id(&self, formula_id: Uuid) -> Result<RisFormulaVersion> {
        let formula = sqlx::query_as!(
            RisFormulaVersion,
            r#"
            SELECT formula_id, year, effective_from, effective_until, is_current,
                   men_a, men_k, men_b, men_v, men_q,
                   women_a, women_k, women_b, women_v, women_q,
                   notes, created_at
            FROM ris_formula_versions
            WHERE formula_id = $1
            "#,
            formula_id
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(formula)
    }

    pub async fn get_formula_by_year(&self, year: i32) -> Result<RisFormulaVersion> {
        let formula = sqlx::query_as!(
            RisFormulaVersion,
            r#"
            SELECT formula_id, year, effective_from, effective_until, is_current,
                   men_a, men_k, men_b, men_v, men_q,
                   women_a, women_k, women_b, women_v, women_q,
                   notes, created_at
            FROM ris_formula_versions
            WHERE year = $1
            "#,
            year
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(formula)
    }

    pub async fn get_current_formula(&self) -> Result<RisFormulaVersion> {
        let formula = sqlx::query_as!(
            RisFormulaVersion,
            r#"
            SELECT formula_id, year, effective_from, effective_until, is_current,
                   men_a, men_k, men_b, men_v, men_q,
                   women_a, women_k, women_b, women_v, women_q,
                   notes, created_at
            FROM ris_formula_versions
            WHERE is_current = true
            LIMIT 1
            "#
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(formula)
    }

    pub async fn get_formula_for_date(&self, date: NaiveDate) -> Result<RisFormulaVersion> {
        let formula = sqlx::query_as!(
            RisFormulaVersion,
            r#"
            SELECT formula_id, year, effective_from, effective_until, is_current,
                   men_a, men_k, men_b, men_v, men_q,
                   women_a, women_k, women_b, women_v, women_q,
                   notes, created_at
            FROM ris_formula_versions
            WHERE effective_from <= $1
              AND (effective_until IS NULL OR effective_until > $1)
            ORDER BY effective_from DESC
            LIMIT 1
            "#,
            date
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(formula)
    }

    pub async fn list_all_formulas(&self) -> Result<Vec<RisFormulaVersion>> {
        let formulas = sqlx::query_as!(
            RisFormulaVersion,
            r#"
            SELECT formula_id, year, effective_from, effective_until, is_current,
                   men_a, men_k, men_b, men_v, men_q,
                   women_a, women_k, women_b, women_v, women_q,
                   notes, created_at
            FROM ris_formula_versions
            ORDER BY year DESC
            "#
        )
        .fetch_all(self.pool)
        .await?;

        Ok(formulas)
    }

    pub async fn upsert_ris_score(
        &self,
        participant_id: Uuid,
        formula_id: Uuid,
        ris_score: Decimal,
        bodyweight: Decimal,
        total_weight: Decimal,
    ) -> Result<RisScoreHistory> {
        let score_history = sqlx::query_as!(
            RisScoreHistory,
            r#"
            INSERT INTO ris_scores_history (participant_id, formula_id, ris_score, bodyweight, total_weight)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (participant_id, formula_id)
            DO UPDATE SET
                ris_score = EXCLUDED.ris_score,
                bodyweight = EXCLUDED.bodyweight,
                total_weight = EXCLUDED.total_weight,
                computed_at = CURRENT_TIMESTAMP
            RETURNING ris_score_id, participant_id, formula_id, ris_score, bodyweight, total_weight, computed_at
            "#,
            participant_id,
            formula_id,
            ris_score,
            bodyweight,
            total_weight
        )
        .fetch_one(self.pool)
        .await?;

        Ok(score_history)
    }

    pub async fn get_participant_ris_history(
        &self,
        participant_id: Uuid,
    ) -> Result<Vec<RisScoreHistory>> {
        let history = sqlx::query_as!(
            RisScoreHistory,
            r#"
            SELECT ris_score_id, participant_id, formula_id, ris_score, bodyweight, total_weight, computed_at
            FROM ris_scores_history
            WHERE participant_id = $1
            ORDER BY computed_at DESC
            "#,
            participant_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(history)
    }

    pub async fn get_participant_ris_for_formula(
        &self,
        participant_id: Uuid,
        formula_id: Uuid,
    ) -> Result<Option<RisScoreHistory>> {
        let score = sqlx::query_as!(
            RisScoreHistory,
            r#"
            SELECT ris_score_id, participant_id, formula_id, ris_score, bodyweight, total_weight, computed_at
            FROM ris_scores_history
            WHERE participant_id = $1 AND formula_id = $2
            "#,
            participant_id,
            formula_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(score)
    }

    pub async fn update_participant_current_ris(
        &self,
        participant_id: Uuid,
        ris_score: Decimal,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE competition_participants
            SET ris_score = $1
            WHERE participant_id = $2
            "#,
            ris_score,
            participant_id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
