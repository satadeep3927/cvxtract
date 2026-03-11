use super::{Document, DocumentElement, DocumentLoader, DocumentMetadata, FileType, Result};
use std::fs;
use std::path::Path;

/// HTML document loader
pub struct HtmlLoader;

impl HtmlLoader {
    pub fn new() -> Self {
        Self
    }

    /// Simple HTML tag stripper (basic implementation)
    fn strip_html_tags(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_script_or_style = false;
        let mut tag_name = String::new();

        let chars: Vec<char> = html.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '<' {
                in_tag = true;
                tag_name.clear();

                // Check if this is a script or style tag
                if i + 6 < chars.len() {
                    let next_chars: String = chars[i + 1..i + 7].iter().collect();
                    if next_chars.to_lowercase() == "script"
                        || next_chars.to_lowercase().starts_with("style")
                    {
                        in_script_or_style = true;
                    }
                }
            } else if ch == '>' && in_tag {
                in_tag = false;

                // Check if this is a closing script or style tag
                if tag_name.to_lowercase() == "/script" || tag_name.to_lowercase() == "/style" {
                    in_script_or_style = false;
                }
            } else if in_tag {
                tag_name.push(ch);
            } else if !in_script_or_style {
                // Only add content if we're not inside script or style tags
                result.push(ch);
            }

            i += 1;
        }

        // Clean up whitespace
        result
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Extract basic document structure from HTML
    fn extract_elements(html: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();

        // Simple approach: extract text and split into chunks
        let content = Self::strip_html_tags(html);

        // Split into paragraphs/chunks for processing
        let paragraphs: Vec<&str> = content
            .split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() > 10) // Filter out very short chunks
            .collect();

        for (i, paragraph) in paragraphs.iter().enumerate() {
            elements.push(DocumentElement {
                element_type: "text_chunk".to_string(),
                text: paragraph.to_string(),
                metadata: Some(format!("chunk_{i}")),
            });
        }

        elements
    }
}

impl DocumentLoader for HtmlLoader {
    fn load_from_path(&self, path: &Path) -> Result<Document> {
        let html_content = fs::read_to_string(path)?;
        let metadata = fs::metadata(path)?;

        let content = Self::strip_html_tags(&html_content);
        let elements = Self::extract_elements(&html_content);

        let doc_metadata = DocumentMetadata {
            filename: path.file_name().map(|s| s.to_string_lossy().to_string()),
            file_size: Some(metadata.len()),
            file_type: FileType::Html,
            page_count: None,
        };

        Ok(Document {
            content,
            metadata: doc_metadata,
            elements,
        })
    }

    fn load_from_bytes(&self, data: &[u8], filename: Option<&str>) -> Result<Document> {
        let html_content = String::from_utf8_lossy(data).to_string();
        let content = Self::strip_html_tags(&html_content);
        let elements = Self::extract_elements(&html_content);

        let doc_metadata = DocumentMetadata {
            filename: filename.map(|s| s.to_string()),
            file_size: Some(data.len() as u64),
            file_type: FileType::Html,
            page_count: None,
        };

        Ok(Document {
            content,
            metadata: doc_metadata,
            elements,
        })
    }
}
