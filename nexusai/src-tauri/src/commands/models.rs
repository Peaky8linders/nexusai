use serde::{Deserialize, Serialize};
use tauri::State;

use crate::inference::EngineState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub quantization: String,
    pub parameters: String,
    pub family: String,
    pub downloaded: bool,
    pub loaded: bool,
    pub tq_compatible: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadProgress {
    pub model_id: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub percent: f64,
    pub speed_mbps: f64,
}

#[tauri::command]
pub async fn list_models() -> Result<Vec<ModelInfo>, String> {
    // Built-in model catalog — curated models known to work well with TurboQuant
    Ok(vec![
        ModelInfo {
            id: "qwen3.5-35b-a3b-q4km".into(),
            name: "Qwen 3.5 35B-A3B".into(),
            size_bytes: 19_000_000_000,
            quantization: "Q4_K_M".into(),
            parameters: "35B (3B active MoE)".into(),
            family: "Qwen".into(),
            downloaded: false,
            loaded: false,
            tq_compatible: true,
        },
        ModelInfo {
            id: "llama3.3-70b-q4km".into(),
            name: "Llama 3.3 70B".into(),
            size_bytes: 40_000_000_000,
            quantization: "Q4_K_M".into(),
            parameters: "70B".into(),
            family: "Llama".into(),
            downloaded: false,
            loaded: false,
            tq_compatible: true,
        },
        ModelInfo {
            id: "gemma3-27b-q4km".into(),
            name: "Gemma 3 27B".into(),
            size_bytes: 15_000_000_000,
            quantization: "Q4_K_M".into(),
            parameters: "27B".into(),
            family: "Gemma".into(),
            downloaded: false,
            loaded: false,
            tq_compatible: true,
        },
        ModelInfo {
            id: "phi4-14b-q4km".into(),
            name: "Phi-4 14B".into(),
            size_bytes: 8_000_000_000,
            quantization: "Q4_K_M".into(),
            parameters: "14B".into(),
            family: "Phi".into(),
            downloaded: false,
            loaded: false,
            tq_compatible: true,
        },
        ModelInfo {
            id: "nomic-embed-text-v1.5".into(),
            name: "Nomic Embed Text v1.5".into(),
            size_bytes: 274_000_000,
            quantization: "F16".into(),
            parameters: "137M".into(),
            family: "Nomic".into(),
            downloaded: false,
            loaded: false,
            tq_compatible: false,
        },
    ])
}

#[tauri::command]
pub async fn download_model(model_id: String) -> Result<(), String> {
    tracing::info!("Starting download for model: {}", model_id);
    // TODO: Implement HuggingFace download with resume support
    // This will use reqwest streaming + progress events
    Err("Model download not yet implemented — place GGUF files manually in the models directory".into())
}

#[tauri::command]
pub async fn delete_model(model_id: String) -> Result<(), String> {
    tracing::info!("Deleting model: {}", model_id);
    // TODO: Remove GGUF from models directory
    Ok(())
}

#[tauri::command]
pub async fn get_download_progress(_model_id: String) -> Result<Option<DownloadProgress>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn load_model(
    model_id: String,
    tq_bits: Option<u8>,
    engine: State<'_, EngineState>,
) -> Result<(), String> {
    let mut engine_guard = engine.0.lock().map_err(|e| e.to_string())?;
    engine_guard.load_model(&model_id, tq_bits.unwrap_or(3))?;
    Ok(())
}

#[tauri::command]
pub async fn unload_model(engine: State<'_, EngineState>) -> Result<(), String> {
    let mut engine_guard = engine.0.lock().map_err(|e| e.to_string())?;
    engine_guard.unload();
    Ok(())
}
