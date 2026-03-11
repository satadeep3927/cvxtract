# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-03-11

### Added
- `UnstructuredLoader` — load PDF, DOCX, HTML, and plain-text CVs/resumes into a unified `Document` type
- `Extractor` — LLM-powered structured extraction from any loaded document
  - `extract<T>()` — generic extraction into any `serde::Deserialize + schemars::JsonSchema` type
  - `extract_resume()` — built-in extraction into the comprehensive `Resume` type
- `Resume` and nested types (`Experience`, `Education`, `SkillGroup`, `Project`, `Certification`, `Language`, `Award`, `DateRange`, `PartialDate`)
- Multi-provider `Model` abstraction:
  - `Model::from_local()` — on-device inference via llama-cpp-2 (Qwen3.5-2B-Q4_K_M)
  - `Model::from_openai()` — OpenAI API
  - `Model::from_openrouter()` — OpenRouter
  - `Model::from_ollama()` — local Ollama instance
  - `Model::from_openai_compatible()` — any OpenAI-compatible endpoint
  - `Model::from_copilot()` — GitHub Copilot
- GPU acceleration feature flags: `cuda`, `metal`, `vulkan`
- `ExtractionError` — structured error type covering load, model, and parse failures
