use super::{
    Document, DocumentElement, DocumentLoader, DocumentMetadata, FileType, LoaderError, Result,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub struct DocxLoader;

impl DocxLoader {
    pub fn new() -> Self {
        Self
    }

    /// Extract text from DOCX - super simple approach
    fn extract_docx_text(data: &[u8]) -> Result<String> {
        let cursor = std::io::Cursor::new(data);
        let mut zip = ZipArchive::new(cursor)
            .map_err(|e| LoaderError::ParseError(format!("Invalid DOCX: {e}")))?;

        // Read main document
        let mut document = zip
            .by_name("word/document.xml")
            .map_err(|e| LoaderError::ParseError(format!("No document.xml: {e}")))?;

        let mut xml_content = String::new();
        document
            .read_to_string(&mut xml_content)
            .map_err(|e| LoaderError::ParseError(format!("Read error: {e}")))?;

        // Extract text from XML
        let mut reader = Reader::from_str(&xml_content);
        let mut text = String::new();
        let mut inside_text = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                    inside_text = true;
                }
                Ok(Event::Text(e)) if inside_text => {
                    if let Ok(decoded) = e.unescape() {
                        text.push_str(&decoded);
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" => {
                    inside_text = false;
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                    text.push('\n'); // Paragraph break
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(LoaderError::ParseError(format!("XML error: {e}"))),
                _ => {}
            }
        }

        Ok(text)
    }
}

impl DocumentLoader for DocxLoader {
    fn load_from_path(&self, path: &Path) -> Result<Document> {
        let data = fs::read(path)?;
        let filename = path.file_name().map(|s| s.to_string_lossy().to_string());
        self.load_from_bytes(&data, filename.as_deref())
    }

    fn load_from_bytes(&self, data: &[u8], filename: Option<&str>) -> Result<Document> {
        let content = Self::extract_docx_text(data)?;

        let doc_metadata = DocumentMetadata {
            filename: filename.map(|s| s.to_string()),
            file_size: Some(data.len() as u64),
            file_type: FileType::Docx,
            page_count: None,
        };

        // Simple chunking by paragraphs
        let elements: Vec<DocumentElement> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|text| DocumentElement {
                element_type: "text_chunk".to_string(),
                text: text.trim().to_string(),
                metadata: None,
            })
            .collect();

        Ok(Document {
            content,
            metadata: doc_metadata,
            elements,
        })
    }
}
