use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_model: Option<String>,
    pub tq_kv_bits: u8,          // 3 or 4
    pub context_length: u32,     // default 65536
    pub temperature: f32,        // default 0.7
    pub top_p: f32,              // default 0.9
    pub max_tokens: u32,         // default 4096
    pub gpu_layers: i32,         // -1 = all, 0 = CPU only
    pub theme: String,           // "dark" | "light"
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

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    // TODO: Load from config file
    Ok(AppSettings::default())
}

#[tauri::command]
pub async fn update_settings(settings: AppSettings) -> Result<(), String> {
    tracing::info!("Updating settings: TQ bits={}, ctx={}", settings.tq_kv_bits, settings.context_length);
    // TODO: Persist to config file
    Ok(())
}
