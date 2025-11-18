use super::models::ApiResponse;
use crate::error::Result;

pub struct LiftControlClient {
    base_url: String,
    client: reqwest::Client,
}

impl LiftControlClient {
    pub fn new() -> Self {
        Self {
            base_url: "https://liftcontrol.fr".to_string(),
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_live_general_table(&self, event_slug: &str) -> Result<ApiResponse> {
        let url = format!(
            "{}/evenements-liftcontrol/get-live-data/tableau-general/{}",
            self.base_url, event_slug
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ApiResponse>().await?;

        Ok(data)
    }
}

impl Default for LiftControlClient {
    fn default() -> Self {
        Self::new()
    }
}
