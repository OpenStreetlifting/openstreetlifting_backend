use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::competition::{
    CompetitionListResponse, CreateCompetitionRequest, FederationInfo, MovementInfo,
};
use crate::error::{Result, StorageError};
use crate::models::{Competition, CompetitionMovement, Federation};

pub struct CompetitionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> CompetitionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

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

    pub async fn list_with_details(&self) -> Result<Vec<CompetitionListResponse>> {
        let competitions = self.list().await?;
        let mut results = Vec::with_capacity(competitions.len());

        for comp in competitions {
            let federation = sqlx::query_as!(
                Federation,
                "SELECT federation_id, name, rulebook_id, country, abbreviation
                 FROM federations
                 WHERE federation_id = $1",
                comp.federation_id
            )
            .fetch_one(self.pool)
            .await?;

            let movements = sqlx::query_as!(
                CompetitionMovement,
                "SELECT competition_id, movement_name, is_required, display_order
                 FROM competition_movements
                 WHERE competition_id = $1
                 ORDER BY display_order",
                comp.competition_id
            )
            .fetch_all(self.pool)
            .await?;

            results.push(CompetitionListResponse {
                competition_id: comp.competition_id,
                name: comp.name,
                created_at: comp.created_at,
                slug: comp.slug,
                status: comp.status,
                venue: comp.venue,
                city: comp.city,
                country: comp.country,
                start_date: comp.start_date,
                end_date: comp.end_date,
                federation: FederationInfo {
                    federation_id: federation.federation_id,
                    name: federation.name,
                    abbreviation: federation.abbreviation,
                    country: federation.country,
                },
                movements: movements
                    .into_iter()
                    .map(|m| MovementInfo {
                        movement_name: m.movement_name,
                        is_required: m.is_required,
                        display_order: m.display_order,
                    })
                    .collect(),
            });
        }

        Ok(results)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Competition> {
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
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code().as_deref() == Some("23505")
            {
                return StorageError::ConstraintViolation("Slug already exists".to_string());
            }
            StorageError::from(e)
        })?;

        Ok(competition)
    }

    /// Update an existing competition
    pub async fn update(
        &self,
        id: Uuid,
        existing: &Competition,
        req: &crate::dto::competition::UpdateCompetitionRequest,
    ) -> Result<Competition> {
        // Merge update fields with existing data
        let name = req.name.as_ref().unwrap_or(&existing.name);
        let slug = req.slug.as_ref().unwrap_or(&existing.slug);
        let status = req.status.as_ref().unwrap_or(&existing.status);
        let federation_id = req.federation_id.unwrap_or(existing.federation_id);
        let venue = req.venue.as_ref().or(existing.venue.as_ref());
        let city = req.city.as_ref().or(existing.city.as_ref());
        let country = req.country.as_ref().or(existing.country.as_ref());
        let start_date = req.start_date.or(existing.start_date);
        let end_date = req.end_date.or(existing.end_date);
        let number_of_judge = req.number_of_judge.or(existing.number_of_judge);

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
    pub async fn delete(&self, id: Uuid) -> Result<()> {
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
