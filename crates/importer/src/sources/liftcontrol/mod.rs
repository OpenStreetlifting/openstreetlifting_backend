mod client;
mod models;
mod movement_mapper;
mod spec;
mod transformer;

pub use client::LiftControlClient;
pub use models::*;
pub use movement_mapper::LiftControlMovementMapper;
pub use spec::{CompetitionConfig, CompetitionId, LiftControlRegistry, LiftControlSpec};
pub use transformer::LiftControlTransformer;

use crate::{ImportContext, Result, traits::CompetitionImporter};
use tracing::info;

/// The LiftControl importer fetches competition data from the LiftControl API platform.
/// It handles competitions that may have multiple sessions/divisions identified by sub-slugs.
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
    type Spec = LiftControlSpec;

    async fn import(&self, spec: &Self::Spec, context: &ImportContext) -> Result<()> {
        info!(
            "Importing competition '{}' from {} sub-slugs",
            spec.base_slug(),
            spec.sub_slugs().len()
        );

        for sub_slug in spec.sub_slugs() {
            let sub_slug = sub_slug.trim();
            if sub_slug.is_empty() {
                continue;
            }

            info!("Fetching data for sub-slug: {}", sub_slug);
            let api_response = self.client.fetch_live_general_table(sub_slug).await?;
            let transformer =
                LiftControlTransformer::with_base_slug(&context.pool, spec.base_slug().to_string());
            info!("Competition status: {}", api_response.contest.status);
            transformer.import_competition(api_response).await?;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "LiftControl"
    }
}
