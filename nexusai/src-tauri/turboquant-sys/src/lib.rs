//! TurboQuant FFI bindings for Rust.
//!
//! This crate provides safe Rust wrappers around the TurboQuant C API.
//! In Phase 1, we use stub implementations. Once the llama.cpp fork is
//! compiled with TQ support, these will call into the actual C functions.

/// Quantization type
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum TqType {
    Tq3_0 = 0,
    Tq4_0 = 1,
}

/// Pre-computed codebook for scalar quantization
pub struct Codebook {
    pub tq_type: TqType,
    pub dim: usize,
    pub boundaries: Vec<f32>,
    pub centroids: Vec<f32>,
}

impl Codebook {
    /// Compute Lloyd-Max optimal codebook for Beta(d/2, d/2) distribution.
    ///
    /// TurboQuant insight: after WHT rotation, each coordinate of a unit vector
    /// follows Beta(d/2, d/2), which is nearly Gaussian for large d.
    /// The codebook is computed once and reused for all tokens.
    pub fn new(dim: usize, tq_type: TqType) -> Self {
        let num_levels = match tq_type {
            TqType::Tq3_0 => 8,   // 2^3
            TqType::Tq4_0 => 16,  // 2^4
        };

        // Stub: uniform boundaries/centroids
        // Real implementation uses iterative Lloyd-Max algorithm on Beta CDF
        let step = 1.0 / num_levels as f32;
        let boundaries: Vec<f32> = (1..num_levels).map(|i| i as f32 * step).collect();
        let centroids: Vec<f32> = (0..num_levels)
            .map(|i| (i as f32 + 0.5) * step)
            .collect();

        Self {
            tq_type,
            dim,
            boundaries,
            centroids,
        }
    }
}

/// TurboQuant quantization engine
pub struct TurboQuant {
    codebook: Codebook,
}

impl TurboQuant {
    pub fn new(dim: usize, tq_type: TqType) -> Self {
        Self {
            codebook: Codebook::new(dim, tq_type),
        }
    }

    /// Quantize a float vector using TurboQuant
    pub fn quantize(&self, input: &[f32]) -> Vec<u8> {
        assert_eq!(input.len(), self.codebook.dim);

        // Stub implementation:
        // 1. WHT rotation (Walsh-Hadamard Transform)
        // 2. Scalar quantize each coordinate
        // 3. Pack bits

        let bytes_needed = (input.len() * self.bits_per_element()) / 8;
        vec![0u8; bytes_needed] // placeholder
    }

    /// Dequantize packed data back to float vector
    pub fn dequantize(&self, data: &[u8]) -> Vec<f32> {
        // Stub: return zeros
        vec![0.0f32; self.codebook.dim]
    }

    /// Bits per element
    pub fn bits_per_element(&self) -> usize {
        match self.codebook.tq_type {
            TqType::Tq3_0 => 3,
            TqType::Tq4_0 => 4,
        }
    }

    /// Compression ratio vs FP16
    pub fn compression_ratio(&self) -> f64 {
        16.0 / self.bits_per_element() as f64
    }

    /// Memory for N tokens of KV cache (bytes)
    pub fn kv_memory_bytes(&self, num_tokens: usize, num_layers: usize, num_kv_heads: usize) -> usize {
        let elements = 2 * num_tokens * num_layers * num_kv_heads * self.codebook.dim;
        (elements * self.bits_per_element()) / 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codebook_creation() {
        let cb = Codebook::new(128, TqType::Tq3_0);
        assert_eq!(cb.boundaries.len(), 7);  // 2^3 - 1
        assert_eq!(cb.centroids.len(), 8);   // 2^3
    }

    #[test]
    fn test_compression_ratio() {
        let tq = TurboQuant::new(128, TqType::Tq3_0);
        assert!((tq.compression_ratio() - 5.333).abs() < 0.01);
    }

    #[test]
    fn test_kv_memory() {
        let tq = TurboQuant::new(128, TqType::Tq3_0);
        // 65536 tokens, 64 layers, 8 KV heads
        let mem = tq.kv_memory_bytes(65536, 64, 8);
        let fp16_mem = 2 * 65536 * 64 * 8 * 128 * 2; // FP16
        let ratio = fp16_mem as f64 / mem as f64;
        assert!(ratio > 5.0);
    }
}
