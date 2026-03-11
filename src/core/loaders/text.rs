use super::{Document, DocumentElement, DocumentLoader, DocumentMetadata, FileType, Result};
use std::fs;
use std::path::Path;

/// Simple text file loader
pub struct TextLoader;

impl TextLoader {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentLoader for TextLoader {
    fn load_from_path(&self, path: &Path) -> Result<Document> {
        let content = fs::read_to_string(path)?;
        let metadata = fs::metadata(path)?;

        let doc_metadata = DocumentMetadata {
            filename: path.file_name().map(|s| s.to_string_lossy().to_string()),
            file_size: Some(metadata.len()),
            file_type: FileType::Text,
            page_count: None, // Text files don't have pages
        };

        // Basic text processing - split into paragraphs as chunks
        let elements = content
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .enumerate()
            .map(|(i, paragraph)| DocumentElement {
                element_type: "paragraph".to_string(),
                text: paragraph.trim().to_string(),
                metadata: Some(format!("paragraph_{}", i)),
            })
            .collect();

        Ok(Document {
            content,
            metadata: doc_metadata,
            elements,
        })
    }

    fn load_from_bytes(&self, data: &[u8], filename: Option<&str>) -> Result<Document> {
        let content = String::from_utf8_lossy(data).to_string();

        let doc_metadata = DocumentMetadata {
            filename: filename.map(|s| s.to_string()),
            file_size: Some(data.len() as u64),
            file_type: FileType::Text,
            page_count: None,
        };

        // Basic text processing - split into paragraphs as chunks
        let elements = content
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .enumerate()
            .map(|(i, paragraph)| DocumentElement {
                element_type: "paragraph".to_string(),
                text: paragraph.trim().to_string(),
                metadata: Some(format!("paragraph_{}", i)),
            })
            .collect();

        Ok(Document {
            content,
            metadata: doc_metadata,
            elements,
        })
    }

    fn supports_format(&self, file_type: &FileType) -> bool {
        matches!(file_type, FileType::Text)
    }

    fn primary_format(&self) -> FileType {
        FileType::Text
    }
}
