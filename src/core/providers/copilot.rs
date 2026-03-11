use crate::core::providers::config::{
    CHAT_ENDPOINT, CHAT_TIMEOUT_SECS, DEFAULT_MODEL, TOKEN_ENDPOINT, TOKEN_TIMEOUT_SECS,
    TOKEN_TTL_SECS,
};
use crate::core::providers::schema::{ChatMessage, ChatRequest, ChatResponse};
use reqwest::Client;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug)]
pub enum CopilotError {
    TokenNotFound,
    TokenExchange(String),
    Request(String),
    Parse(String),
}

impl std::fmt::Display for CopilotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopilotError::TokenNotFound => write!(
                f,
                "COPILOT_TOKEN not set — provide it at compile time or as a runtime env var"
            ),
            CopilotError::TokenExchange(msg) => write!(f, "Token exchange error: {}", msg),
            CopilotError::Request(msg) => write!(f, "Request error: {}", msg),
            CopilotError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for CopilotError {}

/// Resolve COPILOT_TOKEN: compile-time env first, then runtime env.
fn resolve_copilot_token() -> Result<String, CopilotError> {
    // 1. Compile-time: set via `COPILOT_TOKEN=xxx cargo build`
    if let Some(token) = option_env!("COPILOT_TOKEN") {
        return Ok(token.to_string());
    }
    // 2. Runtime: set via shell environment
    std::env::var("COPILOT_TOKEN").map_err(|_| CopilotError::TokenNotFound)
}

pub struct Copilot {
    client: Client,
    model: String,
    copilot_token: String,
    api_key: Mutex<String>,
    last_token_refresh: Mutex<std::time::Instant>,
}

impl Copilot {
    /// Create a new Copilot provider, resolving the token automatically.
    /// Returns an error if `COPILOT_TOKEN` is not available.
    pub fn new(model: Option<String>) -> Result<Self, CopilotError> {
        let copilot_token = resolve_copilot_token()?;
        Ok(Self {
            client: Client::new(),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            copilot_token,
            api_key: Mutex::new(String::new()),
            // Set to past time to force refresh on first call
            last_token_refresh: Mutex::new(
                std::time::Instant::now() - Duration::from_secs(TOKEN_TTL_SECS + 1),
            ),
        })
    }

    pub fn is_token_valid(&self) -> bool {
        let Ok(key) = self.api_key.lock() else {
            return false;
        };
        let Ok(last_refresh) = self.last_token_refresh.lock() else {
            return false;
        };

        !key.is_empty() && last_refresh.elapsed() < Duration::from_secs(TOKEN_TTL_SECS)
    }

    async fn refresh_token(&self) -> Result<(), CopilotError> {
        let fetch = async {
            let response = self
                .client
                .get(TOKEN_ENDPOINT)
                .header("Authorization", format!("token {}", self.copilot_token))
                .header("Editor-Version", "vscode/1.103.1")
                .header("Editor-Plugin-Version", "copilot.vim/1.16.0")
                .header("User-Agent", "GithubCopilot/1.155.0")
                .timeout(Duration::from_secs(TOKEN_TIMEOUT_SECS))
                .send()
                .await
                .map_err(|e| CopilotError::Request(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(CopilotError::TokenExchange(format!("{}: {}", status, body)));
            }

            let data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| CopilotError::Parse(e.to_string()))?;

            let token = data
                .get("token")
                .and_then(|t| t.as_str())
                .ok_or_else(|| CopilotError::Parse("Missing token field".to_string()))?;

            *self.api_key.lock().unwrap() = token.to_string();
            *self.last_token_refresh.lock().unwrap() = std::time::Instant::now();

            Ok(())
        };

        timeout(Duration::from_secs(TOKEN_TIMEOUT_SECS + 5), fetch)
            .await
            .map_err(|_| CopilotError::TokenExchange("Token refresh timed out".to_string()))?
    }

    pub async fn generate(&self, prompt: &str) -> String {
        if !self.is_token_valid() {
            if let Err(e) = self.refresh_token().await {
                return format!("Error: {}", e);
            }
        }

        let api_key = self.api_key.lock().unwrap().clone();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content:
                        "You are a helpful AI assistant specialized in analyzing resumes and CVs."
                            .to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        };

        let response = self
            .client
            .post(CHAT_ENDPOINT)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("Editor-Version", "vscode/1.103.1")
            .header("Editor-Plugin-Version", "copilot.vim/1.16.0")
            .header("User-Agent", "GithubCopilot/1.155.0")
            .timeout(Duration::from_secs(CHAT_TIMEOUT_SECS))
            .json(&request)
            .send()
            .await;

        match response {
            Err(e) => format!("Error: {}", e),
            Ok(resp) if !resp.status().is_success() => {
                let body = resp.text().await.unwrap_or_default();
                format!("Error: {}", body)
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
