use serde::{Deserialize, Serialize};

/// GGUF model metadata parsed from file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufMetadata {
    pub architecture: String,
    pub name: String,
    pub parameters: u64,
    pub context_length: u32,
    pub embedding_length: u32,
    pub head_count: u32,
    pub head_count_kv: u32,
    pub layer_count: u32,
    pub quantization_version: u32,
}

/// Curated model entry from the NexusAI catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub family: String,
    pub parameters: String,
    pub quantization: String,
    pub huggingface_repo: String,
    pub huggingface_file: String,
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub tq_compatible: bool,
    pub tq_recommended_bits: u8,
    pub recommended_context: u32,
    pub min_ram_gb: u32,
    pub tags: Vec<String>,
}

/// Returns the built-in model catalog.
/// Phase 1 ships with a curated list; Phase 3 adds dynamic HuggingFace search.
pub fn get_catalog() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry {
            id: "qwen3.5-35b-a3b-q4km".into(),
            name: "Qwen 3.5 35B-A3B (MoE)".into(),
            family: "Qwen".into(),
            parameters: "35B (3B active)".into(),
            quantization: "Q4_K_M".into(),
            huggingface_repo: "Qwen/Qwen3.5-35B-A3B-GGUF".into(),
            huggingface_file: "qwen3.5-35b-a3b-q4_k_m.gguf".into(),
            size_bytes: 19_500_000_000,
            sha256: None,
            tq_compatible: true,
            tq_recommended_bits: 3,
            recommended_context: 65536,
            min_ram_gb: 16,
            tags: vec!["moe".into(), "fast".into(), "coding".into()],
        },
        CatalogEntry {
            id: "gemma3-27b-q4km".into(),
            name: "Gemma 3 27B".into(),
            family: "Gemma".into(),
            parameters: "27B".into(),
            quantization: "Q4_K_M".into(),
            huggingface_repo: "google/gemma-3-27b-GGUF".into(),
            huggingface_file: "gemma-3-27b-q4_k_m.gguf".into(),
            size_bytes: 15_200_000_000,
            sha256: None,
            tq_compatible: true,
            tq_recommended_bits: 3,
            recommended_context: 65536,
            min_ram_gb: 16,
            tags: vec!["google".into(), "multimodal".into()],
        },
        CatalogEntry {
            id: "phi4-14b-q4km".into(),
            name: "Phi-4 14B".into(),
            family: "Phi".into(),
            parameters: "14B".into(),
            quantization: "Q4_K_M".into(),
            huggingface_repo: "microsoft/phi-4-GGUF".into(),
            huggingface_file: "phi-4-q4_k_m.gguf".into(),
            size_bytes: 8_100_000_000,
            sha256: None,
            tq_compatible: true,
            tq_recommended_bits: 3,
            recommended_context: 32768,
            min_ram_gb: 8,
            tags: vec!["microsoft".into(), "efficient".into(), "reasoning".into()],
        },
        CatalogEntry {
            id: "nomic-embed-text-v1.5".into(),
            name: "Nomic Embed Text v1.5".into(),
            family: "Nomic".into(),
            parameters: "137M".into(),
            quantization: "F16".into(),
            huggingface_repo: "nomic-ai/nomic-embed-text-v1.5-GGUF".into(),
            huggingface_file: "nomic-embed-text-v1.5.f16.gguf".into(),
            size_bytes: 274_000_000,
            sha256: None,
            tq_compatible: false,
            tq_recommended_bits: 0,
            recommended_context: 8192,
            min_ram_gb: 2,
            tags: vec!["embedding".into(), "rag".into()],
        },
    ]
}
