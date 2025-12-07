use crate::error::ImporterError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

type Result<T> = std::result::Result<T, ImporterError>;

#[derive(Debug, Clone, Serialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: i64,
    pub digest: String,
    pub modified_at: String,
}

#[derive(Debug, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

/// Client for interacting with Ollama API
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
    default_options: OllamaOptions,
}

impl OllamaClient {
    /// Create a new Ollama client
    ///
    /// # Arguments
    /// * `base_url` - Base URL of Ollama API (e.g., "http://localhost:11434")
    /// * `model` - Model name (e.g., "qwen2.5:7b")
    pub fn new(base_url: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes for large extractions
            .build()
            .expect("Failed to create HTTP client");

        // Default options optimized for structured JSON extraction
        let default_options = OllamaOptions {
            temperature: Some(0.1),    // Low temperature for consistency
            top_p: Some(0.9),          // Focused sampling
            top_k: Some(40),           // Limit vocabulary
            repeat_penalty: Some(1.1), // Avoid repetition
            num_predict: Some(8192),   // Max tokens to generate
        };

        Self {
            client,
            base_url,
            model,
            default_options,
        }
    }

    /// Generate structured JSON from a prompt
    ///
    /// # Arguments
    /// * `system_prompt` - System instructions
    /// * `user_prompt` - User input (e.g., HTML content to extract)
    /// * `custom_options` - Optional custom generation parameters
    pub async fn generate_json(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        custom_options: Option<OllamaOptions>,
    ) -> Result<String> {
        self.generate_json_internal(system_prompt, user_prompt, None, custom_options)
            .await
    }

    pub async fn generate_json_from_image(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        image_base64: &str,
        custom_options: Option<OllamaOptions>,
    ) -> Result<String> {
        self.generate_json_internal(
            system_prompt,
            user_prompt,
            Some(vec![image_base64.to_string()]),
            custom_options,
        )
        .await
    }

    async fn generate_json_internal(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        images: Option<Vec<String>>,
        custom_options: Option<OllamaOptions>,
    ) -> Result<String> {
        let mut request = serde_json::json!({
            "model": self.model.clone(),
            "prompt": user_prompt.to_string(),
            "system": system_prompt.to_string(),
            "stream": false,
            "format": "json",
            "options": custom_options.unwrap_or_else(|| self.default_options.clone()),
        });

        if let Some(imgs) = images {
            request["images"] = serde_json::json!(imgs);
        }

        tracing::info!(
            "Sending request to Ollama (model: {}, prompt length: {} chars, has_images: {})",
            self.model,
            user_prompt.len(),
            request.get("images").is_some()
        );

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ImporterError::ExtractionError(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ImporterError::ExtractionError(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaGenerateResponse = response.json().await.map_err(|e| {
            ImporterError::ExtractionError(format!("Failed to parse Ollama response: {}", e))
        })?;

        if let (Some(total), Some(eval)) = (
            ollama_response.total_duration,
            ollama_response.eval_duration,
        ) {
            let total_secs = total as f64 / 1_000_000_000.0;
            let tokens_per_sec = if eval > 0 {
                (ollama_response.response.len() as f64 / 4.0) / (eval as f64 / 1_000_000_000.0)
            } else {
                0.0
            };

            tracing::info!(
                "Ollama generation complete: {:.2}s total, ~{:.1} tokens/sec, {} chars output",
                total_secs,
                tokens_per_sec,
                ollama_response.response.len()
            );
        }

        Ok(ollama_response.response)
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| ImporterError::ExtractionError(format!("Failed to list models: {}", e)))?;

        let models_response: OllamaModelsResponse = response.json().await.map_err(|e| {
            ImporterError::ExtractionError(format!("Failed to parse models response: {}", e))
        })?;

        Ok(models_response.models)
    }

    /// Check if Ollama service is available
    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| ImporterError::ExtractionError(format!("Health check failed: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// Verify the configured model is available
    pub async fn verify_model(&self) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m.name.starts_with(&self.model)))
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new(
            "http://localhost:11434".to_string(),
            "qwen2.5:7b".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run when Ollama is running
    async fn test_ollama_health_check() {
        let client = OllamaClient::default();
        let result = client.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run when Ollama is running
    async fn test_list_models() {
        let client = OllamaClient::default();
        let models = client.list_models().await.unwrap();
        assert!(!models.is_empty());
    }

    #[tokio::test]
    #[ignore] // Only run when Ollama is running
    async fn test_simple_json_generation() {
        let client = OllamaClient::default();
        let system = "Extract the name from the text and return JSON: {\"name\": \"...\"}";
        let user = "John Doe competed in the competition.";

        let result = client.generate_json(system, user, None).await;
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.get("name").is_some());
    }
}
