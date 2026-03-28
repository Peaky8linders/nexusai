use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub total_memory_gb: f64,
    pub gpu_name: Option<String>,
    pub gpu_memory_gb: Option<f64>,
    pub metal_supported: bool,
    pub cuda_supported: bool,
    pub recommended_context: u32,
    pub recommended_tq_bits: u8,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let total_memory = sys_total_memory_gb();

    let (recommended_ctx, recommended_bits) = match total_memory {
        m if m >= 64.0 => (131072, 3), // 64 GB+ → 128K context, TQ3
        m if m >= 32.0 => (65536, 3),  // 32 GB  → 64K,  TQ3
        m if m >= 16.0 => (32768, 3),  // 16 GB  → 32K,  TQ3
        _ => (16384, 4),               // <16 GB → 16K,  TQ4 (less compression, faster)
    };

    Ok(SystemInfo {
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        total_memory_gb: total_memory,
        gpu_name: detect_gpu_name(),
        gpu_memory_gb: None, // TODO: query via Metal/CUDA API
        metal_supported: cfg!(target_os = "macos"),
        cuda_supported: cfg!(feature = "cuda"),
        recommended_context: recommended_ctx,
        recommended_tq_bits: recommended_bits,
    })
}

/// Returns installed system RAM in GB using the sysinfo crate.
fn sys_total_memory_gb() -> f64 {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.total_memory() as f64 / 1_073_741_824.0 // bytes → GB
}

fn detect_gpu_name() -> Option<String> {
    if cfg!(target_os = "macos") {
        Some("Apple Silicon (Metal)".into())
    } else {
        None // TODO: query CUDA/Vulkan
    }
}
