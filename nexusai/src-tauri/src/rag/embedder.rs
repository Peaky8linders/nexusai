//! Embedding generation using a local model (nomic-embed-text).
//!
//! Uses the same llama.cpp backend to run the embedding model.

use super::Chunk;

/// Embedding model configuration
pub struct EmbedderConfig {
    pub model_path: String,
    pub embedding_dim: usize, // 768 for nomic-embed-text-v1.5
    pub batch_size: usize,
    pub normalize: bool,
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            embedding_dim: 768,
            batch_size: 32,
            normalize: true,
        }
    }
}

/// Local embedding model
pub struct Embedder {
    config: EmbedderConfig,
    is_loaded: bool,
}

impl Embedder {
    pub fn new(config: EmbedderConfig) -> Self {
        Self {
            config,
            is_loaded: false,
        }
    }

    /// Load the embedding model
    pub fn load(&mut self) -> Result<(), String> {
        if self.config.model_path.is_empty() {
            return Err("No embedding model path configured".into());
        }

        // TODO: Load via llama.cpp with embedding mode
        // unsafe {
        //     let params = llama_model_default_params();
        //     params.n_gpu_layers = -1;
        //     let model = llama_load_model_from_file(path, params);
        //
        //     let ctx_params = llama_context_default_params();
        //     ctx_params.n_ctx = 8192;
        //     ctx_params.embeddings = true; // Enable embedding mode
        //     let ctx = llama_new_context_with_model(model, ctx_params);
        // }

        self.is_loaded = true;
        Ok(())
    }

    /// Generate embedding for a single text
    pub fn embed_text(&self, _text: &str) -> Result<Vec<f32>, String> {
        if !self.is_loaded {
            return Err("Embedding model not loaded".into());
        }

        // TODO: Replace with actual embedding generation
        // 1. Tokenize with "search_document: " prefix (nomic convention)
        // 2. Run through model in embedding mode
        // 3. Extract embedding from llama_get_embeddings()
        // 4. L2 normalize

        // Stub: return zero vector
        Ok(vec![0.0f32; self.config.embedding_dim])
    }

    /// Batch embed multiple chunks
    pub fn embed_chunks(&self, chunks: &mut [Chunk]) -> Result<(), String> {
        if !self.is_loaded {
            return Err("Embedding model not loaded".into());
        }

        for chunk in chunks.iter_mut() {
            let embedding = self.embed_text(&chunk.content)?;
            chunk.embedding = Some(embedding);
        }

        Ok(())
    }

    /// Embed a query (uses "search_query: " prefix for nomic models)
    pub fn embed_query(&self, query: &str) -> Result<Vec<f32>, String> {
        let prefixed = format!("search_query: {}", query);
        self.embed_text(&prefixed)
    }

    pub fn dim(&self) -> usize {
        self.config.embedding_dim
    }
}
