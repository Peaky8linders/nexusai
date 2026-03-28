#!/bin/bash
# NexusAI — llama.cpp fork setup with TurboQuant support
#
# This script:
# 1. Clones llama.cpp
# 2. Adds TurboQuant remotes (community forks)
# 3. Cherry-picks TQ3_0 type support
# 4. Builds with Metal (macOS) or CUDA (Linux/Windows)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LLAMA_DIR="$PROJECT_DIR/src-tauri/llama-cpp"

echo "=== NexusAI llama.cpp Setup ==="
echo ""

# Step 1: Clone llama.cpp
if [ ! -d "$LLAMA_DIR" ]; then
    echo "[1/4] Cloning llama.cpp..."
    git clone https://github.com/ggml-org/llama.cpp "$LLAMA_DIR"
else
    echo "[1/4] llama.cpp already cloned at $LLAMA_DIR"
fi

cd "$LLAMA_DIR"

# Step 2: Add TurboQuant forks as remotes
echo "[2/4] Adding TurboQuant fork remotes..."

# CPU implementation (Aaryan-Kapoor)
git remote add tq-cpu https://github.com/Aaryan-Kapoor/llama.cpp 2>/dev/null || true
git fetch tq-cpu turboquant-tq3_0 2>/dev/null || echo "  Warning: Could not fetch tq-cpu remote"

# Metal implementation (TheTom)
git remote add tq-metal https://github.com/TheTom/llama.cpp 2>/dev/null || true
git fetch tq-metal turboquant_plus 2>/dev/null || echo "  Warning: Could not fetch tq-metal remote"

# CUDA implementation (spiritbuun)
git remote add tq-cuda https://github.com/spiritbuun/llama.cpp 2>/dev/null || true
git fetch tq-cuda 2>/dev/null || echo "  Warning: Could not fetch tq-cuda remote"

echo "  Remotes configured."

# Step 3: Create NexusAI branch and merge TQ commits
echo "[3/4] Creating nexusai-tq branch..."
if git rev-parse --verify nexusai-tq >/dev/null 2>&1; then
    git checkout nexusai-tq
    echo "  Branch nexusai-tq already exists, checked out."
else
    git checkout -b nexusai-tq
    echo "  Created new branch nexusai-tq."
fi

# Merge TQ CPU support (this may require manual conflict resolution)
echo "  Merging TQ3_0 support from tq-cpu..."
if git rev-parse --verify tq-cpu/turboquant-tq3_0 >/dev/null 2>&1; then
    git merge tq-cpu/turboquant-tq3_0 --no-edit || {
        echo "  ⚠ Merge conflict — resolve manually, then re-run this script."
        echo "  Run: git mergetool && git merge --continue"
        exit 1
    }
else
    echo "  ⚠ tq-cpu/turboquant-tq3_0 branch not available — skipping merge."
    echo "  You can manually apply TQ patches later."
fi

# Step 4: Build
echo "[4/4] Building llama.cpp..."

OS=$(uname -s)
case $OS in
    Darwin)
        echo "  Building for macOS with Metal..."
        cmake -B build \
            -DGGML_METAL=ON \
            -DCMAKE_BUILD_TYPE=Release \
            -DBUILD_SHARED_LIBS=ON
        cmake --build build -j$(sysctl -n hw.ncpu)
        ;;
    Linux)
        echo "  Building for Linux..."
        if command -v nvcc &>/dev/null; then
            echo "  CUDA detected, building with CUDA support..."
            cmake -B build \
                -DGGML_CUDA=ON \
                -DCMAKE_BUILD_TYPE=Release \
                -DBUILD_SHARED_LIBS=ON
        else
            echo "  No CUDA, building CPU-only..."
            cmake -B build \
                -DCMAKE_BUILD_TYPE=Release \
                -DBUILD_SHARED_LIBS=ON
        fi
        cmake --build build -j$(nproc)
        ;;
    MINGW*|MSYS*|CYGWIN*)
        echo "  Building for Windows..."
        cmake -B build \
            -DCMAKE_BUILD_TYPE=Release \
            -DBUILD_SHARED_LIBS=ON
        cmake --build build --config Release -j
        ;;
    *)
        echo "  Unknown OS: $OS"
        exit 1
        ;;
esac

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Test TurboQuant KV cache:"
echo "  ./build/bin/llama-cli \\"
echo "    -m /path/to/model.gguf \\"
echo "    --cache-type-k turbo3 \\"
echo "    --cache-type-v turbo3 \\"
echo "    -c 65536 \\"
echo '    -p "Hello, world!"'
