use crate::core::loaders::LoaderError;

/// Errors that can occur during structured extraction.
#[derive(Debug)]
pub enum ExtractionError {
    /// The document could not be read or parsed (PDF, DOCX, etc.)
    LoadError(LoaderError),
    /// The LLM returned an error or an empty response.
    ModelError(String),
    /// The LLM output could not be parsed as JSON or deserialised into the target type.
    ParseError(serde_json::Error),
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionError::LoadError(e) => write!(f, "Document load error: {e}"),
            ExtractionError::ModelError(msg) => write!(f, "Model error: {msg}"),
            ExtractionError::ParseError(e) => write!(f, "JSON parse error: {e}"),
        }
    }
}

impl std::error::Error for ExtractionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExtractionError::LoadError(e) => Some(e),
            ExtractionError::ParseError(e) => Some(e),
            ExtractionError::ModelError(_) => None,
        }
    }
}

impl From<LoaderError> for ExtractionError {
    fn from(e: LoaderError) -> Self {
        ExtractionError::LoadError(e)
    }
}

impl From<serde_json::Error> for ExtractionError {
    fn from(e: serde_json::Error) -> Self {
        ExtractionError::ParseError(e)
    }
}
