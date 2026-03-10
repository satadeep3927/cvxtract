# CVXtract - Resume/CV Document Loader

A Rust library for loading and extracting content from resumes/CVs in multiple formats, inspired by Python's UnstructuredLoader.

## Features

- **Multi-format support**: PDF, DOCX, HTML, TXT, and more
- **Automatic format detection**: Based on file extension and content analysis
- **Text chunking**: Breaks documents into logical text chunks/paragraphs
- **Unified interface**: Single API for all document types
- **Batch processing**: Load multiple documents from directories

## Quick Start

```rust
use cvxtract::UnstructuredLoader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = UnstructuredLoader::new();
    
    // Load a single document
    let document = loader.load("resume.pdf")?;
    
    println!("Content: {}", document.content);
    println!("File type: {:?}", document.metadata.file_type);
    
    // Access text chunks
    for element in document.elements {
        println!("{}: {}", element.element_type, element.text);
    }
    
    Ok(())
}
```

## Supported Formats

| Format | Status | Dependencies Required |
|--------|--------|-----------------------|
| TXT | ✅ Ready | None |
| HTML | ✅ Ready | None |
| PDF | ✅ Ready | `pdf-extract = "0.6"` (included) |
| DOCX | ✅ Ready | `zip + quick-xml` (included) |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cvxtract = "0.1.0"
# All major document formats supported by default:
# - PDF support (pdf-extract)
# - DOCX support (zip + quick-xml)  
# - HTML support (built-in)
# - Text support (built-in)
```

## Usage Examples

### Basic Document Loading

```rust
use cvxtract::UnstructuredLoader;

let loader = UnstructuredLoader::new();

// Load any supported format - format detected automatically
let doc = loader.load("resume.pdf")?;
println!("Extracted {} characters", doc.content.len());
```

### Batch Processing

```rust
// Load all documents from a directory
let documents = loader.load_directory("resumes/", true)?; // recursive=true

for doc in documents {
    println!("Processing: {:?}", doc.metadata.filename);
    
    // Access text chunks
    let chunks: Vec<_> = doc.elements
        .iter()
        .collect();
    
    println!("Found {} text chunks", chunks.len());
}
```

### Working with Document Elements

```rust
let doc = loader.load("resume.docx")?;

// Access different chunks
let text_chunks = doc.elements
    .iter()
    .filter(|e| e.element_type == "paragraph" || e.element_type == "text_chunk")
    .collect::<Vec<_>>();

println!("Text chunks: {}", text_chunks.len());

// Process each chunk
for chunk in text_chunks {
    println!("Chunk: {}", chunk.text);
}
```

### Format-Specific Loading

```rust
use cvxtract::FileType;

// Force a specific format (bypasses auto-detection)
let doc = loader.load_from_bytes(&pdf_data, FileType::Pdf, Some("resume.pdf"))?;
```

## Document Structure

The `Document` struct contains:

```rust
pub struct Document {
    pub content: String,           // Raw extracted text
    pub metadata: DocumentMetadata, // File info, size, type, etc.
    pub elements: Vec<DocumentElement>, // Text chunks/paragraphs
}

pub struct DocumentElement {
    pub element_type: String,      // "paragraph", "text_chunk", etc.
    pub text: String,             // Element content
    pub metadata: Option<String>, // Additional element info
}
```

## Common Element Types

- `"paragraph"` - Text paragraphs from plain text files
- `"text_chunk"` - Text chunks extracted from HTML/other formats
- Additional element types may be added by specific loaders based on document structure

## Extending Support

To add support for new formats:

1. Create a new loader implementing `DocumentLoader` trait
2. Register it in `UnstructuredLoader::new()`
3. Add format detection logic in `utils.rs`

```rust
use cvxtract::loaders::{DocumentLoader, Document, FileType, Result};

pub struct MyCustomLoader;

impl DocumentLoader for MyCustomLoader {
    fn load_from_path<P: AsRef<Path>>(&self, path: P) -> Result<Document> {
        // Your implementation here
    }
    
    fn load_from_bytes(&self, data: &[u8], filename: Option<&str>) -> Result<Document> {
        // Your implementation here
    }
    
    fn supports_format(&self, file_type: &FileType) -> bool {
        // Return true for your format
    }
    
    fn primary_format(&self) -> FileType {
        // Return your format type
    }
}
```

## Error Handling

```rust
use cvxtract::{UnstructuredLoader, LoaderError};

let loader = UnstructuredLoader::new();

match loader.load("resume.pdf") {
    Ok(doc) => println!("Success: {} chars", doc.content.len()),
    Err(LoaderError::UnsupportedFormat(format)) => {
        println!("Unsupported format: {}", format);
    },
    Err(LoaderError::IoError(err)) => {
        println!("File error: {}", err);
    },
    Err(LoaderError::ParseError(msg)) => {
        println!("Parse error: {}", msg);
    },
    Err(err) => println!("Other error: {}", err),
}
```

## Contributing

1. Fork the repository
2. Add support for new formats or improve existing ones
3. Write tests for your changes
4. Submit a pull request

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.