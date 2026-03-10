use llama_cpp_rs::{LlamaModel, LlamaContext, LlamaSession, SessionParams, LlamaParams};
use std::path::{Path, PathBuf};

pub struct Local {
    session: Option<LlamaSession>,
    model_path: PathBuf,
}

#[derive(Debug)]
pub enum LocalError {
    LoadError(String),
    InferenceError(String),
    DownloadError(String),
}

impl std::fmt::Display for LocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalError::LoadError(msg) => write!(f, "Model load error: {}", msg),
            LocalError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            LocalError::DownloadError(msg) => write!(f, "Download error: {}", msg),
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
        
        let model_path = cache_dir.join("Qwen3.5-2B-Q4_K_M.gguf");
        
        Self {
            session: None,
            model_path,
        }
    }
    
    /// Initialize the model (download if needed, then load)
    pub async fn initialize(&mut self) -> Result<(), LocalError> {
        // Ensure model exists (download if needed)
        self.ensure_model_exists().await?;
        
        // Load the model with LLamaCPP
        self.load_model()?;
        
        Ok(())
    }
    
    /// Check if model exists in cache, download if not
    async fn ensure_model_exists(&self) -> Result<(), LocalError> {
        if self.model_path.exists() {
            println!("✅ Model found in cache: {}", self.model_path.display());
            return Ok(());
        }
        
        println!("📥 Model not found, downloading...");
        self.download_model().await?;
        Ok(())
    }
    
    /// Download model from HuggingFace
    async fn download_model(&self) -> Result<(), LocalError> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;
        
        // Create cache directory
        if let Some(parent) = self.model_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LocalError::DownloadError(format!("Failed to create cache dir: {}", e)))?;
        }
        
        let url = "https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/resolve/main/Qwen3.5-2B-Q4_K_M.gguf";
        
        println!("🌐 Downloading from: {}", url);
        
        let client = reqwest::Client::new();
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| LocalError::DownloadError(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(LocalError::DownloadError(format!(
                "HTTP error: {}", response.status()
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
                print!("\r📥 Downloaded: {:.1}% ({:.1}MB / {:.1}MB)", 
                    progress, 
                    downloaded as f64 / 1_048_576.0,
                    total_size as f64 / 1_048_576.0
                );
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }
        }
        
        println!("\n✅ Download complete: {}", self.model_path.display());
        Ok(())
    }
    
    /// Load the model with LLamaCPP
    fn load_model(&mut self) -> Result<(), LocalError> {
        println!("🔄 Loading model with LLamaCPP...");
        
        // Configure LLamaCPP parameters
        let model_params = LlamaParams {
            n_ctx: 2048,      // Context size
            n_batch: 8,       // Batch size  
            n_gpu_layers: 20, // GPU layers (adjust based on your GPU)
            ..Default::default()
        };
        
        // Load the model
        let model = LlamaModel::load_from_file(&self.model_path, model_params)
            .map_err(|e| LocalError::LoadError(format!("Failed to load model: {:?}", e)))?;
        
        // Create context
        let ctx_params = SessionParams {
            n_len: 512,    // Max response length
            n_ctx: 2048,   // Context window size
            ..Default::default()
        };
        
        // Create session
        let session = LlamaSession::new(model, ctx_params)
            .map_err(|e| LocalError::LoadError(format!("Failed to create session: {:?}", e)))?;
        
        self.session = Some(session);
        
        println!("✅ Model loaded successfully!");
        Ok(())
    }
    
    /// Generate text using the loaded model
    pub fn generate(&mut self, prompt: &str) -> String {
        let session = match &mut self.session {
            Some(session) => session,
            None => {
                return "❌ Model not initialized. Call initialize() first.".to_string();
            }
        };
        
        // Build the complete prompt with chat template
        let formatted_prompt = format!(
            "<|im_start|>system\nYou are a helpful AI assistant that analyzes documents and answers questions about them.<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            prompt
        );
        
        println!("🔄 Generating response...");
        
        // Generate response
        match session.inference_with_prompt(&formatted_prompt) {
            Ok(response) => {
                println!("✅ Generation complete");
                response
            }
            Err(e) => {
                eprintln!("❌ Generation error: {:?}", e);
                format!("Error generating response: {:?}", e)
            }
        }
    }
    
    /// Check if model is ready for inference
    pub fn is_ready(&self) -> bool {
        self.session.is_some() && self.model_path.exists()
    }
    
    /// Get model path
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }
}