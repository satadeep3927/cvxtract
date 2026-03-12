use crate::core::providers::config::{
    CHAT_TIMEOUT_SECS, OLLAMA_BASE_URL, OPENAI_BASE_URL, OPENROUTER_BASE_URL,
};
use crate::core::providers::schema::{ChatMessage, ChatRequest, ChatResponse};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug)]
pub enum OpenAIError {
    ApiKeyNotFound(String),
    Request(String),
    Parse(String),
}

impl std::fmt::Display for OpenAIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAIError::ApiKeyNotFound(var) => write!(
                f,
                "{var} not set — provide it at compile time or as a runtime env var"
            ),
            OpenAIError::Request(msg) => write!(f, "Request error: {msg}"),
            OpenAIError::Parse(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for OpenAIError {}

/// Resolve an API key: compile-time env first, then runtime env.
fn resolve_api_key(env_var: &str) -> Result<String, OpenAIError> {
    // Compile-time resolution is not possible for dynamic var names,
    // so we go straight to runtime for OpenAI-compatible providers.
    std::env::var(env_var).map_err(|_| OpenAIError::ApiKeyNotFound(env_var.to_string()))
}

/// A generic OpenAI-compatible provider.
/// Works with OpenAI, OpenRouter, Ollama, Together AI, Groq, or any
/// service that implements the `/chat/completions` endpoint.
pub struct OpenAI {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
    system_prompt: String,
}

impl OpenAI {
    /// Create a provider with an explicit API key.
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            system_prompt:
                "You are a helpful AI assistant specialized in analyzing resumes and CVs."
                    .to_string(),
        }
    }

    /// Create a provider resolving the API key from an environment variable.
    pub fn from_env(
        base_url: impl Into<String>,
        env_var: &str,
        model: impl Into<String>,
    ) -> Result<Self, OpenAIError> {
        let api_key = resolve_api_key(env_var)?;
        Ok(Self::new(base_url, api_key, model))
    }

    /// Use the official OpenAI API. Reads `OPENAI_API_KEY` from the environment.
    pub fn for_openai(model: impl Into<String>) -> Result<Self, OpenAIError> {
        Self::from_env(OPENAI_BASE_URL, "OPENAI_API_KEY", model)
    }

    /// Use OpenRouter. Reads `OPENROUTER_API_KEY` from the environment.
    pub fn openrouter(model: impl Into<String>) -> Result<Self, OpenAIError> {
        Self::from_env(OPENROUTER_BASE_URL, "OPENROUTER_API_KEY", model)
    }

    /// Use a local Ollama instance (no API key required).
    pub fn ollama(model: impl Into<String>) -> Self {
        Self::new(OLLAMA_BASE_URL, "", model)
    }

    pub async fn generate(&self, prompt: &str) -> String {
        let request = ChatRequest {
            model: self.model.clone(),
            temperature: Some(0.1),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: self.system_prompt.clone(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        };

        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let mut req = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(CHAT_TIMEOUT_SECS))
            .json(&request);

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        match req.send().await {
            Err(e) => format!("Error: {e}"),
            Ok(resp) if !resp.status().is_success() => {
                let body = resp.text().await.unwrap_or_default();
                format!("Error: {body}")
            }
            Ok(resp) => resp
                .json::<ChatResponse>()
                .await
                .ok()
                .and_then(|r| r.choices.into_iter().next())
                .map(|c| c.message.content.trim().to_string())
                .unwrap_or_default(),
        }
    }
}
