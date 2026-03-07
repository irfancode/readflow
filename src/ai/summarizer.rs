use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
}

pub struct AiSummarizer {
    model: String,
    endpoint: String,
    enabled: bool,
}

impl AiSummarizer {
    pub fn new(model: Option<String>, endpoint: Option<String>) -> Self {
        let model = model.unwrap_or_else(|| "llama3.2".to_string());
        let endpoint = endpoint.unwrap_or_else(|| "http://localhost:11434".to_string());
        
        let enabled = std::net::TcpStream::connect("localhost:11434").is_ok();
        
        if !enabled {
            warn!("Ollama not running at localhost:11434 - AI features disabled");
        }

        Self {
            model,
            endpoint,
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn summarize(&self, content: &str, max_length: Option<usize>) -> Result<String> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Ollama not available"));
        }

        let max_len = max_length.unwrap_or(500);
        
        let prompt = format!(
            "Summarize the following article in {} characters or less. \
            Focus on the main points and key takeaways:\n\n{}",
            max_len,
            content
        );

        self.generate(&prompt).await
    }

    pub async fn extract_key_points(&self, content: &str) -> Result<Vec<String>> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Ollama not available"));
        }

        let prompt = format!(
            "Extract 5-7 key points from the following article as a bullet list:\n\n{}",
            content
        );

        let response = self.generate(&prompt).await?;
        
        let points: Vec<String> = response
            .lines()
            .filter(|l| l.trim().starts_with('-') || l.trim().starts_with('*'))
            .map(|l| l.trim().trim_start_matches('-').trim_start_matches('*').trim().to_string())
            .collect();

        Ok(points)
    }

    pub async fn suggest_tags(&self, content: &str) -> Result<Vec<String>> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Ollama not available"));
        }

        let prompt = format!(
            "Suggest 5 relevant tags for the following article (comma-separated, no explanation):\n\n{}",
            content
        );

        let response = self.generate(&prompt).await?;
        
        let tags: Vec<String> = response
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();

        Ok(tags)
    }

    pub async fn answer_question(&self, content: &str, question: &str) -> Result<String> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Ollama not available"));
        }

        let prompt = format!(
            "Based on the following article, answer this question: {}\n\nArticle:\n{}",
            question, content
        );

        self.generate(&prompt).await
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let client = reqwest::Client::new();
        
        let response = client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Ollama returned error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(ollama_response.response.trim().to_string())
    }

    pub async fn check_model_available(&self) -> bool {
        let client = reqwest::Client::new();
        
        match client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(models) = response.json::<serde_json::Value>().await {
                        let available: Vec<String> = models["models"]
                            .as_array()
                            .unwrap_or(&Vec::new())
                            .iter()
                            .filter_map(|m| m["name"].as_str().map(String::from))
                            .collect();
                        
                        available.iter().any(|m| m.starts_with(&self.model))
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

impl Default for AiSummarizer {
    fn default() -> Self {
        Self::new(None, None)
    }
}
