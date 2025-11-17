use sqlx::PgPool;

use crate::dto::competition::CreateCompetitionRequest;
use crate::error::{Result, StorageError};
use crate::models::Competition;

/// Repository for Competition database operations
pub struct CompetitionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> CompetitionRepository<'a> {
    /// Create a new CompetitionRepository
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// List all competitions
    pub async fn list(&self) -> Result<Vec<Competition>> {
        let competitions = sqlx::query_as!(
            Competition,
            r#"
            SELECT competition_id, name, created_at, slug, status, federation_id,
                   venue, city, country, start_date, end_date, number_of_judge
            FROM competitions
            ORDER BY start_date DESC, created_at DESC
            "#
        )
        .fetch_all(self.pool)
        .await?;

        Ok(competitions)
    }

    /// Get a competition by ID
    pub async fn find_by_id(&self, id: i32) -> Result<Competition> {
        let competition = sqlx::query_as!(
            Competition,
            r#"
            SELECT competition_id, name, created_at, slug, status, federation_id,
                   venue, city, country, start_date, end_date, number_of_judge
            FROM competitions
            WHERE competition_id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(competition)
    }

    /// Get a competition by slug
    pub async fn find_by_slug(&self, slug: &str) -> Result<Competition> {
        let competition = sqlx::query_as!(
            Competition,
            r#"
            SELECT competition_id, name, created_at, slug, status, federation_id,
                   venue, city, country, start_date, end_date, number_of_judge
            FROM competitions
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(competition)
    }

    /// Create a new competition
    pub async fn create(&self, req: &CreateCompetitionRequest) -> Result<Competition> {
        let competition = sqlx::query_as!(
            Competition,
            r#"
            INSERT INTO competitions (
                name, slug, status, federation_id, venue, city, country,
                start_date, end_date, number_of_judge
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING competition_id, name, created_at, slug, status, federation_id,
                      venue, city, country, start_date, end_date, number_of_judge
            "#,
            req.name,
            req.slug,
            req.status,
            req.federation_id,
            req.venue,
            req.city,
            req.country,
            req.start_date,
            req.end_date,
            req.number_of_judge
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            // Handle unique constraint violations for slug
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.code().as_deref() == Some("23505") {
                    return StorageError::ConstraintViolation("Slug already exists".to_string());
                }
            }
            StorageError::from(e)
        })?;

        Ok(competition)
    }

    /// Update an existing competition
    pub async fn update(
        &self,
        id: i32,
        name: String,
        slug: String,
        status: String,
        federation_id: i32,
        venue: Option<String>,
        city: Option<String>,
        country: Option<String>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        number_of_judge: Option<i16>,
    ) -> Result<Competition> {
        let competition = sqlx::query_as!(
            Competition,
            r#"
            UPDATE competitions
            SET
                name = $2,
                slug = $3,
                status = $4,
                federation_id = $5,
                venue = $6,
                city = $7,
                country = $8,
                start_date = $9,
                end_date = $10,
                number_of_judge = $11
            WHERE competition_id = $1
            RETURNING competition_id, name, created_at, slug, status, federation_id,
                      venue, city, country, start_date, end_date, number_of_judge
            "#,
            id,
            name,
            slug,
            status,
            federation_id,
            venue,
            city,
            country,
            start_date,
            end_date,
            number_of_judge
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(competition)
    }

    /// Delete a competition by ID
    pub async fn delete(&self, id: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM competitions
            WHERE competition_id = $1
            "#,
            id
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        Ok(())
    }
}
