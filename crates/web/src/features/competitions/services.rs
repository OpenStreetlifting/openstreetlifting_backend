use sqlx::PgPool;
use storage::{
    dto::competition::{
        CompetitionDetailResponse, CompetitionListResponse, CreateCompetitionRequest,
        UpdateCompetitionRequest,
    },
    error::Result,
    models::Competition,
    repository::competition::CompetitionRepository,
};

/// List all competitions
pub async fn list_competitions(pool: &PgPool) -> Result<Vec<Competition>> {
    let repo = CompetitionRepository::new(pool);
    repo.list().await
}

/// List competitions with detailed information
pub async fn list_competitions_detailed(pool: &PgPool) -> Result<Vec<CompetitionListResponse>> {
    let repo = CompetitionRepository::new(pool);
    repo.list_with_details().await
}

/// Get competition by slug
pub async fn get_competition_by_slug(pool: &PgPool, slug: &str) -> Result<Competition> {
    let repo = CompetitionRepository::new(pool);
    repo.find_by_slug(slug).await
}

/// Get competition with detailed information
pub async fn get_competition_detailed(
    pool: &PgPool,
    slug: &str,
) -> Result<CompetitionDetailResponse> {
    let repo = CompetitionRepository::new(pool);
    repo.find_by_slug_detailed(slug).await
}

/// Create a new competition
pub async fn create_competition(
    pool: &PgPool,
    request: &CreateCompetitionRequest,
) -> Result<Competition> {
    let repo = CompetitionRepository::new(pool);
    repo.create(request).await
}

/// Update a competition
pub async fn update_competition(
    pool: &PgPool,
    slug: &str,
    request: &UpdateCompetitionRequest,
) -> Result<Competition> {
    let repo = CompetitionRepository::new(pool);

    let existing = repo.find_by_slug(slug).await?;
    repo.update(existing.competition_id, &existing, request)
        .await
}

/// Delete a competition
pub async fn delete_competition(pool: &PgPool, slug: &str) -> Result<()> {
    let repo = CompetitionRepository::new(pool);
    let competition = repo.find_by_slug(slug).await?;
    repo.delete(competition.competition_id).await
}
