use llama_cpp_2::{
    context::{params::LlamaContextParams, LlamaContext},
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel},
    sampling::LlamaSampler,
};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Local {
    backend: Option<LlamaBackend>,
    model: Option<LlamaModel>,
    model_path: PathBuf,
    initialized: bool,
}

#[derive(Debug)]
pub enum LocalError {
    LoadError(String),
    InferenceError(String),
    DownloadError(String),
    ContextError(String),
}

impl std::fmt::Display for LocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalError::LoadError(msg) => write!(f, "Model load error: {}", msg),
            LocalError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            LocalError::DownloadError(msg) => write!(f, "Download error: {}", msg),
            LocalError::ContextError(msg) => write!(f, "Context error: {}", msg),
        }
    }
}

impl std::error::Error for LocalError {}

impl Local {
    pub fn new() -> Self {
        let cache_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".cache")
            .join("models");

        // Back to Qwen 3.5 model for better performance with llama-cpp-2
        let model_path = cache_dir.join("Qwen3.5-2B-Q4_K_M.gguf");

        Self {
            backend: None,
            model: None,
            model_path,
            initialized: false,
        }
    }

    /// Initialize the model with llama-cpp-2 API (download if needed, then load)
    pub async fn initialize(&mut self) -> Result<(), LocalError> {
        // Ensure model exists (download if needed)
        self.ensure_model_exists().await?;

        // Initialize the llama backend
        let backend = LlamaBackend::init()
            .map_err(|e| LocalError::LoadError(format!("Failed to initialize backend: {:?}", e)))?;

        self.backend = Some(backend);

        // Load the model with default parameters
        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(
            self.backend.as_ref().unwrap(),
            &self.model_path,
            &model_params,
        )
        .map_err(|e| LocalError::LoadError(format!("Failed to load Qwen 3.5 model: {:?}", e)))?;

        self.model = Some(model);
        self.initialized = true;

        Ok(())
    }

    /// Check if model exists in cache, download if not
    async fn ensure_model_exists(&self) -> Result<(), LocalError> {
        if self.model_path.exists() {
            return Ok(());
        }

        println!("Preparing AI model for document analysis...");
        self.download_model().await?;
        Ok(())
    }

    /// Download model from HuggingFace
    async fn download_model(&self) -> Result<(), LocalError> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        // Create cache directory
        if let Some(parent) = self.model_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LocalError::DownloadError(format!("Failed to create cache dir: {}", e))
            })?;
        }

        let url =
            "https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/resolve/main/Qwen3.5-2B-Q4_K_M.gguf";

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| LocalError::DownloadError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalError::DownloadError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);

        let mut file = tokio::fs::File::create(&self.model_path)
            .await
            .map_err(|e| LocalError::DownloadError(format!("Failed to create file: {}", e)))?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| LocalError::DownloadError(format!("Download chunk error: {}", e)))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| LocalError::DownloadError(format!("Write error: {}", e)))?;

            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                print!("\rDownloading AI model: {:.0}%", progress);
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }
        }

        println!("\nAI model ready for document analysis.");
        Ok(())
    }

    /// Generate text using the loaded Qwen 3.5 model
    pub async fn generate(&mut self, prompt: &str) -> String {
        // Auto-initialize if not done yet
        if !self.initialized {
            if let Err(e) = self.initialize().await {
                return format!("Initialization error: {}", e);
            }
        }

        let model = match &self.model {
            Some(model) => model,
            None => {
                return "Model not loaded.".to_string();
            }
        };

        let backend = match &self.backend {
            Some(backend) => backend,
            None => {
                return "Backend not initialized.".to_string();
            }
        };

        // Create context with sufficient size for large CVs
        let context_params =
            LlamaContextParams::default().with_n_ctx(std::num::NonZeroU32::new(8192));

        let mut context = match model.new_context(backend, context_params) {
            Ok(ctx) => ctx,
            Err(e) => {
                return format!("Failed to create context: {:?}", e);
            }
        };

        // Build the complete prompt with Qwen 3.5 chat template.
        // The <think> prefix activates Qwen 3.5's native thinking mode — no system instructions needed.
        let formatted_prompt = format!(
            "<|im_start|>system\nYou are a helpful AI assistant specialized in analyzing resumes and CVs.<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n<think>\n",
            prompt
        );

        // Tokenize the prompt using the model
        let tokens = match model.str_to_token(&formatted_prompt, AddBos::Always) {
            Ok(tokens) => tokens,
            Err(e) => {
                return format!("Tokenization error: {:?}", e);
            }
        };

        // Create batch and add tokens
        let mut batch = LlamaBatch::new(512, 1);
        let last_index = tokens.len() as i32 - 1;
        for (i, token) in (0_i32..).zip(tokens.into_iter()) {
            let is_last = i == last_index;
            if let Err(e) = batch.add(token, i, &[0], is_last) {
                return format!("Batch add error: {:?}", e);
            }
        }

        // Evaluate the prompt tokens
        if let Err(e) = context.decode(&mut batch) {
            return format!("Decode error: {:?}", e);
        }

        let mut response = String::new();
        let max_tokens = 10_00_000;
        let mut n_cur = batch.n_tokens();

        // Initialize decoder and sampler
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut sampler = LlamaSampler::greedy();

        // Generate tokens one by one
        for _ in 0..max_tokens {
            // Sample a token
            let new_token = sampler.sample(&context, batch.n_tokens() - 1);
            sampler.accept(new_token);

            // Check for end-of-sequence
            if new_token == model.token_eos() {
                break;
            }

            // Convert token to string using the model
            if let Ok(token_str) = model.token_to_piece(new_token, &mut decoder, true, None) {
                response.push_str(&token_str);

                // Stop on end markers
                if response.contains("<|im_end|>") || response.contains("</s>") {
                    break;
                }
            }

            // Prepare next batch
            batch.clear();
            if let Err(e) = batch.add(new_token, n_cur, &[0], true) {
                return format!("Batch error: {:?}", e);
            }
            n_cur += 1;

            // Decode the new token
            if let Err(e) = context.decode(&mut batch) {
                return format!("Decode error: {:?}", e);
            }
        }

        // Clean up the response and extract thinking
        let cleaned = response
            .split("<|im_end|>")
            .next()
            .unwrap_or(&response)
            .trim()
            .to_string();

        // Extract thinking content and final response
        self.extract_thinking_and_response(&cleaned)
    }

    /// Extract thinking process and return clean final response
    fn extract_thinking_and_response(&self, text: &str) -> String {
        // Since <think> is injected in the prompt, the response starts directly with
        // thinking content. We only need to find the closing </think> tag.
        if let Some(think_end) = text.find("</think>") {
            return text[think_end + 8..].trim().to_string();
        }

        // Fallback: handle case where model re-added the opening <think> tag
        if let (Some(think_start), Some(think_end)) = (text.find("<think>"), text.find("</think>"))
        {
            if think_start < think_end {
                return text[think_end + 8..].trim().to_string();
            }
        }

        // No thinking tags found, return text as-is
        text.trim().to_string()
    }

    /// Check if model is ready for inference
    pub fn is_ready(&self) -> bool {
        self.initialized
            && self.backend.is_some()
            && self.model.is_some()
            && self.model_path.exists()
    }

    /// Get model path
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }
}
