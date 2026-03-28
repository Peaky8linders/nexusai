//! File parsers for document ingestion.
//! Supports: plain text, markdown, code files, and stubs for PDF/DOCX.

use super::{DocType, Document};
use std::path::Path;

/// Parse a file into a Document
pub fn parse_file(path: &Path) -> Result<Document, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let content = match ext.as_str() {
        "pdf" => parse_pdf(path)?,
        "docx" => parse_docx(path)?,
        _ => std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?,
    };

    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let doc_type = detect_doc_type(path);
    let title = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Ok(Document {
        id: uuid::Uuid::new_v4().to_string(),
        path: path.to_path_buf(),
        title,
        content,
        doc_type,
        size_bytes: metadata.len(),
        last_modified: metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    })
}

/// Check if a file type is supported for ingestion
pub fn is_supported_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    matches!(
        ext.as_str(),
        "txt" | "md" | "markdown"
            | "rs" | "py" | "js" | "ts" | "tsx" | "jsx"
            | "go" | "java" | "c" | "cpp" | "h" | "hpp"
            | "rb" | "php" | "swift" | "kt" | "scala"
            | "css" | "scss" | "html" | "xml" | "json"
            | "yaml" | "yml" | "toml" | "ini" | "cfg"
            | "sh" | "bash" | "zsh" | "fish"
            | "sql" | "graphql"
            | "pdf" | "docx"
            | "csv" | "tsv"
            | "log"
    )
}

fn detect_doc_type(path: &Path) -> DocType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "md" | "markdown" => DocType::Markdown,
        "pdf" => DocType::Pdf,
        "docx" => DocType::Docx,
        "html" | "htm" => DocType::Html,
        "txt" | "log" | "csv" | "tsv" | "json" | "yaml" | "yml" | "toml" | "xml" | "ini"
        | "cfg" => DocType::PlainText,
        lang => DocType::Code(lang.to_string()),
    }
}

fn parse_pdf(_path: &Path) -> Result<String, String> {
    // Phase 3: integrate a PDF parsing library (pdf-extract or lopdf)
    Err("PDF parsing not yet implemented — install pdf-extract crate".into())
}

fn parse_docx(_path: &Path) -> Result<String, String> {
    // Phase 3: integrate a DOCX parser (docx-rs)
    Err("DOCX parsing not yet implemented — install docx-rs crate".into())
}
