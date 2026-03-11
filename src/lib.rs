//! # cvxtract
//!
//! LLM-powered structured extraction from CVs and resumes.
//!
//! **cvxtract** loads a CV/resume in any common format (PDF, DOCX, HTML, plain text),
//! sends the text to an LLM, and deserialises the response directly into typed Rust
//! structs — no regex, no hand-written parsers.
//!
//! ## Quick start
//!
//! ```no_run
//! use cvxtract::{Extractor, Model};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Use any provider — here we use a local quantised model (no API key required).
//!     let mut extractor = Extractor::new(Some(Model::from_local()));
//!
//!     match extractor.extract_resume("resume.pdf".into()).await {
//!         Ok(resume) => println!("{:#?}", resume),
//!         Err(e) => eprintln!("Extraction failed: {e}"),
//!     }
//! }
//! ```
//!
//! ## Providers
//!
//! | Constructor | Backend | Requires |
//! |---|---|---|
//! | [`Model::from_local()`] | llama-cpp-2 on-device (Qwen3.5-2B) | nothing — model auto-downloaded |
//! | [`Model::from_openai()`] | OpenAI API | `OPENAI_API_KEY` env var |
//! | [`Model::from_openrouter()`] | OpenRouter | `OPENROUTER_API_KEY` env var |
//! | [`Model::from_ollama()`] | Local Ollama | Ollama running on `localhost:11434` |
//! | [`Model::from_openai_compatible()`] | Any OpenAI-compatible endpoint | explicit key + URL |
//! | [`Model::from_copilot()`] | GitHub Copilot | `COPILOT_TOKEN` env var |
//!
//! ## GPU acceleration
//!
//! Compile with a feature flag to offload the local model to your GPU:
//!
//! ```bash
//! # NVIDIA CUDA
//! cargo build --release --features cuda
//! # Apple Silicon (Metal)
//! cargo build --release --features metal
//! # AMD / Intel / Vulkan
//! cargo build --release --features vulkan
//! ```
//!
//! ## Custom types
//!
//! Implement [`serde::Deserialize`] and [`schemars::JsonSchema`] on any struct to
//! extract *arbitrary* shapes from a CV:
//!
//! ```no_run
//! use cvxtract::{Extractor, Model};
//! use schemars::JsonSchema;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize, JsonSchema)]
//! struct ContactInfo {
//!     name: String,
//!     email: Option<String>,
//!     phone: Option<String>,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut extractor = Extractor::new(Some(Model::from_local()));
//!     let info: ContactInfo = extractor.extract("resume.pdf".into()).await.unwrap();
//!     println!("{:#?}", info);
//! }
//! ```

mod core;

// ── Public API ──────────────────────────────────────────────────────────────

pub use core::extractor::error::ExtractionError;
pub use core::extractor::resume::{
    Award, Certification, DateRange, Education, Experience, Language, PartialDate, Project, Resume,
    SkillGroup,
};
pub use core::extractor::Extractor;
pub use core::loaders::{Document, DocumentElement, DocumentMetadata, FileType, LoaderError};
pub use core::model::Model;
pub use core::unstructured::UnstructuredLoader;
