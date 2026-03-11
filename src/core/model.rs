use crate::core::providers::copilot::{Copilot, CopilotError};
use crate::core::providers::local::Local;
use crate::core::providers::openai::{OpenAI, OpenAIError};

/// Enum representing different model providers
pub enum ModelProvider {
    Local(Local),
    Copilot(Copilot),
    OpenAI(OpenAI),
}

/// Common Model struct that can work with different providers
pub struct Model {
    provider: ModelProvider,
}

impl Model {
    pub fn from_local() -> Self {
        Self {
            provider: ModelProvider::Local(Local::new()),
        }
    }

    pub fn from_copilot(model: Option<String>) -> Result<Self, CopilotError> {
        Ok(Self {
            provider: ModelProvider::Copilot(Copilot::new(model)?),
        })
    }

    /// OpenAI official API — reads `OPENAI_API_KEY` from the environment.
    pub fn from_openai(model: impl Into<String>) -> Result<Self, OpenAIError> {
        Ok(Self {
            provider: ModelProvider::OpenAI(OpenAI::openai(model)?),
        })
    }

    /// OpenRouter — reads `OPENROUTER_API_KEY` from the environment.
    pub fn from_openrouter(model: impl Into<String>) -> Result<Self, OpenAIError> {
        Ok(Self {
            provider: ModelProvider::OpenAI(OpenAI::openrouter(model)?),
        })
    }

    /// Local Ollama instance — no API key required.
    pub fn from_ollama(model: impl Into<String>) -> Self {
        Self {
            provider: ModelProvider::OpenAI(OpenAI::ollama(model)),
        }
    }

    /// Any OpenAI-compatible endpoint with an explicit API key.
    pub fn from_openai_compatible(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            provider: ModelProvider::OpenAI(OpenAI::new(base_url, api_key, model)),
        }
    }

    pub async fn generate(&mut self, prompt: &str) -> String {
        match &mut self.provider {
            ModelProvider::Local(local) => local.generate(prompt).await,
            ModelProvider::Copilot(copilot) => copilot.generate(prompt).await,
            ModelProvider::OpenAI(openai) => openai.generate(prompt).await,
        }
    }

    pub fn is_ready(&self) -> bool {
        match &self.provider {
            ModelProvider::Local(local) => local.is_ready(),
            ModelProvider::Copilot(_) | ModelProvider::OpenAI(_) => true,
        }
    }
}
