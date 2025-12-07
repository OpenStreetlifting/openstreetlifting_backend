use crate::canonical::models::CanonicalFormat;
use crate::error::ImporterError;
use crate::extractors::{
    ollama_client::OllamaClient, preprocessor::Preprocessor, prompts::PromptBuilder,
};
use std::path::Path;

type Result<T> = std::result::Result<T, ImporterError>;

pub struct HtmlExtractor {
    ollama: OllamaClient,
    max_tokens: usize,
}

impl HtmlExtractor {
    pub fn new(ollama_url: String, model: String) -> Self {
        Self {
            ollama: OllamaClient::new(ollama_url, model),
            max_tokens: 16000,
        }
    }

    pub async fn extract_from_url(&self, url: &str) -> Result<CanonicalFormat> {
        let html = Preprocessor::fetch_html(url).await?;
        self.extract_from_html(&html).await
    }

    pub async fn extract_from_html(&self, html: &str) -> Result<CanonicalFormat> {
        tracing::info!("Starting HTML extraction ({} bytes)", html.len());

        let truncated = Preprocessor::truncate_to_tokens(html, self.max_tokens);
        let system_prompt = PromptBuilder::system_prompt();
        let user_prompt = PromptBuilder::user_prompt_html(truncated);

        let json_response = self
            .ollama
            .generate_json(&system_prompt, &user_prompt, None)
            .await?;

        let canonical: CanonicalFormat = serde_json::from_str(&json_response)
            .map_err(|e| ImporterError::ExtractionError(format!("Invalid JSON from LLM: {}", e)))?;

        tracing::info!("Extraction complete: {}", canonical.competition.name);
        Ok(canonical)
    }

    pub async fn extract_and_save(
        &self,
        url: &str,
        output_dir: &Path,
    ) -> Result<std::path::PathBuf> {
        let canonical = self.extract_from_url(url).await?;

        let competition_dir = output_dir.join(&canonical.competition.slug);
        tokio::fs::create_dir_all(&competition_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
        let filename = format!("{}_html.json", timestamp);
        let filepath = competition_dir.join(&filename);

        let json = serde_json::to_string_pretty(&canonical)?;
        tokio::fs::write(&filepath, json).await?;

        tracing::info!("Saved to: {}", filepath.display());
        Ok(filepath)
    }
}

impl Default for HtmlExtractor {
    fn default() -> Self {
        Self::new(
            "http://localhost:11434".to_string(),
            "qwen2.5:7b".to_string(),
        )
    }
}

pub struct CsvExtractor {
    ollama: OllamaClient,
    max_tokens: usize,
}

impl CsvExtractor {
    pub fn new(ollama_url: String, model: String) -> Self {
        Self {
            ollama: OllamaClient::new(ollama_url, model),
            max_tokens: 16000,
        }
    }

    pub async fn extract_from_file(&self, path: &str) -> Result<CanonicalFormat> {
        let csv = Preprocessor::read_csv(path).await?;
        self.extract_from_csv(&csv).await
    }

    pub async fn extract_from_csv(&self, csv: &str) -> Result<CanonicalFormat> {
        tracing::info!("Starting CSV extraction ({} bytes)", csv.len());

        let truncated = Preprocessor::truncate_to_tokens(csv, self.max_tokens);
        let system_prompt = PromptBuilder::system_prompt();
        let user_prompt = PromptBuilder::user_prompt_csv(truncated);

        let json_response = self
            .ollama
            .generate_json(&system_prompt, &user_prompt, None)
            .await?;

        let canonical: CanonicalFormat = serde_json::from_str(&json_response)
            .map_err(|e| ImporterError::ExtractionError(format!("Invalid JSON from LLM: {}", e)))?;

        tracing::info!("Extraction complete: {}", canonical.competition.name);
        Ok(canonical)
    }

    pub async fn extract_and_save(
        &self,
        path: &str,
        output_dir: &Path,
    ) -> Result<std::path::PathBuf> {
        let canonical = self.extract_from_file(path).await?;

        let competition_dir = output_dir.join(&canonical.competition.slug);
        tokio::fs::create_dir_all(&competition_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
        let filename = format!("{}_csv.json", timestamp);
        let filepath = competition_dir.join(&filename);

        let json = serde_json::to_string_pretty(&canonical)?;
        tokio::fs::write(&filepath, json).await?;

        tracing::info!("Saved to: {}", filepath.display());
        Ok(filepath)
    }
}

impl Default for CsvExtractor {
    fn default() -> Self {
        Self::new(
            "http://localhost:11434".to_string(),
            "qwen2.5:7b".to_string(),
        )
    }
}

pub struct ImageExtractor {
    ollama: OllamaClient,
}

impl ImageExtractor {
    pub fn new(ollama_url: String, model: String) -> Self {
        Self {
            ollama: OllamaClient::new(ollama_url, model),
        }
    }

    pub async fn extract_from_file(&self, path: &str) -> Result<CanonicalFormat> {
        let image_base64 = Preprocessor::read_image_as_base64(path).await?;
        self.extract_from_image(&image_base64).await
    }

    pub async fn extract_from_url(&self, url: &str) -> Result<CanonicalFormat> {
        let image_base64 = Preprocessor::fetch_image_as_base64(url).await?;
        self.extract_from_image(&image_base64).await
    }

    async fn extract_from_image(&self, image_base64: &str) -> Result<CanonicalFormat> {
        tracing::info!("Starting image extraction");

        let system_prompt = PromptBuilder::system_prompt();
        let user_prompt = PromptBuilder::user_prompt_image();

        let json_response = self
            .ollama
            .generate_json_from_image(&system_prompt, &user_prompt, image_base64, None)
            .await?;

        let canonical: CanonicalFormat = serde_json::from_str(&json_response)
            .map_err(|e| ImporterError::ExtractionError(format!("Invalid JSON from LLM: {}", e)))?;

        tracing::info!("Extraction complete: {}", canonical.competition.name);
        Ok(canonical)
    }

    pub async fn extract_and_save(
        &self,
        source: &str,
        output_dir: &Path,
        is_url: bool,
    ) -> Result<std::path::PathBuf> {
        let canonical = if is_url {
            self.extract_from_url(source).await?
        } else {
            self.extract_from_file(source).await?
        };

        let competition_dir = output_dir.join(&canonical.competition.slug);
        tokio::fs::create_dir_all(&competition_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
        let filename = format!("{}_image.json", timestamp);
        let filepath = competition_dir.join(&filename);

        let json = serde_json::to_string_pretty(&canonical)?;
        tokio::fs::write(&filepath, json).await?;

        tracing::info!("Saved to: {}", filepath.display());
        Ok(filepath)
    }
}

impl Default for ImageExtractor {
    fn default() -> Self {
        Self::new("http://localhost:11434".to_string(), "llava:7b".to_string())
    }
}
