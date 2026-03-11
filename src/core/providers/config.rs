// GitHub Copilot
pub const TOKEN_ENDPOINT: &str = "https://api.github.com/copilot_internal/v2/token";
pub const CHAT_ENDPOINT: &str = "https://api.githubcopilot.com/chat/completions";
pub const DEFAULT_MODEL: &str = "gpt-4.1";

pub const TOKEN_TTL_SECS: u64 = 600; // 10 minutes
pub const TOKEN_TIMEOUT_SECS: u64 = 15;
pub const CHAT_TIMEOUT_SECS: u64 = 300;

// OpenAI-compatible base URLs
pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const OLLAMA_BASE_URL: &str = "http://localhost:11434/v1";
