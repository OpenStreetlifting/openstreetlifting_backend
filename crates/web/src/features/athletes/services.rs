use sqlx::PgPool;
use storage::{
    dto::athlete::{
        AthleteDetailResponse, CreateAthleteRequest, UpdateAthleteRequest,
    },
    error::Result,
    models::Athlete,
    repository::athlete::AthleteRepository,
};

/// List all athletes
pub async fn list_athletes(pool: &PgPool) -> Result<Vec<Athlete>> {
    let repo = AthleteRepository::new(pool);
    repo.list().await
}

/// Get athlete by slug
pub async fn get_athlete_by_slug(pool: &PgPool, slug: &str) -> Result<Athlete> {
    let repo = AthleteRepository::new(pool);
    repo.find_by_slug(slug).await
}

/// Get athlete with detailed information
pub async fn get_athlete_detailed(pool: &PgPool, slug: &str) -> Result<AthleteDetailResponse> {
    let repo = AthleteRepository::new(pool);
    repo.find_by_slug_detailed(slug).await
}

/// Create a new athlete
pub async fn create_athlete(pool: &PgPool, request: &CreateAthleteRequest) -> Result<Athlete> {
    let repo = AthleteRepository::new(pool);
    repo.create(request).await
}

/// Update an athlete
pub async fn update_athlete(
    pool: &PgPool,
    slug: &str,
    request: &UpdateAthleteRequest,
) -> Result<Athlete> {
    let repo = AthleteRepository::new(pool);

    let existing = repo.find_by_slug(slug).await?;
    repo.update(existing.athlete_id, &existing, request).await
}

/// Delete an athlete
pub async fn delete_athlete(pool: &PgPool, slug: &str) -> Result<()> {
    let repo = AthleteRepository::new(pool);
    let athlete = repo.find_by_slug(slug).await?;
    repo.delete(athlete.athlete_id).await
}
