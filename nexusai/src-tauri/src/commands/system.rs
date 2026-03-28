use serde::Serialize;

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
    let total_memory = sys_info_total_memory();

    let (recommended_ctx, recommended_bits) = match total_memory {
        m if m >= 64.0 => (131072, 3), // 64GB+ → 128K context, TQ3
        m if m >= 32.0 => (65536, 3),  // 32GB → 64K, TQ3
        m if m >= 16.0 => (32768, 3),  // 16GB → 32K, TQ3
        _ => (16384, 4),               // <16GB → 16K, TQ4 (less compression but faster)
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

fn sys_info_total_memory() -> f64 {
    // Rough cross-platform memory detection — returns placeholder values.
    // TODO: Use sysinfo crate for real detection.
    if cfg!(target_os = "macos") {
        8.0
    } else if cfg!(target_os = "windows") {
        16.0
    } else {
        16.0
    }
}

fn detect_gpu_name() -> Option<String> {
    if cfg!(target_os = "macos") {
        Some("Apple Silicon (Metal)".into())
    } else {
        None // TODO: query CUDA/Vulkan
    }
}
