use super::{Document, DocumentElement, DocumentLoader, DocumentMetadata, FileType, Result, LoaderError};
use std::fs;
use std::path::Path;

/// PDF document loader using pdf-extract crate
pub struct PdfLoader;

impl PdfLoader {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract text from PDF using pdf-extract crate
    fn extract_text_from_pdf(&self, data: &[u8]) -> Result<String> {
        pdf_extract::extract_text_from_mem(data)
            .map_err(|e| LoaderError::ParseError(format!("PDF parsing error: {}", e)))
    }
    
    /// Extract text chunks from PDF content
    fn extract_pdf_elements(text: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();
        
        // Simple approach: split into paragraphs/chunks
        let paragraphs: Vec<&str> = text
            .split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() > 10) // Filter out very short chunks
            .collect();
            
        for (i, paragraph) in paragraphs.iter().enumerate() {
            elements.push(DocumentElement {
                element_type: "text_chunk".to_string(),
                text: paragraph.to_string(),
                metadata: Some(format!("chunk_{}", i)),
            });
        }
        
        // If no paragraphs found, try splitting by lines
        if elements.is_empty() {
            let lines: Vec<&str> = text
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && s.len() > 5)
                .collect();
                
            for (i, line) in lines.iter().enumerate() {
                elements.push(DocumentElement {
                    element_type: "text_chunk".to_string(),
                    text: line.to_string(),
                    metadata: Some(format!("line_{}", i)),
                });
            }
        }
        
        elements
    }
}

impl DocumentLoader for PdfLoader {
    fn load_from_path(&self, path: &Path) -> Result<Document> {
        let data = fs::read(path)?;
        let metadata = fs::metadata(path)?;
        
        let content = self.extract_text_from_pdf(&data)?;
        let elements = Self::extract_pdf_elements(&content);
        
        let doc_metadata = DocumentMetadata {
            filename: path.file_name().map(|s| s.to_string_lossy().to_string()),
            file_size: Some(metadata.len()),
            file_type: FileType::Pdf,
            page_count: None, // TODO: Extract page count from PDF
        };
        
        Ok(Document {
            content,
            metadata: doc_metadata,
            elements,
        })
    }
    
    fn load_from_bytes(&self, data: &[u8], filename: Option<&str>) -> Result<Document> {
        let content = self.extract_text_from_pdf(data)?;
        let elements = Self::extract_pdf_elements(&content);
        
        let doc_metadata = DocumentMetadata {
            filename: filename.map(|s| s.to_string()),
            file_size: Some(data.len() as u64),
            file_type: FileType::Pdf,
            page_count: None, // TODO: Extract page count from PDF
        };
        
        Ok(Document {
            content,
            metadata: doc_metadata,
            elements,
        })
    }
    
    fn supports_format(&self, file_type: &FileType) -> bool {
        matches!(file_type, FileType::Pdf)
    }
    
    fn primary_format(&self) -> FileType {
        FileType::Pdf
    }
}