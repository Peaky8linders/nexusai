pub mod download;
pub mod llama_ffi;
pub mod tq_config;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tq_config::TurboQuantConfig;

/// Wraps the inference engine in Tauri managed state
pub struct EngineState(pub Mutex<InferenceEngine>);

/// Core inference engine that manages model loading and text generation.
///
/// In Phase 1, this is a stub that returns placeholder text.
/// It will be replaced with llama.cpp FFI bindings once the C library is built.
pub struct InferenceEngine {
    model_loaded: bool,
    model_id: Option<String>,
    tq_config: TurboQuantConfig,
    stop_flag: AtomicBool,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            model_loaded: false,
            model_id: None,
            tq_config: TurboQuantConfig::default(),
            stop_flag: AtomicBool::new(false),
        }
    }

    pub fn load_model(&mut self, model_id: &str, tq_bits: u8) -> Result<(), String> {
        tracing::info!(
            "Loading model {} with TurboQuant TQ{} KV cache",
            model_id,
            tq_bits
        );

        self.tq_config = TurboQuantConfig::new(tq_bits);
        self.model_id = Some(model_id.to_string());
        self.model_loaded = true;

        // TODO: Replace with actual llama.cpp model loading:
        // 1. llama_model_load(path, params)
        // 2. llama_context_new(model, ctx_params) with TQ KV cache type
        // 3. Set cache_type_k = GGML_TYPE_TQ3_0 / TQ4_0

        Ok(())
    }

    pub fn unload(&mut self) {
        tracing::info!("Unloading model");
        self.model_loaded = false;
        self.model_id = None;
        // TODO: llama_free(ctx), llama_model_free(model)
    }

    pub fn generate(&self, prompt: &str) -> Result<String, String> {
        if !self.model_loaded {
            return Err("No model loaded. Select and load a model first.".into());
        }

        self.stop_flag.store(false, Ordering::Relaxed);

        // Stub response — will be replaced with actual llama.cpp inference
        let model_name = self.model_id.as_deref().unwrap_or("unknown");
        Ok(format!(
            "**[NexusAI Stub Response]**\n\n\
             Model: `{}`\n\
             TurboQuant: TQ{} KV cache ({:.1}x compression)\n\
             Context: {} tokens\n\n\
             Your prompt: \"{}\"\n\n\
             ---\n\n\
             This is a placeholder response. Once the llama.cpp fork with TurboQuant \
             support is compiled and linked, this will produce real inference output.\n\n\
             **Next steps:**\n\
             1. Install Rust toolchain: `rustup-init`\n\
             2. Fork llama.cpp and apply TQ patches\n\
             3. Build with `cargo tauri dev`\n",
            model_name,
            self.tq_config.bits,
            self.tq_config.compression_ratio(),
            self.tq_config.max_context,
            &prompt[..(0..=prompt.len().min(100)).rev().find(|&i| prompt.is_char_boundary(i)).unwrap_or(0)]
        ))
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_loaded(&self) -> bool {
        self.model_loaded
    }
}
