use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::athlete::{
    AthleteCompetitionSummary, AthleteDetailResponse, CreateAthleteRequest, PersonalRecord,
    UpdateAthleteRequest,
};
use crate::error::{Result, StorageError};
use crate::models::Athlete;

pub struct AthleteRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AthleteRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// List all athletes
    pub async fn list(&self) -> Result<Vec<Athlete>> {
        let athletes = sqlx::query_as!(
            Athlete,
            r#"
            SELECT athlete_id, first_name, last_name, gender, created_at,
                   nationality, country, profile_picture_url, slug,
                   COALESCE(slug_history, '[]'::jsonb) as "slug_history!: sqlx::types::Json<Vec<String>>"
            FROM athletes
            ORDER BY last_name, first_name
            "#
        )
        .fetch_all(self.pool)
        .await?;

        Ok(athletes)
    }

    /// Find athlete by slug (or check slug_history for redirects)
    pub async fn find_by_slug(&self, slug: &str) -> Result<Athlete> {
        // First try to find by current slug
        let athlete = sqlx::query_as!(
            Athlete,
            r#"
            SELECT athlete_id, first_name, last_name, gender, created_at,
                   nationality, country, profile_picture_url, slug,
                   COALESCE(slug_history, '[]'::jsonb) as "slug_history!: sqlx::types::Json<Vec<String>>"
            FROM athletes
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(self.pool)
        .await?;

        if let Some(athlete) = athlete {
            return Ok(athlete);
        }

        // If not found, check slug_history for redirect
        let athlete_from_history = sqlx::query_as!(
            Athlete,
            r#"
            SELECT athlete_id, first_name, last_name, gender, created_at,
                   nationality, country, profile_picture_url, slug,
                   COALESCE(slug_history, '[]'::jsonb) as "slug_history!: sqlx::types::Json<Vec<String>>"
            FROM athletes
            WHERE slug_history @> to_jsonb($1::text)
            "#,
            slug
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(athlete_from_history)
    }

    /// Find athlete by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Athlete> {
        let athlete = sqlx::query_as!(
            Athlete,
            r#"
            SELECT athlete_id, first_name, last_name, gender, created_at,
                   nationality, country, profile_picture_url, slug,
                   COALESCE(slug_history, '[]'::jsonb) as "slug_history!: sqlx::types::Json<Vec<String>>"
            FROM athletes
            WHERE athlete_id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(athlete)
    }

    /// Get detailed athlete info with competition history
    pub async fn find_by_slug_detailed(&self, slug: &str) -> Result<AthleteDetailResponse> {
        let athlete = self.find_by_slug(slug).await?;
        self.get_detailed_athlete(athlete).await
    }

    /// Helper to build detailed athlete response
    async fn get_detailed_athlete(&self, athlete: Athlete) -> Result<AthleteDetailResponse> {
        // Get competition history
        let competitions = sqlx::query_as!(
            AthleteCompetitionSummary,
            r#"
            SELECT
                c.competition_id,
                c.name as competition_name,
                c.slug as competition_slug,
                c.start_date as competition_date,
                cat.name as category_name,
                cp.rank,
                COALESCE(SUM(l.max_weight), 0) as "total!: Decimal",
                cp.ris_score,
                cp.is_disqualified
            FROM competition_participants cp
            JOIN competitions c ON cp.competition_id = c.competition_id
            JOIN categories cat ON cp.category_id = cat.category_id
            LEFT JOIN lifts l ON l.participant_id = cp.participant_id
            WHERE cp.athlete_id = $1
            GROUP BY c.competition_id, c.name, c.slug, c.start_date, cat.name, cp.rank, cp.ris_score, cp.is_disqualified
            ORDER BY c.start_date DESC NULLS LAST
            "#,
            athlete.athlete_id
        )
        .fetch_all(self.pool)
        .await?;

        // Get personal records
        let personal_records = sqlx::query_as!(
            PersonalRecord,
            r#"
            SELECT DISTINCT ON (l.movement_name)
                l.movement_name,
                l.max_weight,
                c.name as competition_name,
                c.slug as competition_slug,
                c.start_date as date
            FROM lifts l
            JOIN competition_participants cp ON l.participant_id = cp.participant_id
            JOIN competitions c ON cp.competition_id = c.competition_id
            WHERE cp.athlete_id = $1
            ORDER BY l.movement_name, l.max_weight DESC
            "#,
            athlete.athlete_id
        )
        .fetch_all(self.pool)
        .await?;

        // Count total competitions
        let total_competitions = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT cp.competition_id)::bigint as "count!"
            FROM competition_participants cp
            WHERE cp.athlete_id = $1
            "#,
            athlete.athlete_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(AthleteDetailResponse {
            athlete_id: athlete.athlete_id,
            first_name: athlete.first_name,
            last_name: athlete.last_name,
            slug: athlete.slug,
            gender: athlete.gender,
            nationality: athlete.nationality,
            country: athlete.country,
            profile_picture_url: athlete.profile_picture_url,
            created_at: athlete.created_at,
            competitions,
            personal_records,
            total_competitions,
        })
    }

    /// Generate unique slug from first and last name
    pub async fn generate_unique_slug(&self, first_name: &str, last_name: &str) -> Result<String> {
        let base_slug = format!("{}-{}", first_name, last_name)
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-");

        let base_slug = if base_slug.is_empty() {
            "athlete".to_string()
        } else {
            base_slug
        };

        let mut final_slug = base_slug.clone();
        let mut counter = 2;

        while sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM athletes WHERE slug = $1)",
            final_slug
        )
        .fetch_one(self.pool)
        .await?
        .unwrap_or(false)
        {
            final_slug = format!("{}-{}", base_slug, counter);
            counter += 1;
        }

        Ok(final_slug)
    }

    /// Create a new athlete
    pub async fn create(&self, req: &CreateAthleteRequest) -> Result<Athlete> {
        let slug = self
            .generate_unique_slug(&req.first_name, &req.last_name)
            .await?;

        let athlete = sqlx::query_as!(
            Athlete,
            r#"
            INSERT INTO athletes (first_name, last_name, gender, nationality, country, profile_picture_url, slug)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING athlete_id, first_name, last_name, gender, created_at,
                      nationality, country, profile_picture_url, slug,
                      COALESCE(slug_history, '[]'::jsonb) as "slug_history!: sqlx::types::Json<Vec<String>>"
            "#,
            req.first_name,
            req.last_name,
            req.gender,
            req.nationality,
            req.country,
            req.profile_picture_url,
            slug
        )
        .fetch_one(self.pool)
        .await?;

        Ok(athlete)
    }

    /// Update an existing athlete
    pub async fn update(
        &self,
        id: Uuid,
        existing: &Athlete,
        req: &UpdateAthleteRequest,
    ) -> Result<Athlete> {
        let first_name = req.first_name.as_ref().unwrap_or(&existing.first_name);
        let last_name = req.last_name.as_ref().unwrap_or(&existing.last_name);
        let gender = req.gender.as_ref().unwrap_or(&existing.gender);
        let nationality = req.nationality.as_ref().or(existing.nationality.as_ref());
        let country = req.country.as_ref().unwrap_or(&existing.country);
        let profile_picture_url = req
            .profile_picture_url
            .as_ref()
            .or(existing.profile_picture_url.as_ref());

        // Check if name changed - if so, generate new slug and store old one
        let (slug, slug_history) = if req.first_name.is_some() || req.last_name.is_some() {
            let new_slug = self.generate_unique_slug(first_name, last_name).await?;
            let mut history = existing.slug_history.0.clone();
            history.push(existing.slug.clone());
            (new_slug, sqlx::types::Json(history))
        } else {
            (existing.slug.clone(), existing.slug_history.clone())
        };

        let athlete = sqlx::query_as!(
            Athlete,
            r#"
            UPDATE athletes
            SET first_name = $2,
                last_name = $3,
                gender = $4,
                nationality = $5,
                country = $6,
                profile_picture_url = $7,
                slug = $8,
                slug_history = $9
            WHERE athlete_id = $1
            RETURNING athlete_id, first_name, last_name, gender, created_at,
                      nationality, country, profile_picture_url, slug,
                      COALESCE(slug_history, '[]'::jsonb) as "slug_history!: sqlx::types::Json<Vec<String>>"
            "#,
            id,
            first_name,
            last_name,
            gender,
            nationality,
            country,
            profile_picture_url,
            slug,
            slug_history as _
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(athlete)
    }

    /// Delete an athlete by ID
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query!("DELETE FROM athletes WHERE athlete_id = $1", id)
            .execute(self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        Ok(())
    }
}
