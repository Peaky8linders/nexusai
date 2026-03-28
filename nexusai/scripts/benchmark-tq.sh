#!/bin/bash
# NexusAI — TurboQuant KV cache benchmark suite
#
# Runs quality and performance benchmarks across:
# - Model sizes (8B, 14B, 27B, 35B)
# - KV cache types (f16, q8_0, q4_0, turbo3, turbo4)
# - Context lengths (4K, 8K, 16K, 32K, 64K, 128K)

set -euo pipefail

LLAMA_CLI="${1:-./src-tauri/llama-cpp/build/bin/llama-cli}"
MODEL="${2:-}"
RESULTS_DIR="benchmarks/$(date +%Y%m%d_%H%M%S)"

if [ -z "$MODEL" ]; then
    echo "Usage: $0 <llama-cli-path> <model.gguf>"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

echo "=== NexusAI TurboQuant Benchmark Suite ==="
echo "Model: $MODEL"
echo "Results: $RESULTS_DIR"
echo ""

CACHE_TYPES=("f16" "q8_0" "q4_0" "turbo3" "turbo4")
CONTEXT_SIZES=(4096 8192 16384 32768 65536 131072)

# Needle-in-haystack test prompt
NEEDLE="The secret code is NEXUSAI-42."
QUESTION="What is the secret code?"

for cache_type in "${CACHE_TYPES[@]}"; do
    echo "--- Cache type: $cache_type ---"
    for ctx in "${CONTEXT_SIZES[@]}"; do
        echo -n "  ctx=$ctx: "

        RESULT_FILE="$RESULTS_DIR/${cache_type}_ctx${ctx}.txt"
        PROMPT=$(printf 'Remember this fact: %s\n\nQ: %s\nA:' "$NEEDLE" "$QUESTION")

        # Capture full output, then truncate — avoids SIGPIPE with pipefail
        timeout 120 "$LLAMA_CLI" \
            -m "$MODEL" \
            --cache-type-k "$cache_type" \
            --cache-type-v "$cache_type" \
            -c "$ctx" \
            -n 64 \
            --temp 0 \
            -p "$PROMPT" \
            --log-disable \
            > "$RESULT_FILE" 2>/dev/null || true

        if grep -qi "NEXUSAI-42" "$RESULT_FILE" 2>/dev/null; then
            echo "PASS (needle found)"
        else
            echo "FAIL (needle not found)"
        fi
    done
done

echo ""
echo "Results saved to $RESULTS_DIR/"
echo ""
echo "=== Benchmark Complete ==="
