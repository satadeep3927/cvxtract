# cvxtract

[![Crates.io](https://img.shields.io/crates/v/cvxtract)](https://crates.io/crates/cvxtract)
[![Docs.rs](https://docs.rs/cvxtract/badge.svg)](https://docs.rs/cvxtract)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

LLM-powered structured extraction from CVs and resumes.

**cvxtract** loads a CV/resume in any common format (PDF, DOCX, HTML, plain text),
sends the text to an LLM of your choice, and deserialises the response directly
into typed Rust structs — no regex, no hand-written parsers.

## Quick start

```rust
use cvxtract::{Extractor, Model};

#[tokio::main]
async fn main() {
    // No API key needed — model is downloaded automatically on first run.
    let mut extractor = Extractor::new(Some(Model::from_local()));

    match extractor.extract_resume("resume.pdf".into()).await {
        Ok(resume) => {
            println!("Name:  {}", resume.name);
            println!("Email: {}", resume.email.as_deref().unwrap_or("-"));
            println!("Jobs:  {}", resume.experience.len());
        }
        Err(e) => eprintln!("Extraction failed: {e}"),
    }
}
```

## Installation

```toml
[dependencies]
cvxtract = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Providers

| Constructor | Backend | Auth |
|---|---|---|
| `Model::from_local()` | llama-cpp-2 on-device (Qwen3.5-2B) | none — model auto-downloaded |
| `Model::from_openai()` | OpenAI API | `OPENAI_API_KEY` |
| `Model::from_openrouter()` | OpenRouter | `OPENROUTER_API_KEY` |
| `Model::from_ollama()` | Local Ollama | Ollama running on `localhost:11434` |
| `Model::from_openai_compatible()` | Any OpenAI-compatible endpoint | explicit key + URL |
| `Model::from_copilot()` | GitHub Copilot | `COPILOT_TOKEN` |

```rust
// OpenAI
let model = Model::from_openai("gpt-4o-mini");

// Ollama (local)
let model = Model::from_ollama("llama3.2");

// Any OpenAI-compatible endpoint
let model = Model::from_openai_compatible(
    "https://api.my-provider.com/v1",
    std::env::var("MY_API_KEY").unwrap(),
    "my-model-name",
);
```

## Supported input formats

| Format | Extension |
|---|---|
| PDF | `.pdf` |
| Word | `.docx` |
| HTML | `.html`, `.htm` |
| Plain text | `.txt` |

## Built-in `Resume` type

`extract_resume()` populates a comprehensive `Resume` struct:

```rust
pub struct Resume {
    pub name:           String,
    pub email:          Option<String>,
    pub phone:          Option<String>,
    pub location:       Option<String>,
    pub linkedin:       Option<String>,
    pub github:         Option<String>,
    pub website:        Option<String>,
    pub summary:        Option<String>,
    pub experience:     Vec<Experience>,   // company, role, dates, highlights
    pub education:      Vec<Education>,    // institution, degree, field, dates
    pub skills:         Vec<SkillGroup>,   // grouped or flat skill lists
    pub projects:       Vec<Project>,      // name, tech stack, URL
    pub certifications: Vec<Certification>,
    pub languages:      Vec<Language>,
    pub awards:         Vec<Award>,
}
```

## Custom types

Extract *any* shape by deriving `serde::Deserialize` and `schemars::JsonSchema`:

```rust
use cvxtract::{Extractor, Model};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
struct ContactInfo {
    name:  String,
    email: Option<String>,
    phone: Option<String>,
}

#[tokio::main]
async fn main() {
    let mut extractor = Extractor::new(Some(Model::from_openai("gpt-4o-mini")));
    let info: ContactInfo = extractor
        .extract::<ContactInfo>("resume.pdf".into())
        .await
        .unwrap();
    println!("{info:#?}");
}
```

## GPU acceleration (local model)

When using `Model::from_local()`, compile with a feature flag to offload layers to
your GPU. llama.cpp auto-fits what it can into VRAM and falls back to CPU for the
remainder — this is safe even on GPUs with limited memory.

```bash
# NVIDIA CUDA
cargo build --release --features cuda

# Apple Silicon (Metal)
cargo build --release --features metal

# AMD / Intel Vulkan
cargo build --release --features vulkan
```

```toml
# Cargo.toml
[dependencies]
cvxtract = { version = "0.1", features = ["cuda"] }
```

## Error handling

All async methods return `Result<T, ExtractionError>`:

```rust
use cvxtract::ExtractionError;

match extractor.extract_resume(path).await {
    Ok(resume) => { /* use resume */ }
    Err(ExtractionError::LoadError(e))  => eprintln!("Could not load file: {e}"),
    Err(ExtractionError::ModelError(m)) => eprintln!("LLM error: {m}"),
    Err(ExtractionError::ParseError(e)) => eprintln!("JSON parse error: {e}"),
}
```

## Raw document loading

Use `UnstructuredLoader` to extract plain text from a file without any LLM call:

```rust
use cvxtract::UnstructuredLoader;

let loader = UnstructuredLoader::new();
let doc = loader.load("resume.pdf")?;
println!("{} characters extracted", doc.content.len());
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

3. Write tests for your changes
4. Submit a pull request

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.