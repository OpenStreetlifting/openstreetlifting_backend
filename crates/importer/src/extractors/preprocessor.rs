use crate::error::ImporterError;
use std::time::Duration;

type Result<T> = std::result::Result<T, ImporterError>;

const HTTP_TIMEOUT_SECS: u64 = 30;
const USER_AGENT: &str = "OpenStreetLifting Importer/1.0";

fn build_http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| ImporterError::ExtractionError(format!("HTTP client error: {}", e)))
}

fn check_response_status(response: &reqwest::Response, url: &str) -> Result<()> {
    if response.status().is_success() {
        Ok(())
    } else {
        Err(ImporterError::ExtractionError(format!(
            "HTTP error {}: {}",
            response.status(),
            url
        )))
    }
}

pub struct Preprocessor;

impl Preprocessor {
    pub async fn fetch_html(url: &str) -> Result<String> {
        tracing::info!("Fetching HTML from: {}", url);

        let client = build_http_client()?;
        let response = client.get(url).send().await?;
        check_response_status(&response, url)?;

        let html = response.text().await?;
        tracing::info!("Fetched {} bytes of HTML", html.len());
        Ok(html)
    }

    pub async fn read_csv(path: &str) -> Result<String> {
        let content = tokio::fs::read_to_string(path).await?;
        tracing::info!("Read CSV file: {} ({} bytes)", path, content.len());
        Ok(content)
    }

    pub async fn read_image_as_base64(path: &str) -> Result<String> {
        let bytes = tokio::fs::read(path).await?;
        let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        tracing::info!(
            "Read image file: {} ({} bytes, {} base64)",
            path,
            bytes.len(),
            base64.len()
        );
        Ok(base64)
    }

    pub async fn fetch_image_as_base64(url: &str) -> Result<String> {
        tracing::info!("Fetching image from: {}", url);

        let client = build_http_client()?;
        let response = client.get(url).send().await?;
        check_response_status(&response, url)?;

        let bytes = response.bytes().await?;
        let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        tracing::info!(
            "Fetched image: {} bytes, {} base64",
            bytes.len(),
            base64.len()
        );
        Ok(base64)
    }

    pub fn truncate_to_tokens(text: &str, max_tokens: usize) -> &str {
        let max_chars = max_tokens * 4;

        if text.len() <= max_chars {
            return text;
        }

        tracing::warn!(
            "Truncating content from {} to ~{} tokens",
            text.len(),
            max_tokens
        );

        text[..max_chars]
            .rfind('\n')
            .or_else(|| text[..max_chars].rfind(' '))
            .map(|pos| &text[..pos])
            .unwrap_or(&text[..max_chars])
    }
}
