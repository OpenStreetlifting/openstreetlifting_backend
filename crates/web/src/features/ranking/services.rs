use sqlx::PgPool;
use storage::{
    dto::ranking::{GlobalRankingEntry, GlobalRankingFilter},
    error::Result,
    repository::ranking::RankingRepository,
};

/// Get global ranking with filtering and pagination
pub async fn get_global_ranking(
    pool: &PgPool,
    filter: &GlobalRankingFilter,
) -> Result<(Vec<GlobalRankingEntry>, i64)> {
    let repo = RankingRepository::new(pool);
    repo.get_global_ranking(filter).await
}
