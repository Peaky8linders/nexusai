use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_model: Option<String>,
    pub tq_kv_bits: u8,      // 3 or 4
    pub context_length: u32, // default 65536
    pub temperature: f32,    // default 0.7
    pub top_p: f32,          // default 0.9
    pub max_tokens: u32,     // default 4096
    pub gpu_layers: i32,     // -1 = all, 0 = CPU only
    pub theme: String,       // "dark" | "light"
    pub stream_response: bool,
    pub show_token_stats: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_model: None,
            tq_kv_bits: 3,
            context_length: 65536,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 4096,
            gpu_layers: -1,
            theme: "dark".into(),
            stream_response: true,
            show_token_stats: true,
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|d| d.join("settings.json"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    // Fall back to defaults if the file is malformed (e.g. schema changed after upgrade)
    Ok(serde_json::from_str(&json).unwrap_or_else(|_| AppSettings::default()))
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app)?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    tracing::info!(
        "Settings saved: TQ bits={}, ctx={}, temp={:.2}",
        settings.tq_kv_bits,
        settings.context_length,
        settings.temperature
    );
    Ok(())
}
