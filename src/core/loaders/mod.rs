use std::fmt;
use std::path::Path;

pub mod docx;
pub mod html;
pub mod pdf;
pub mod text;

/// Represents extracted document content with metadata
#[derive(Debug, Clone)]
pub struct Document {
    /// The main text content extracted from the document
    pub content: String,
    /// Document metadata (file type, size, etc.)
    pub metadata: DocumentMetadata,
    /// Optional structured elements (headings, contact info, etc.)
    pub elements: Vec<DocumentElement>,
}

/// Document metadata
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    /// File name
    pub filename: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Detected file type
    pub file_type: FileType,
    /// Number of pages (if applicable)
    pub page_count: Option<u32>,
}

/// Supported file types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileType {
    Pdf,
    Docx,
    Text,
    Html,
    Rtf,
    Unknown,
}

/// Document elements (structured content)
#[derive(Debug, Clone)]
pub struct DocumentElement {
    /// Type of element (e.g., "heading", "contact_info", "experience")
    pub element_type: String,
    /// Text content of the element
    pub text: String,
    /// Optional metadata for the element
    pub metadata: Option<String>,
}

/// Error types for document loading
#[derive(Debug)]
pub enum LoaderError {
    IoError(std::io::Error),
    FormatError(String),
    UnsupportedFormat(String),
    ParseError(String),
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoaderError::IoError(err) => write!(f, "IO error: {err}"),
            LoaderError::FormatError(msg) => write!(f, "Format error: {msg}"),
            LoaderError::UnsupportedFormat(format) => write!(f, "Unsupported format: {format}"),
            LoaderError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for LoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoaderError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for LoaderError {
    fn from(error: std::io::Error) -> Self {
        LoaderError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, LoaderError>;

/// Core trait for document loaders
pub trait DocumentLoader {
    /// Load a document from a file path
    fn load_from_path(&self, path: &Path) -> Result<Document>;

    /// Load a document from raw bytes
    fn load_from_bytes(&self, data: &[u8], filename: Option<&str>) -> Result<Document>;

    /// Check if this loader supports the given file type
    fn supports_format(&self, file_type: &FileType) -> bool;

    /// Get the primary format this loader handles
    fn primary_format(&self) -> FileType;
}

/// Utility functions for file type detection
pub mod utils {
    use super::FileType;
    use std::path::Path;

    /// Detect file type from file extension
    pub fn detect_file_type<P: AsRef<Path>>(path: P) -> FileType {
        if let Some(ext) = path.as_ref().extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                "pdf" => FileType::Pdf,
                "docx" => FileType::Docx,
                "txt" | "text" => FileType::Text,
                "html" | "htm" => FileType::Html,
                "rtf" => FileType::Rtf,
                _ => FileType::Unknown,
            }
        } else {
            FileType::Unknown
        }
    }

    /// Detect file type from file magic bytes
    pub fn detect_from_bytes(data: &[u8]) -> FileType {
        if data.len() < 4 {
            return FileType::Unknown;
        }

        // PDF magic bytes
        if data.starts_with(b"%PDF") {
            return FileType::Pdf;
        }

        // ZIP-based formats (DOCX is a ZIP file)
        if data.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            // Could be DOCX, but we'd need to inspect the ZIP contents to be sure
            return FileType::Docx;
        }

        // HTML
        if data.starts_with(b"<!DOCTYPE html")
            || data.starts_with(b"<html")
            || data.starts_with(b"<HTML")
        {
            return FileType::Html;
        }

        // RTF
        if data.starts_with(b"{\\rtf") {
            return FileType::Rtf;
        }

        // Default to text if mostly ASCII
        if data.iter().take(1000).all(|&b| b.is_ascii()) {
            return FileType::Text;
        }

        FileType::Unknown
    }
}
