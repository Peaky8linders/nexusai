use serde::{Deserialize, Serialize};

/// TurboQuant configuration for KV cache compression.
///
/// TurboQuant works by:
/// 1. Applying a Walsh-Hadamard Transform (WHT) rotation to make vector coordinates
///    follow a predictable Beta distribution
/// 2. Using pre-computed Lloyd-Max scalar quantization codebooks per coordinate
/// 3. Applying 1-bit QJL (Quantized Johnson-Lindenstrauss) on the residual for
///    unbiased inner product estimation
///
/// This is "online" (no calibration data needed) and the codebooks are computed once
/// at startup based on the head dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurboQuantConfig {
    /// Quantization bits: 3 (TQ3_0) or 4 (TQ4_0)
    pub bits: u8,

    /// Maximum context length achievable with this compression
    pub max_context: u32,

    /// Head dimension for the model (typically 64, 128, or 256)
    pub head_dim: u32,

    /// Whether to use QJL residual correction (improves quality, slight overhead)
    pub use_qjl_residual: bool,

    /// Number of random rotation matrices to ensemble (more = better quality, slower)
    pub num_rotations: u32,
}

impl Default for TurboQuantConfig {
    fn default() -> Self {
        Self {
            bits: 3,
            max_context: 65536,
            head_dim: 128,
            use_qjl_residual: true,
            num_rotations: 1,
        }
    }
}

impl TurboQuantConfig {
    pub fn new(bits: u8) -> Self {
        let bits = if bits == 4 { 4 } else { 3 }; // clamp to supported values
        let max_context = match bits {
            3 => 65536,  // ~5.3x compression → much larger context fits
            4 => 49152,  // ~4x compression
            _ => 32768,
        };
        Self {
            bits,
            max_context,
            ..Default::default()
        }
    }

    /// Memory compression ratio vs FP16 KV cache
    pub fn compression_ratio(&self) -> f64 {
        match self.bits {
            3 => 16.0 / 3.0,  // ~5.33x
            4 => 16.0 / 4.0,  // 4x
            _ => 1.0,
        }
    }

    /// Estimated KV cache memory in bytes for given context length and model params
    pub fn estimate_kv_memory(&self, context_len: u32, num_layers: u32, num_kv_heads: u32) -> u64 {
        let bytes_per_element = self.bits as u64;
        let elements_per_token = 2 * num_layers as u64 * num_kv_heads as u64 * self.head_dim as u64;
        // bits to bytes
        (context_len as u64 * elements_per_token * bytes_per_element) / 8
    }

    /// Estimated KV cache memory with FP16 (for comparison)
    pub fn estimate_kv_memory_fp16(&self, context_len: u32, num_layers: u32, num_kv_heads: u32) -> u64 {
        let elements_per_token = 2 * num_layers as u64 * num_kv_heads as u64 * self.head_dim as u64;
        context_len as u64 * elements_per_token * 2 // 2 bytes per FP16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_ratios() {
        let tq3 = TurboQuantConfig::new(3);
        assert!((tq3.compression_ratio() - 5.33).abs() < 0.1);

        let tq4 = TurboQuantConfig::new(4);
        assert!((tq4.compression_ratio() - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_kv_memory_estimation() {
        // Qwen 3.5 35B: 64 layers, 8 KV heads, head_dim 128
        let tq3 = TurboQuantConfig::new(3);
        let tq_mem = tq3.estimate_kv_memory(65536, 64, 8);
        let fp16_mem = tq3.estimate_kv_memory_fp16(65536, 64, 8);

        // TQ3 should be ~5.3x smaller
        let ratio = fp16_mem as f64 / tq_mem as f64;
        assert!(ratio > 5.0 && ratio < 6.0);
    }
}
