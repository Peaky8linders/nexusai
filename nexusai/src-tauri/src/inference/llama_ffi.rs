//! FFI bindings to llama.cpp with TurboQuant KV cache support.
//!
//! This module provides safe Rust wrappers around the llama.cpp C API.
//! When the llama.cpp fork is compiled, these will link to the actual functions.
//! Until then, the module compiles with stub implementations behind cfg flags.

// FFI types — unused until llama.cpp is linked, kept for documentation
#[allow(unused_imports)]
use std::ffi::{CStr, CString};
#[allow(unused_imports)]
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Re-export types
pub type LlamaModel = *mut c_void;
pub type LlamaContext = *mut c_void;
pub type LlamaToken = i32;

/// KV cache quantization type — maps to GGML types in llama.cpp
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KvCacheType {
    F16 = 0,
    Q8_0 = 8,
    Q4_0 = 2,
    Turbo3 = 30, // GGML_TYPE_TQ3_0
    Turbo4 = 31, // GGML_TYPE_TQ4_0
}

impl KvCacheType {
    pub fn from_tq_bits(bits: u8) -> Self {
        match bits {
            3 => KvCacheType::Turbo3,
            4 => KvCacheType::Turbo4,
            8 => KvCacheType::Q8_0,
            16 => KvCacheType::F16,
            _ => KvCacheType::Turbo3,
        }
    }
}

/// Model parameters for loading
#[derive(Debug, Clone)]
pub struct ModelParams {
    pub n_gpu_layers: i32,
    pub use_mmap: bool,
    pub use_mlock: bool,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            n_gpu_layers: -1, // offload all layers
            use_mmap: true,
            use_mlock: false,
        }
    }
}

/// Context parameters for inference
#[derive(Debug, Clone)]
pub struct ContextParams {
    pub n_ctx: u32,
    pub n_batch: u32,
    pub n_threads: u32,
    pub cache_type_k: KvCacheType,
    pub cache_type_v: KvCacheType,
    pub flash_attn: bool,
}

impl Default for ContextParams {
    fn default() -> Self {
        Self {
            n_ctx: 65536,
            n_batch: 512,
            n_threads: 4,
            cache_type_k: KvCacheType::Turbo3,
            cache_type_v: KvCacheType::Turbo3,
            flash_attn: true,
        }
    }
}

/// Sampling parameters
#[derive(Debug, Clone)]
pub struct SamplingParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub max_tokens: u32,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            max_tokens: 4096,
        }
    }
}

/// Token generation statistics
#[derive(Debug, Clone, Default)]
pub struct GenerationStats {
    pub tokens_generated: u32,
    pub prompt_tokens: u32,
    pub tokens_per_second: f64,
    pub time_to_first_token_ms: f64,
    pub total_time_ms: f64,
    pub kv_cache_used_bytes: u64,
    pub kv_cache_total_bytes: u64,
}

/// Safe wrapper around a loaded llama.cpp model + context
pub struct LlamaSession {
    model_path: String,
    ctx_params: ContextParams,
    sampling_params: SamplingParams,
    stop_flag: Arc<AtomicBool>,
    is_loaded: bool,
}

impl LlamaSession {
    pub fn new() -> Self {
        Self {
            model_path: String::new(),
            ctx_params: ContextParams::default(),
            sampling_params: SamplingParams::default(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            is_loaded: false,
        }
    }

    /// Load a GGUF model from disk
    pub fn load_model(
        &mut self,
        path: &Path,
        model_params: &ModelParams,
        ctx_params: &ContextParams,
    ) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Model file not found: {}", path.display()));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext != "gguf" {
            return Err(format!("Expected .gguf file, got .{}", ext));
        }

        self.model_path = path.to_string_lossy().to_string();
        self.ctx_params = ctx_params.clone();
        self.is_loaded = true;

        tracing::info!(
            "Model loaded: {} (ctx={}, kv_type={:?})",
            path.file_name().unwrap_or_default().to_string_lossy(),
            ctx_params.n_ctx,
            ctx_params.cache_type_k
        );

        // TODO: Replace with actual llama.cpp calls:
        //
        // unsafe {
        //     llama_backend_init();
        //
        //     let mparams = llama_model_default_params();
        //     mparams.n_gpu_layers = model_params.n_gpu_layers;
        //     mparams.use_mmap = model_params.use_mmap;
        //
        //     let model_cstr = CString::new(path.to_str().unwrap()).unwrap();
        //     self.model = llama_load_model_from_file(model_cstr.as_ptr(), mparams);
        //     if self.model.is_null() {
        //         return Err("Failed to load model".into());
        //     }
        //
        //     let cparams = llama_context_default_params();
        //     cparams.n_ctx = ctx_params.n_ctx;
        //     cparams.n_batch = ctx_params.n_batch;
        //     cparams.n_threads = ctx_params.n_threads;
        //     cparams.type_k = ctx_params.cache_type_k as i32; // GGML_TYPE_TQ3_0
        //     cparams.type_v = ctx_params.cache_type_v as i32;
        //     cparams.flash_attn = ctx_params.flash_attn;
        //
        //     self.ctx = llama_new_context_with_model(self.model, cparams);
        //     if self.ctx.is_null() {
        //         llama_free_model(self.model);
        //         return Err("Failed to create context".into());
        //     }
        // }

        Ok(())
    }

    /// Generate tokens from a prompt, calling the callback for each token
    pub fn generate<F>(
        &self,
        messages: &[(String, String)], // (role, content) pairs
        sampling: &SamplingParams,
        mut on_token: F,
    ) -> Result<GenerationStats, String>
    where
        F: FnMut(&str, bool) -> bool, // (token_text, is_done) -> should_continue
    {
        if !self.is_loaded {
            return Err("No model loaded".into());
        }

        self.stop_flag.store(false, Ordering::Relaxed);
        let start = std::time::Instant::now();

        // Build the prompt from messages using ChatML format
        let prompt = build_chatml_prompt(messages);

        // TODO: Replace with actual llama.cpp inference:
        //
        // unsafe {
        //     // Tokenize
        //     let prompt_cstr = CString::new(prompt.as_str()).unwrap();
        //     let mut tokens = vec![0i32; prompt.len() + 256];
        //     let n_tokens = llama_tokenize(
        //         self.model, prompt_cstr.as_ptr(),
        //         prompt.len() as i32, tokens.as_mut_ptr(),
        //         tokens.len() as i32, true, false
        //     );
        //     tokens.truncate(n_tokens as usize);
        //
        //     // Create batch and decode prompt
        //     let batch = llama_batch_init(self.ctx_params.n_batch as i32, 0, 1);
        //     // ... batch processing ...
        //
        //     // Generate tokens
        //     let sampler = llama_sampler_chain_init(llama_sampler_chain_default_params());
        //     llama_sampler_chain_add(sampler, llama_sampler_init_temp(sampling.temperature));
        //     llama_sampler_chain_add(sampler, llama_sampler_init_top_p(sampling.top_p, 1));
        //     llama_sampler_chain_add(sampler, llama_sampler_init_dist(0));
        //
        //     for i in 0..sampling.max_tokens {
        //         if self.stop_flag.load(Ordering::Relaxed) { break; }
        //
        //         let token = llama_sampler_sample(sampler, self.ctx, -1);
        //         if llama_token_is_eog(self.model, token) { break; }
        //
        //         let piece = llama_token_to_piece(self.model, token);
        //         let should_continue = on_token(&piece, false);
        //         if !should_continue { break; }
        //
        //         // Prepare next batch
        //         llama_batch_clear(&mut batch);
        //         llama_batch_add(&mut batch, token, n_past + i, &[0], true);
        //         llama_decode(self.ctx, batch);
        //     }
        //
        //     on_token("", true); // signal done
        //     llama_sampler_free(sampler);
        //     llama_batch_free(batch);
        // }

        // Stub: generate a fake response
        let stub = format!(
            "**NexusAI** running `{}` with TQ{:?} KV cache\n\n\
             Context: {} tokens | Temperature: {:.1}\n\n\
             _{} messages in conversation_\n\n---\n\n\
             Real inference requires building llama.cpp. \
             Run `scripts/setup-llama-cpp.sh` then `cargo tauri dev`.",
            self.model_path.split('/').last().unwrap_or("model.gguf"),
            self.ctx_params.cache_type_k,
            self.ctx_params.n_ctx,
            sampling.temperature,
            messages.len()
        );

        let mut tokens_sent = 0u32;
        for chunk in stub.as_bytes().chunks(16) {
            if self.stop_flag.load(Ordering::Relaxed) {
                break;
            }
            let text = String::from_utf8_lossy(chunk);
            let should_continue = on_token(&text, false);
            if !should_continue {
                break;
            }
            tokens_sent += 1;
        }
        on_token("", true);

        let elapsed = start.elapsed();
        Ok(GenerationStats {
            tokens_generated: tokens_sent,
            prompt_tokens: prompt.len() as u32 / 4, // rough estimate
            tokens_per_second: tokens_sent as f64 / elapsed.as_secs_f64().max(0.001),
            time_to_first_token_ms: 50.0, // stub
            total_time_ms: elapsed.as_secs_f64() * 1000.0,
            kv_cache_used_bytes: 0,
            kv_cache_total_bytes: 0,
        })
    }

    /// Signal the generation loop to stop
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    /// Get the stop flag for sharing with async tasks
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        self.stop_flag.clone()
    }
}

impl Drop for LlamaSession {
    fn drop(&mut self) {
        if self.is_loaded {
            tracing::info!("Freeing model: {}", self.model_path);
            // TODO: unsafe { llama_free(self.ctx); llama_free_model(self.model); }
        }
    }
}

/// Build a ChatML-formatted prompt from message pairs
fn build_chatml_prompt(messages: &[(String, String)]) -> String {
    let mut prompt = String::new();
    for (role, content) in messages {
        prompt.push_str(&format!("<|im_start|>{}\n{}<|im_end|>\n", role, content));
    }
    prompt.push_str("<|im_start|>assistant\n");
    prompt
}

// Extern C declarations for llama.cpp (commented out until library is linked)
//
// extern "C" {
//     fn llama_backend_init();
//     fn llama_backend_free();
//     fn llama_model_default_params() -> LlamaModelParams;
//     fn llama_context_default_params() -> LlamaContextParams;
//     fn llama_load_model_from_file(path: *const c_char, params: LlamaModelParams) -> LlamaModel;
//     fn llama_free_model(model: LlamaModel);
//     fn llama_new_context_with_model(model: LlamaModel, params: LlamaContextParams) -> LlamaContext;
//     fn llama_free(ctx: LlamaContext);
//     fn llama_tokenize(model: LlamaModel, text: *const c_char, text_len: c_int, tokens: *mut LlamaToken, n_tokens_max: c_int, add_special: bool, parse_special: bool) -> c_int;
//     fn llama_decode(ctx: LlamaContext, batch: LlamaBatch) -> c_int;
//     fn llama_token_is_eog(model: LlamaModel, token: LlamaToken) -> bool;
//     fn llama_token_to_piece(model: LlamaModel, token: LlamaToken, buf: *mut c_char, length: c_int, lstrip: c_int, special: bool) -> c_int;
//     fn llama_kv_cache_view_init(ctx: LlamaContext, n_seq_max: c_int) -> LlamaKvCacheView;
//     fn llama_kv_cache_view_free(view: *mut LlamaKvCacheView);
// }
