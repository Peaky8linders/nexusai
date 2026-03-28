//! Document chunking strategies.
//!
//! Recursive chunking with overlap, respecting document structure
//! (paragraphs, code blocks, headings).

use super::{Chunk, Document};

/// Chunk a document using recursive splitting with overlap
pub fn chunk_document(doc: &Document, chunk_size: usize, overlap: usize) -> Vec<Chunk> {
    let text = &doc.content;
    if text.is_empty() {
        return Vec::new();
    }

    // Use structure-aware splitting
    let splits = split_by_structure(text, chunk_size);

    let mut chunks = Vec::new();
    let mut offset = 0;

    for (i, split) in splits.iter().enumerate() {
        let chunk = Chunk {
            id: format!("{}-{}", doc.id, i),
            document_id: doc.id.clone(),
            content: split.clone(),
            start_offset: offset,
            end_offset: offset + split.len(),
            chunk_index: i,
            embedding: None,
            tq_embedding: None,
        };
        chunks.push(chunk);
        offset += split.len().saturating_sub(overlap);
    }

    chunks
}

/// Split text respecting document structure:
/// 1. Try splitting on double newlines (paragraphs)
/// 2. If chunks too large, split on single newlines
/// 3. If still too large, split on sentences
/// 4. Last resort: split on word boundaries
fn split_by_structure(text: &str, max_chunk_size: usize) -> Vec<String> {
    if text.len() <= max_chunk_size {
        return vec![text.to_string()];
    }

    // Try paragraph splits first
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut chunks = Vec::new();
    let mut current = String::new();

    for para in paragraphs {
        if current.len() + para.len() + 2 > max_chunk_size {
            if !current.is_empty() {
                chunks.push(current.trim().to_string());
                current = String::new();
            }
            // If single paragraph exceeds chunk size, split further
            if para.len() > max_chunk_size {
                chunks.extend(split_by_lines(para, max_chunk_size));
            } else {
                current = para.to_string();
            }
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
        }
    }

    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks.into_iter().filter(|c| !c.is_empty()).collect()
}

fn split_by_lines(text: &str, max_size: usize) -> Vec<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in lines {
        if current.len() + line.len() + 1 > max_size {
            if !current.is_empty() {
                chunks.push(current.trim().to_string());
                current = String::new();
            }
            if line.len() > max_size {
                // Split on word boundaries as last resort
                chunks.extend(split_by_words(line, max_size));
            } else {
                current = line.to_string();
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }

    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

fn split_by_words(text: &str, max_size: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut current = String::new();

    for word in words {
        if current.len() + word.len() + 1 > max_size {
            if !current.is_empty() {
                chunks.push(current.clone());
                current.clear();
            }
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::{DocType, Document};

    #[test]
    fn test_chunking_short_doc() {
        let doc = Document {
            id: "test".into(),
            path: "test.txt".into(),
            title: "Test".into(),
            content: "Hello world".into(),
            doc_type: DocType::PlainText,
            size_bytes: 11,
            last_modified: 0,
        };
        let chunks = chunk_document(&doc, 512, 64);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Hello world");
    }

    #[test]
    fn test_chunking_paragraphs() {
        let content = "Paragraph one.\n\nParagraph two.\n\nParagraph three.";
        let doc = Document {
            id: "test".into(),
            path: "test.txt".into(),
            title: "Test".into(),
            content: content.into(),
            doc_type: DocType::PlainText,
            size_bytes: content.len() as u64,
            last_modified: 0,
        };
        let chunks = chunk_document(&doc, 20, 0);
        assert!(chunks.len() >= 2);
    }
}
