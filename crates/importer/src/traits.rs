use crate::Result;
use sqlx::PgPool;

pub struct ImportContext {
    pub pool: PgPool,
}

#[async_trait::async_trait]
pub trait CompetitionImporter: Send + Sync {
    async fn import(&self, identifier: &str, context: &ImportContext) -> Result<()>;
}
