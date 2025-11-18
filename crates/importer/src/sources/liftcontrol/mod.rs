mod client;
mod models;
mod transformer;

pub use client::LiftControlClient;
pub use models::*;
pub use transformer::LiftControlTransformer;

use crate::{ImportContext, Result, traits::CompetitionImporter};

pub struct LiftControlImporter {
    client: LiftControlClient,
}

impl LiftControlImporter {
    pub fn new() -> Self {
        Self {
            client: LiftControlClient::new(),
        }
    }
}

impl Default for LiftControlImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CompetitionImporter for LiftControlImporter {
    async fn import(&self, event_slug: &str, context: &ImportContext) -> Result<()> {
        let api_response = self.client.fetch_live_general_table(event_slug).await?;
        let transformer = LiftControlTransformer::new(&context.pool);
        transformer.import_competition(api_response).await?;
        Ok(())
    }
}
