//! TQ-compressed HNSW vector index for RAG retrieval.
//!
//! Uses TurboQuant to compress embedding vectors from 768 × 4 bytes (3KB)
//! to 768 × 3 bits (288 bytes) — a 10.6x reduction. This means
//! indexing 1M chunks takes ~275MB instead of ~2.9GB.

use super::{Chunk, SearchResult};
use std::path::{Path, PathBuf};

/// HNSW vector index with optional TQ compression
pub struct VectorIndex {
    chunks: Vec<Chunk>,
    #[allow(dead_code)]
    path: PathBuf,
    use_tq: bool,
    tq_bits: u8,
}

impl VectorIndex {
    pub fn new(path: &Path, use_tq: bool, tq_bits: u8) -> Result<Self, String> {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
        Ok(Self {
            chunks: Vec::new(),
            path: path.to_path_buf(),
            use_tq,
            tq_bits,
        })
    }

    /// Add a chunk to the index
    pub fn add_chunk(&mut self, chunk: &Chunk) -> Result<(), String> {
        // TODO: If use_tq, compress the embedding before storing:
        // 1. Apply WHT rotation to embedding
        // 2. Scalar quantize each coordinate using pre-computed codebook
        // 3. Pack into tq_embedding field
        // 4. Add to HNSW graph

        self.chunks.push(chunk.clone());
        Ok(())
    }

    /// Search the index with a text query (temporary: uses keyword matching)
    /// Will be replaced with proper vector search once embeddings work.
    pub fn search_text(&self, query: &str, k: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(usize, f32)> = self
            .chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| {
                let content_lower = chunk.content.to_lowercase();
                let score = query_terms
                    .iter()
                    .filter(|term| content_lower.contains(*term))
                    .count() as f32
                    / query_terms.len().max(1) as f32;
                (i, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);

        scored
            .into_iter()
            .map(|(i, score)| {
                let chunk = &self.chunks[i];
                SearchResult {
                    chunk: chunk.clone(),
                    score,
                    document_path: chunk.document_id.clone(),
                    document_title: chunk.document_id.clone(),
                }
            })
            .collect()
    }

    /// Search with a vector query (proper cosine similarity)
    pub fn search_vector(&self, query_embedding: &[f32], k: usize) -> Vec<SearchResult> {
        // TODO: Implement HNSW search with TQ-compressed distance computation
        // For TQ-compressed vectors:
        // 1. Quantize query using same codebook
        // 2. Use tq_inner_product() for distance computation
        // 3. QJL residual correction for unbiased estimation

        // Stub: brute-force cosine similarity
        let mut scored: Vec<(usize, f32)> = self
            .chunks
            .iter()
            .enumerate()
            .filter_map(|(i, chunk)| {
                chunk.embedding.as_ref().map(|emb| {
                    let score = cosine_similarity(query_embedding, emb);
                    (i, score)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);

        scored
            .into_iter()
            .map(|(i, score)| {
                let chunk = &self.chunks[i];
                SearchResult {
                    chunk: chunk.clone(),
                    score,
                    document_path: chunk.document_id.clone(),
                    document_title: chunk.document_id.clone(),
                }
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Estimated memory usage for the index
    pub fn memory_usage_bytes(&self) -> usize {
        let chunk_overhead: usize = self
            .chunks
            .iter()
            .map(|c| {
                c.content.len()
                    + c.embedding
                        .as_ref()
                        .map(|e| e.len() * 4)
                        .unwrap_or(0)
                    + c.tq_embedding
                        .as_ref()
                        .map(|e| e.len())
                        .unwrap_or(0)
            })
            .sum();

        chunk_overhead
    }

    /// Estimated memory savings from TQ compression
    pub fn compression_stats(&self) -> (usize, usize) {
        let fp32_size: usize = self
            .chunks
            .iter()
            .filter_map(|c| c.embedding.as_ref())
            .map(|e| e.len() * 4) // 4 bytes per f32
            .sum();

        let tq_size = if self.use_tq {
            let bits = self.tq_bits as usize;
            self.chunks
                .iter()
                .filter_map(|c| c.embedding.as_ref())
                .map(|e| (e.len() * bits + 7) / 8) // bits to bytes, rounded up
                .sum()
        } else {
            fp32_size
        };

        (fp32_size, tq_size)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}
