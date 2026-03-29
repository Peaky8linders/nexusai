use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::inference::EngineState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub model_id: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub percent: f64,
    pub speed_mbps: f64,
    pub eta_seconds: f64,
    pub status: String, // "downloading" | "complete" | "error"
    pub error: Option<String>,
}

/// Known model catalog with HuggingFace repo + filename mappings.
struct ModelEntry {
    id: &'static str,
    name: &'static str,
    size_bytes: u64,
    quantization: &'static str,
    parameters: &'static str,
    family: &'static str,
    tq_compatible: bool,
    hf_repo: &'static str,
    hf_filename: &'static str,
}

const MODEL_CATALOG: &[ModelEntry] = &[
    ModelEntry {
        id: "llama3.2-3b",
        name: "Llama 3.2 3B",
        size_bytes: 2_000_000_000,
        quantization: "Q4_K_M",
        parameters: "3B",
        family: "Llama",
        tq_compatible: true,
        hf_repo: "",
        hf_filename: "",
    },
    ModelEntry {
        id: "qwen3-30b-a3b",
        name: "Qwen 3 30B-A3B",
        size_bytes: 18_000_000_000,
        quantization: "Q4_K_M",
        parameters: "30B (3B active MoE)",
        family: "Qwen",
        tq_compatible: true,
        hf_repo: "",
        hf_filename: "",
    },
    ModelEntry {
        id: "qwen3.5-35b-a3b-q4km",
        name: "Qwen 3.5 35B-A3B",
        size_bytes: 19_000_000_000,
        quantization: "Q4_K_M",
        parameters: "35B (3B active MoE)",
        family: "Qwen",
        tq_compatible: true,
        hf_repo: "Qwen/Qwen2.5-35B-A3B-Instruct-GGUF",
        hf_filename: "qwen2.5-35b-a3b-instruct-q4_k_m.gguf",
    },
    ModelEntry {
        id: "llama3.3-70b-q4km",
        name: "Llama 3.3 70B",
        size_bytes: 40_000_000_000,
        quantization: "Q4_K_M",
        parameters: "70B",
        family: "Llama",
        tq_compatible: true,
        hf_repo: "bartowski/Llama-3.3-70B-Instruct-GGUF",
        hf_filename: "Llama-3.3-70B-Instruct-Q4_K_M.gguf",
    },
    ModelEntry {
        id: "gemma3-27b-q4km",
        name: "Gemma 3 27B",
        size_bytes: 15_000_000_000,
        quantization: "Q4_K_M",
        parameters: "27B",
        family: "Gemma",
        tq_compatible: true,
        hf_repo: "bartowski/gemma-3-27b-it-GGUF",
        hf_filename: "gemma-3-27b-it-Q4_K_M.gguf",
    },
    ModelEntry {
        id: "phi4-14b-q4km",
        name: "Phi-4 14B",
        size_bytes: 8_000_000_000,
        quantization: "Q4_K_M",
        parameters: "14B",
        family: "Phi",
        tq_compatible: true,
        hf_repo: "bartowski/phi-4-GGUF",
        hf_filename: "phi-4-Q4_K_M.gguf",
    },
    ModelEntry {
        id: "qwen3-8b-q4km",
        name: "Qwen 3 8B",
        size_bytes: 4_500_000_000,
        quantization: "Q4_K_M",
        parameters: "8B",
        family: "Qwen",
        tq_compatible: true,
        hf_repo: "Qwen/Qwen3-8B-GGUF",
        hf_filename: "qwen3-8b-q4_k_m.gguf",
    },
    ModelEntry {
        id: "nomic-embed-text-v1.5",
        name: "Nomic Embed Text v1.5",
        size_bytes: 274_000_000,
        quantization: "F16",
        parameters: "137M",
        family: "Nomic",
        tq_compatible: false,
        hf_repo: "nomic-ai/nomic-embed-text-v1.5-GGUF",
        hf_filename: "nomic-embed-text-v1.5.Q4_0.gguf",
    },
];

fn models_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|d| d.join("models"))
        .map_err(|e| e.to_string())
}

/// Ollama model tag from /api/tags response
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelTag>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelTag {
    name: String,
}

/// Map catalog IDs to their Ollama model names for download detection
fn catalog_id_to_ollama_name(id: &str) -> Option<&str> {
    match id {
        "llama3.2-3b" => Some("llama3.2:3b"),
        "qwen3-30b-a3b" => Some("qwen3:30b-a3b"),
        "qwen3.5-35b-a3b-q4km" => Some("qwen3:30b-a3b"),
        "qwen3-8b-q4km" => Some("qwen3:8b"),
        "llama3.3-70b-q4km" => Some("llama3.3:70b"),
        "gemma3-27b-q4km" => Some("gemma3:27b"),
        "phi4-14b-q4km" => Some("phi4:14b"),
        _ => None,
    }
}

#[tauri::command]
pub async fn list_models(app: AppHandle) -> Result<Vec<ModelInfo>, String> {
    let dir = models_dir(&app)?;
    // Collect downloaded GGUF filenames for O(1) lookup
    let downloaded_gguf: std::collections::HashSet<String> = if dir.exists() {
        crate::inference::download::scan_models_dir(&dir)
            .await
            .into_iter()
            .filter_map(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase())
            })
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    // Query Ollama for installed models
    let ollama_models: std::collections::HashSet<String> = match reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<OllamaTagsResponse>()
            .await
            .map(|r| r.models.into_iter().map(|m| m.name).collect())
            .unwrap_or_default(),
        Err(_) => std::collections::HashSet::new(),
    };

    Ok(MODEL_CATALOG
        .iter()
        .map(|e| {
            let gguf_downloaded = !e.hf_filename.is_empty()
                && downloaded_gguf.contains(&e.hf_filename.to_lowercase());
            let ollama_available = catalog_id_to_ollama_name(e.id)
                .map(|name| ollama_models.contains(name))
                .unwrap_or(false);

            ModelInfo {
                id: e.id.into(),
                name: e.name.into(),
                size_bytes: e.size_bytes,
                quantization: e.quantization.into(),
                parameters: e.parameters.into(),
                family: e.family.into(),
                downloaded: gguf_downloaded || ollama_available,
                loaded: false,
                tq_compatible: e.tq_compatible,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    model_id: String,
) -> Result<(), String> {
    let entry = MODEL_CATALOG
        .iter()
        .find(|e| e.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let dir = models_dir(&app)?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let repo = entry.hf_repo;
    let filename = entry.hf_filename;
    let mid = model_id.clone();
    let app2 = app.clone();

    // Spawn download as background task so the command returns immediately.
    // Progress is delivered via "download:progress" events.
    tokio::spawn(async move {
        let result = crate::inference::download::download_model(
            repo,
            filename,
            &dir,
            move |progress| {
                let evt = DownloadProgress {
                    model_id: mid.clone(),
                    bytes_downloaded: progress.bytes_downloaded,
                    bytes_total: progress.bytes_total,
                    percent: progress.percent,
                    speed_mbps: progress.speed_mbps,
                    eta_seconds: progress.eta_seconds,
                    status: match &progress.status {
                        crate::inference::download::DownloadStatus::Complete => "complete".into(),
                        crate::inference::download::DownloadStatus::Failed(e) => {
                            format!("error: {}", e)
                        }
                        _ => "downloading".into(),
                    },
                    error: None,
                };
                app2.emit("download:progress", &evt).ok();
            },
        )
        .await;

        if let Err(e) = result {
            tracing::error!("Download failed for {}: {}", filename, e);
            app.emit(
                "download:progress",
                &DownloadProgress {
                    model_id: model_id.clone(),
                    bytes_downloaded: 0,
                    bytes_total: 0,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    eta_seconds: 0.0,
                    status: "error".into(),
                    error: Some(e),
                },
            )
            .ok();
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn delete_model(app: AppHandle, model_id: String) -> Result<(), String> {
    let entry = MODEL_CATALOG
        .iter()
        .find(|e| e.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let path = models_dir(&app)?.join(entry.hf_filename);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        tracing::info!("Deleted model file: {:?}", path);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_download_progress(_model_id: String) -> Result<Option<DownloadProgress>, String> {
    // Progress is push-based via "download:progress" events; this endpoint
    // is kept for compatibility but always returns None.
    Ok(None)
}

#[tauri::command]
pub async fn load_model(
    model_id: String,
    tq_bits: Option<u8>,
    engine: State<'_, EngineState>,
) -> Result<(), String> {
    let bits = tq_bits.unwrap_or(3);
    if bits != 3 && bits != 4 {
        return Err(format!("Invalid tq_bits: {}. Must be 3 or 4.", bits));
    }
    let mut engine_guard = engine.0.lock().await;
    engine_guard.load_model(&model_id, bits)?;
    Ok(())
}

#[tauri::command]
pub async fn unload_model(engine: State<'_, EngineState>) -> Result<(), String> {
    let mut engine_guard = engine.0.lock().await;
    engine_guard.unload();
    Ok(())
}
