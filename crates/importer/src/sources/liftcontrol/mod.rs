mod client;
mod models;
mod transformer;

pub use client::LiftControlClient;
pub use models::*;
use tracing::info;
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
    async fn import(&self, _: &str, context: &ImportContext) -> Result<()> {
        // All slugs for Annecy 4 Lift 2025 competition
        // These will all be grouped into a single competition with base slug "annecy-4-lift-2025"
        const SLUGS: [&str; 2] = [
            "annecy-4-lift-2025-dimanche-matin-39",
            "annecy-4-lift-2025-dimanche-apres-midi-40",
            // Future slugs to add as they become available:
            // "annecy-4-lift-2025-samedi-matin-XX",
            // "annecy-4-lift-2025-samedi-apres-midi-XX",
        ];

        for slug in SLUGS {
            let api_response = self.client.fetch_live_general_table(slug).await?;
            let transformer = LiftControlTransformer::new(&context.pool);
            info!(api_response.contest.status);
            transformer.import_competition(api_response).await?;
        }
        Ok(())
    }
}
