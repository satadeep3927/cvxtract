use crate::core::providers::copilot::{Copilot, CopilotError};
use crate::core::providers::local::Local;
use crate::core::providers::openai::{OpenAI, OpenAIError};

/// Internal enum that dispatches generation to the right backend.
pub enum ModelProvider {
    Local(Local),
    Copilot(Copilot),
    OpenAI(OpenAI),
}

/// An LLM provider that can generate text from a prompt.
///
/// Create one using the provider-specific constructors (e.g. [`Model::from_local`],
/// [`Model::from_openai`]) and pass it to [`crate::Extractor::new`].
///
/// # Example
/// ```no_run
/// use cvxtract::Model;
///
/// // On-device inference — no API key or network required.
/// let model = Model::from_local();
///
/// // OpenAI — reads OPENAI_API_KEY from the environment.
/// let model = Model::from_openai("gpt-4o").unwrap();
///
/// // Ollama running locally.
/// let model = Model::from_ollama("qwen2.5:7b");
/// ```
pub struct Model {
    provider: ModelProvider,
}

impl Model {
    /// Use the on-device local provider (Qwen3.5-2B-Q4_K_M via llama-cpp-2).
    ///
    /// The GGUF model is downloaded automatically on first use and cached in
    /// `.cache/models/`. No API key or network access is required after download.
    /// Compile with `--features cuda`, `--features metal`, or `--features vulkan`
    /// to enable GPU acceleration.
    ///
    /// Use [`Model::from_local_with_temperature`] to customise the sampling temperature.
    pub fn from_local() -> Self {
        Self {
            provider: ModelProvider::Local(Local::new().with_temperature(0.1)),
        }
    }

    /// Use GitHub Copilot as the backend.
    ///
    /// Requires `COPILOT_TOKEN` to be set (compile-time via `COPILOT_TOKEN=xxx cargo build`
    /// or as a runtime environment variable). The token is automatically exchanged for a
    /// short-lived API key via the Copilot internal endpoint.
    ///
    /// `model` defaults to `gpt-4.1` when `None`.
    pub fn from_copilot(model: Option<String>) -> Result<Self, CopilotError> {
        Ok(Self {
            provider: ModelProvider::Copilot(Copilot::new(model)?),
        })
    }

    /// Use the official OpenAI API. Reads `OPENAI_API_KEY` from the environment.
    pub fn from_openai(model: impl Into<String>) -> Result<Self, OpenAIError> {
        Ok(Self {
            provider: ModelProvider::OpenAI(OpenAI::for_openai(model)?),
        })
    }

    /// Use OpenRouter. Reads `OPENROUTER_API_KEY` from the environment.
    pub fn from_openrouter(model: impl Into<String>) -> Result<Self, OpenAIError> {
        Ok(Self {
            provider: ModelProvider::OpenAI(OpenAI::openrouter(model)?),
        })
    }

    /// Use a local [Ollama](https://ollama.com) instance — no API key required.
    ///
    /// Ollama must be running on `http://localhost:11434`. Any model available
    /// in Ollama can be used, e.g. `"qwen2.5:7b"`, `"llama3.2"`, `"mistral"`.
    pub fn from_ollama(model: impl Into<String>) -> Self {
        Self {
            provider: ModelProvider::OpenAI(OpenAI::ollama(model)),
        }
    }

    /// Use any OpenAI-compatible endpoint with an explicit base URL and API key.
    ///
    /// Works with Together AI, Groq, Anyscale, and any other service that
    /// implements the `/v1/chat/completions` endpoint.
    pub fn from_openai_compatible(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            provider: ModelProvider::OpenAI(OpenAI::new(base_url, api_key, model)),
        }
    }

    /// Send `prompt` to the model and return the raw text response.
    pub async fn generate(&mut self, prompt: &str) -> String {
        match &mut self.provider {
            ModelProvider::Local(local) => local.generate(prompt).await,
            ModelProvider::Copilot(copilot) => copilot.generate(prompt).await,
            ModelProvider::OpenAI(openai) => openai.generate(prompt).await,
        }
    }

    /// Returns `true` if the model is ready to accept requests.
    ///
    /// For the local provider this means the model file exists on disk and has
    /// been loaded. For API-based providers this always returns `true`.
    pub fn is_ready(&self) -> bool {
        match &self.provider {
            ModelProvider::Local(local) => local.is_ready(),
            ModelProvider::Copilot(_) | ModelProvider::OpenAI(_) => true,
        }
    }
}
