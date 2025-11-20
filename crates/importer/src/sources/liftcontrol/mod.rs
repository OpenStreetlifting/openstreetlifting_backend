mod client;
mod models;
mod movement_mapper;
mod transformer;

pub use client::LiftControlClient;
pub use models::*;
pub use movement_mapper::LiftControlMovementMapper;
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
    async fn import(&self, identifier: &str, context: &ImportContext) -> Result<()> {
        // identifier format: "base_slug:sub_slug1,sub_slug2,sub_slug3"
        // Example: "annecy-4-lift-2025:annecy-4-lift-2025-dimanche-matin-39,annecy-4-lift-2025-dimanche-apres-midi-40"

        let parts: Vec<&str> = identifier.split(':').collect();
        if parts.len() != 2 {
            return Err(crate::ImporterError::TransformationError(
                "Invalid identifier format. Expected 'base_slug:sub_slug1,sub_slug2,...'".to_string()
            ));
        }

        let base_slug = parts[0].to_string();
        let sub_slugs: Vec<&str> = parts[1].split(',').collect();

        if sub_slugs.is_empty() {
            return Err(crate::ImporterError::TransformationError(
                "No sub-slugs provided".to_string()
            ));
        }

        info!("Importing competition with base slug '{}' from {} sub-slugs", base_slug, sub_slugs.len());

        for sub_slug in sub_slugs {
            let sub_slug = sub_slug.trim();
            if sub_slug.is_empty() {
                continue;
            }

            info!("Fetching data for sub-slug: {}", sub_slug);
            let api_response = self.client.fetch_live_general_table(sub_slug).await?;
            let transformer = LiftControlTransformer::with_base_slug(&context.pool, base_slug.clone());
            info!("Competition status: {}", api_response.contest.status);
            transformer.import_competition(api_response).await?;
        }

        Ok(())
    }
}
