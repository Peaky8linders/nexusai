pub mod download;
pub mod llama_ffi;
pub mod tq_config;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use tq_config::TurboQuantConfig;

/// Wraps the inference engine in Tauri managed state
pub struct EngineState(pub tokio::sync::Mutex<InferenceEngine>);

/// Ollama chat message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

/// Ollama streaming response chunk (one NDJSON line)
#[derive(Debug, Deserialize)]
pub struct OllamaStreamChunk {
    pub message: Option<OllamaMessage>,
    pub done: bool,
    #[serde(default)]
    pub eval_count: Option<u64>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

/// Core inference engine that manages Ollama connectivity and model state.
pub struct InferenceEngine {
    model_loaded: bool,
    model_id: Option<String>,
    ollama_model: Option<String>,
    ollama_url: String,
    tq_config: TurboQuantConfig,
    stop_flag: Arc<AtomicBool>,
    http_client: Client,
}

/// Map internal model catalog IDs to Ollama model names
fn catalog_id_to_ollama(model_id: &str) -> String {
    match model_id {
        "llama3.2-3b" => "llama3.2:3b".to_string(),
        "qwen3-30b-a3b" => "qwen3:30b-a3b".to_string(),
        "qwen3.5-35b-a3b-q4km" => "qwen3:30b-a3b".to_string(),
        "qwen3-8b-q4km" => "qwen3:8b".to_string(),
        "llama3.3-70b-q4km" => "llama3.3:70b".to_string(),
        "gemma3-27b-q4km" => "gemma3:27b".to_string(),
        "phi4-14b-q4km" => "phi4:14b".to_string(),
        // If it doesn't match a catalog ID, assume it's already an Ollama model name
        other => other.to_string(),
    }
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            model_loaded: false,
            model_id: None,
            ollama_model: None,
            ollama_url: "http://localhost:11434".to_string(),
            tq_config: TurboQuantConfig::default(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            http_client: Client::new(),
        }
    }

    pub fn load_model(&mut self, model_id: &str, tq_bits: u8) -> Result<(), String> {
        let ollama_name = catalog_id_to_ollama(model_id);
        tracing::info!(
            "Loading model {} (ollama: {}) with TurboQuant TQ{} KV cache",
            model_id,
            ollama_name,
            tq_bits
        );

        self.tq_config = TurboQuantConfig::new(tq_bits);
        self.model_id = Some(model_id.to_string());
        self.ollama_model = Some(ollama_name);
        self.model_loaded = true;

        Ok(())
    }

    pub fn unload(&mut self) {
        tracing::info!("Unloading model");
        self.model_loaded = false;
        self.model_id = None;
        self.ollama_model = None;
    }

    /// Stream a chat response from Ollama, calling `on_token` for each chunk.
    /// Returns the full accumulated response text.
    pub async fn stream_chat(
        &self,
        messages: Vec<OllamaMessage>,
        on_token: impl Fn(&str, Option<f64>) + Send,
    ) -> Result<String, String> {
        if !self.model_loaded {
            return Err("No model loaded. Select and load a model first.".into());
        }

        let model = self.ollama_model.as_deref().unwrap_or("qwen3:8b");
        self.stop_flag.store(false, Ordering::Relaxed);

        let url = format!("{}/api/chat", self.ollama_url);
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
        });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    "Ollama is not running. Start it with `ollama serve` or check that it's running on localhost:11434.".to_string()
                } else {
                    format!("Ollama request failed: {}", e)
                }
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama returned {}: {}", status, body_text));
        }

        let mut full_response = String::new();
        let mut bytes_buf = Vec::new();
        let mut stream = resp.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            if self.stop_flag.load(Ordering::Relaxed) {
                tracing::info!("Generation stopped by user");
                break;
            }

            let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
            bytes_buf.extend_from_slice(&chunk);

            // Process complete NDJSON lines from the buffer
            while let Some(newline_pos) = bytes_buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = bytes_buf.drain(..=newline_pos).collect();
                let line_str = String::from_utf8_lossy(&line);
                let trimmed = line_str.trim();

                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<OllamaStreamChunk>(trimmed) {
                    Ok(parsed) => {
                        // Calculate tokens/sec (only present on final chunk)
                        let tps = match (parsed.eval_count, parsed.eval_duration) {
                            (Some(count), Some(dur)) if dur > 0 => {
                                Some(count as f64 / (dur as f64 / 1_000_000_000.0))
                            }
                            _ => None,
                        };

                        if let Some(msg) = &parsed.message {
                            if !msg.content.is_empty() {
                                full_response.push_str(&msg.content);
                                on_token(&msg.content, tps);
                            }
                        }

                        if parsed.done {
                            if let Some(tps_val) = tps {
                                tracing::info!(
                                    "Generation complete: {:.1} tokens/sec",
                                    tps_val
                                );
                            }
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse Ollama chunk: {} — line: {}", e, trimmed);
                    }
                }
            }
        }

        Ok(full_response)
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_loaded(&self) -> bool {
        self.model_loaded
    }

    pub fn model_id(&self) -> Option<&str> {
        self.model_id.as_deref()
    }

    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        self.stop_flag.clone()
    }

    pub fn ollama_url(&self) -> &str {
        &self.ollama_url
    }
}
