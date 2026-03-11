use super::loaders::{
    docx::DocxLoader, html::HtmlLoader, pdf::PdfLoader, text::TextLoader, utils::detect_file_type,
    utils::detect_from_bytes, Document, DocumentLoader, FileType, LoaderError, Result,
};
use std::collections::HashMap;
use std::path::Path;

/// Main UnstructuredLoader that automatically detects file format and delegates to appropriate loader
pub struct UnstructuredLoader {
    loaders: HashMap<FileType, Box<dyn DocumentLoader>>,
}

impl UnstructuredLoader {
    /// Create a new UnstructuredLoader with all supported format loaders
    pub fn new() -> Self {
        let mut loaders: HashMap<FileType, Box<dyn DocumentLoader>> = HashMap::new();

        // Register all available loaders
        loaders.insert(FileType::Text, Box::new(TextLoader::new()));
        loaders.insert(FileType::Html, Box::new(HtmlLoader::new()));
        loaders.insert(FileType::Pdf, Box::new(PdfLoader::new()));
        loaders.insert(FileType::Docx, Box::new(DocxLoader::new()));

        Self { loaders }
    }

    /// Load a document from a file path with automatic format detection
    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<Document> {
        let path = path.as_ref();

        // First try to detect format from file extension
        let detected_type = detect_file_type(path);

        if detected_type != FileType::Unknown {
            if let Some(loader) = self.loaders.get(&detected_type) {
                return loader.load_from_path(path);
            }
        }

        // If extension-based detection failed, try reading file and detecting from content
        let data = std::fs::read(path)?;
        let content_type = detect_from_bytes(&data);

        if let Some(loader) = self.loaders.get(&content_type) {
            loader.load_from_bytes(
                &data,
                path.file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .as_deref(),
            )
        } else {
            Err(LoaderError::UnsupportedFormat(format!(
                "Unsupported file format: {:?}",
                content_type
            )))
        }
    }

    /// Load a document from raw bytes with manual format specification
    pub fn load_from_bytes(
        &self,
        data: &[u8],
        format: FileType,
        filename: Option<&str>,
    ) -> Result<Document> {
        if let Some(loader) = self.loaders.get(&format) {
            loader.load_from_bytes(data, filename)
        } else {
            Err(LoaderError::UnsupportedFormat(format!(
                "No loader available for format: {:?}",
                format
            )))
        }
    }

    /// Load a document from raw bytes with automatic format detection
    pub fn load_from_bytes_auto(&self, data: &[u8], filename: Option<&str>) -> Result<Document> {
        let detected_type = detect_from_bytes(data);
        self.load_from_bytes(data, detected_type, filename)
    }

    /// Get list of supported file types
    pub fn supported_formats(&self) -> Vec<FileType> {
        self.loaders.keys().cloned().collect()
    }

    /// Check if a specific format is supported
    pub fn supports_format(&self, format: &FileType) -> bool {
        self.loaders.contains_key(format)
    }

    /// Load multiple documents from a directory
    pub fn load_directory<P: AsRef<Path>>(
        &self,
        dir_path: P,
        recursive: bool,
    ) -> Result<Vec<Document>> {
        let dir_path = dir_path.as_ref();
        let mut documents = Vec::new();

        if !dir_path.is_dir() {
            return Err(LoaderError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Path is not a directory",
            )));
        }

        let entries = if recursive {
            self.walk_directory_recursive(dir_path)?
        } else {
            std::fs::read_dir(dir_path)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .map(|entry| entry.path())
                .collect()
        };

        for file_path in entries {
            match self.load(&file_path) {
                Ok(document) => documents.push(document),
                Err(LoaderError::UnsupportedFormat(_)) => {
                    // Skip unsupported files silently
                    continue;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load {}: {}", file_path.display(), e);
                    continue;
                }
            }
        }

        Ok(documents)
    }

    /// Recursively walk directory to find all files
    fn walk_directory_recursive<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        fn walk_dir(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    walk_dir(&path, files)?;
                } else {
                    files.push(path);
                }
            }
            Ok(())
        }

        walk_dir(dir.as_ref(), &mut files)?;
        Ok(files)
    }
}

impl Default for UnstructuredLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience functions for common use cases
impl UnstructuredLoader {
    /// Quick load function for single file - returns just the text content
    pub fn extract_text<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let document = self.load(path)?;
        Ok(document.content)
    }

    /// Extract all documents from directory and return just text content
    pub fn extract_all_text<P: AsRef<Path>>(
        &self,
        dir_path: P,
        recursive: bool,
    ) -> Result<Vec<(String, String)>> {
        let documents = self.load_directory(dir_path, recursive)?;
        Ok(documents
            .into_iter()
            .map(|doc| {
                let filename = doc
                    .metadata
                    .filename
                    .unwrap_or_else(|| "unknown".to_string());
                (filename, doc.content)
            })
            .collect())
    }
}
