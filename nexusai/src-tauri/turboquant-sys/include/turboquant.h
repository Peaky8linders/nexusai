/**
 * TurboQuant C API — FFI bridge for Rust.
 *
 * This header defines the public interface for TurboQuant quantization
 * and dequantization operations. The implementation will be provided by
 * the llama.cpp fork with TQ3_0/TQ4_0 GGML types.
 *
 * TurboQuant algorithm (from Google Research):
 *   1. Apply randomized Walsh-Hadamard Transform (WHT) to input vector
 *   2. Each coordinate now follows Beta(d/2, d/2) distribution
 *   3. Apply pre-computed Lloyd-Max scalar quantizer per coordinate
 *   4. Store quantized values in packed format (3 or 4 bits)
 *   5. Compute 1-bit QJL projection of residual for unbiased estimation
 */

#ifndef TURBOQUANT_H
#define TURBOQUANT_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Quantization type */
typedef enum {
    TQ_TYPE_TQ3_0 = 0,  /* 3-bit TurboQuant */
    TQ_TYPE_TQ4_0 = 1,  /* 4-bit TurboQuant */
} tq_type_t;

/* Pre-computed codebook for a given (dimension, bit-width) pair */
typedef struct {
    tq_type_t type;
    int       dim;          /* head dimension (64, 128, 256) */
    int       num_levels;   /* 2^bits quantization levels */
    float*    boundaries;   /* decision boundaries, len = num_levels - 1 */
    float*    centroids;    /* reconstruction values, len = num_levels */
} tq_codebook_t;

/* Quantized KV cache entry */
typedef struct {
    uint8_t*  data;         /* packed quantized values */
    uint8_t*  qjl_bits;    /* 1-bit QJL residual projections */
    float     scale;        /* scale factor for dequantization */
    int       dim;
    int       num_tokens;
    tq_type_t type;
} tq_kv_cache_t;

/**
 * Initialize codebooks for the given head dimension.
 * These are computed once at model load time using the Beta distribution CDF.
 * Returns 0 on success, -1 on error.
 */
int tq_init_codebooks(int head_dim, tq_type_t type, tq_codebook_t* out);

/**
 * Free codebook resources.
 */
void tq_free_codebooks(tq_codebook_t* cb);

/**
 * Quantize a float vector in-place using TurboQuant.
 *   1. Apply WHT rotation
 *   2. Scalar quantize each coordinate using codebook
 *   3. Pack into output buffer
 *   4. Compute QJL residual bits
 *
 * @param input     Input float vector, length = dim
 * @param dim       Vector dimension
 * @param codebook  Pre-computed codebook
 * @param output    Output packed buffer (caller-allocated)
 * @param qjl_out   Output QJL bits (caller-allocated, dim/8 bytes)
 * @return 0 on success
 */
int tq_quantize(
    const float*          input,
    int                   dim,
    const tq_codebook_t*  codebook,
    uint8_t*              output,
    uint8_t*              qjl_out
);

/**
 * Dequantize packed values back to float vector.
 *   1. Unpack quantized values
 *   2. Map through codebook centroids
 *   3. Apply inverse WHT rotation
 *
 * @param input     Packed quantized data
 * @param dim       Vector dimension
 * @param codebook  Pre-computed codebook
 * @param output    Output float vector (caller-allocated, length = dim)
 * @return 0 on success
 */
int tq_dequantize(
    const uint8_t*        input,
    int                   dim,
    const tq_codebook_t*  codebook,
    float*                output
);

/**
 * Compute inner product between a query vector (float) and a
 * TQ-quantized KV cache entry. Uses QJL correction for unbiased estimation.
 *
 * This is the key operation for attention score computation.
 */
float tq_inner_product(
    const float*          query,
    const uint8_t*        quantized_key,
    const uint8_t*        qjl_bits,
    int                   dim,
    const tq_codebook_t*  codebook
);

/**
 * Allocate a TQ KV cache for the given number of tokens.
 */
int tq_kv_cache_alloc(
    tq_kv_cache_t*  cache,
    int             dim,
    int             max_tokens,
    tq_type_t       type
);

/**
 * Free KV cache resources.
 */
void tq_kv_cache_free(tq_kv_cache_t* cache);

/**
 * Get memory usage in bytes for current cache state.
 */
size_t tq_kv_cache_memory_usage(const tq_kv_cache_t* cache);

#ifdef __cplusplus
}
#endif

#endif /* TURBOQUANT_H */
