//! RAG (Retrieval-Augmented Generation) pipeline.
//!
//! Provides document ingestion, chunking, embedding, and TQ-compressed
//! vector search for context injection into LLM prompts.

pub mod chunker;
pub mod embedder;
pub mod index;
pub mod parsers;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A document that has been ingested into the RAG pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub doc_type: DocType,
    pub size_bytes: u64,
    pub last_modified: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocType {
    PlainText,
    Markdown,
    Code(String), // language
    Pdf,
    Docx,
    Html,
}

/// A chunk of a document with its embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub chunk_index: usize,
    pub embedding: Option<Vec<f32>>,
    pub tq_embedding: Option<Vec<u8>>, // TQ-compressed embedding
}

/// Search result from the vector index
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub chunk: Chunk,
    pub score: f32,
    pub document_path: String,
    pub document_title: String,
}

/// RAG pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_results: usize,
    pub min_relevance_score: f32,
    pub embedding_model: String,
    pub use_tq_compression: bool,
    pub tq_bits: u8,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 64,
            max_results: 5,
            min_relevance_score: 0.3,
            embedding_model: "nomic-embed-text-v1.5".into(),
            use_tq_compression: true,
            tq_bits: 3,
        }
    }
}

/// Main RAG pipeline
pub struct RagPipeline {
    config: RagConfig,
    index: index::VectorIndex,
    documents: Vec<Document>,
}

impl RagPipeline {
    pub fn new(config: RagConfig, index_path: &Path) -> Result<Self, String> {
        let index = index::VectorIndex::new(index_path, config.use_tq_compression, config.tq_bits)?;
        Ok(Self {
            config,
            index,
            documents: Vec::new(),
        })
    }

    /// Ingest a file into the RAG pipeline
    pub async fn ingest_file(&mut self, path: &Path) -> Result<usize, String> {
        // Parse the file
        let doc = parsers::parse_file(path)?;
        // Chunk the document
        let chunks = chunker::chunk_document(
            &doc,
            self.config.chunk_size,
            self.config.chunk_overlap,
        );

        let chunk_count = chunks.len();
        tracing::info!(
            "Ingested {} → {} chunks ({}b each)",
            path.display(),
            chunk_count,
            self.config.chunk_size
        );

        // TODO: Embed chunks using local embedding model
        // For now, store chunks without embeddings
        for chunk in &chunks {
            self.index.add_chunk(chunk)?;
        }

        self.documents.push(doc);
        Ok(chunk_count)
    }

    /// Ingest all files in a directory
    pub async fn ingest_directory(&mut self, dir: &Path) -> Result<(usize, usize), String> {
        let mut total_files = 0;
        let mut total_chunks = 0;

        let entries = walkdir(dir)?;
        for path in entries {
            match self.ingest_file(&path).await {
                Ok(chunks) => {
                    total_files += 1;
                    total_chunks += chunks;
                }
                Err(e) => {
                    tracing::warn!("Skipping {}: {}", path.display(), e);
                }
            }
        }

        Ok((total_files, total_chunks))
    }

    /// Search for relevant chunks given a query
    pub fn search(&self, query: &str, max_results: Option<usize>) -> Vec<SearchResult> {
        let k = max_results.unwrap_or(self.config.max_results);
        // TODO: Embed query, then search index
        self.index.search_text(query, k)
    }

    /// Build a context string from search results for prompt injection
    pub fn build_context(&self, results: &[SearchResult]) -> String {
        if results.is_empty() {
            return String::new();
        }

        let mut context = String::from("## Relevant Context\n\n");
        for (i, result) in results.iter().enumerate() {
            context.push_str(&format!(
                "### Source {}: {} (score: {:.2})\n```\n{}\n```\n\n",
                i + 1,
                result.document_title,
                result.score,
                result.chunk.content
            ));
        }
        context
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn chunk_count(&self) -> usize {
        self.index.len()
    }
}

/// Walk a directory and return all supported file paths
fn walkdir(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories and common non-content dirs
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                continue;
            }
            files.extend(walkdir(&path)?);
        } else if parsers::is_supported_file(&path) {
            files.push(path);
        }
    }

    Ok(files)
}
