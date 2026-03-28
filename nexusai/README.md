# NexusAI

**TurboQuant-powered local-first AI agent runtime.**

Run 35B+ parameter models with 64K+ context windows on consumer hardware. No cloud. No API keys. Your models, your hardware, your data.

## What is this?

NexusAI is a desktop application that runs large language models locally using [TurboQuant](https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression/) — Google's KV cache compression algorithm that achieves **5.3x memory reduction** without meaningful quality loss.

This means a model that normally needs 48GB of RAM for a 64K context window now fits in 16GB.

```
Traditional KV Cache (FP16):  12.8 GB for 64K context on a 35B model
TurboQuant TQ3 (3-bit):        2.4 GB for the same setup
                              ─────────
                              5.3x smaller
```

## Features

- **Chat UI** — Streaming responses, markdown rendering, conversation management
- **Model Manager** — Curated catalog of GGUF models (Qwen, Llama, Gemma, Phi) with download from HuggingFace
- **TurboQuant KV Cache** — TQ3 (5.3x) and TQ4 (4x) compression modes, configurable per-session
- **RAG Pipeline** — Drag-drop document indexing with structure-aware chunking and TQ-compressed vector search
- **Agent Framework** — Multi-format tool call parsing (Llama 3.x, Qwen/ChatML, generic JSON), approval gates, multi-step execution
- **Persistent Memory** — SQLite-backed memory store for user facts, preferences, and conversation history with full-text search
- **MCP Support** — Model Context Protocol plugin registry for extensible tool ecosystem

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    NexusAI Desktop App                   │
│                   (Tauri v2 + React 19)                  │
├─────────────┬──────────────┬───────────────┬────────────┤
│  Chat UI    │  Agent Panel │  RAG Explorer │  Settings  │
│  (Markdown, │  (Tool calls,│  (Drag-drop   │  (Model    │
│   stream)   │   approvals) │   docs, index)│   select)  │
├─────────────┴──────────────┴───────────────┴────────────┤
│              NexusAI Core Engine (Rust)                  │
├──────────────────┬──────────────────────────────────────┤
│  Model Manager   │  Memory / Search (SQLite)            │
├──────────────────┴──────────────────────────────────────┤
│         TurboQuant Inference Layer (C/C++)               │
├───────────────┬────────────────────┬────────────────────┤
│  llama.cpp    │  TQ KV Cache       │  TQ Vector Index   │
│  (forked)     │  (turbo3/turbo4)   │  (RAG embeddings)  │
├───────────────┴────────────────────┴────────────────────┤
│  Hardware: Metal (Mac) │ CUDA (Nvidia) │ Vulkan │ CPU   │
└─────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19, TypeScript, Tailwind CSS, Zustand |
| Desktop | Tauri v2 (5MB binary vs 150MB Electron) |
| Backend | Rust, SQLite (rusqlite), tokio async |
| Inference | llama.cpp fork with TurboQuant GGML types |
| Embeddings | nomic-embed-text-v1.5 (local) |
| Compression | TurboQuant TQ3/TQ4 (Walsh-Hadamard + Lloyd-Max) |

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) 18+ (for frontend build)
- [Rust](https://rustup.rs/) stable (for Tauri backend)
- ~16GB RAM recommended

### 1. Clone and install

```bash
git clone https://github.com/Peaky8linders/nexusai.git
cd nexusai/nexusai
npm install
```

### 2. Set up llama.cpp with TurboQuant

```bash
bash scripts/setup-llama-cpp.sh
```

This clones llama.cpp, merges TurboQuant branches from community forks, and builds with Metal (Mac) or CUDA (Linux/Windows with Nvidia GPU).

### 3. Run in development

```bash
cargo tauri dev
```

### 4. Build for production

```bash
cargo tauri build
```

The output is a native installer (.dmg on Mac, .exe on Windows, .AppImage on Linux).

### Frontend-only development (no Rust required)

The frontend runs standalone with stub responses for UI development:

```bash
npm run dev
# Open http://localhost:1420
```

## Project Structure

```
nexusai/
├── src/                          # React frontend
│   ├── components/
│   │   ├── chat/                 # ChatView, MessageBubble, ChatInput
│   │   ├── agent/                # AgentPanel (tool call approval UI)
│   │   ├── rag/                  # RagExplorer (drag-drop indexing)
│   │   ├── settings/             # SettingsPanel, ModelSelector, MemoryPanel
│   │   └── sidebar/              # Sidebar with Chat/RAG/Memory tabs
│   ├── stores/appStore.ts        # Zustand state management
│   ├── lib/tauri.ts              # Tauri IPC bridge
│   └── types/index.ts            # Shared TypeScript types
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/             # Tauri IPC commands (chat, models, settings, system)
│   │   ├── inference/            # llama.cpp FFI, model download, TQ config
│   │   ├── rag/                  # Chunker, embedder, vector index, file parsers
│   │   ├── tools/                # Tool call parser, agent execution loop
│   │   ├── memory/               # SQLite persistence, search, memory injection
│   │   ├── mcp/                  # MCP server registry
│   │   └── models/               # GGUF model catalog
│   ├── turboquant-sys/           # TurboQuant C API + Rust FFI bindings
│   │   ├── include/turboquant.h  # C header: codebook, quantize, dequantize, inner product
│   │   └── src/lib.rs            # Rust bindings with compression math
│   └── capabilities/             # Tauri v2 permission capabilities
│
└── scripts/
    ├── setup-llama-cpp.sh        # Clone + patch + build llama.cpp with TQ support
    └── benchmark-tq.sh           # Needle-in-haystack quality benchmark suite
```

## How TurboQuant Works

TurboQuant compresses the KV cache — the memory bottleneck that limits context length in transformer models.

1. **Random Rotation**: Apply a Walsh-Hadamard Transform (WHT) to each key/value vector. After rotation, each coordinate follows a predictable Beta distribution.

2. **Scalar Quantization**: Use pre-computed Lloyd-Max codebooks (computed once at startup) to quantize each coordinate from 16 bits down to 3 bits (TQ3) or 4 bits (TQ4).

3. **QJL Residual**: Apply a 1-bit Quantized Johnson-Lindenstrauss projection to the quantization error, ensuring unbiased inner product estimation for attention scores.

The result: **online** (no calibration data needed), **calibration-free** compression that works on any model architecture.

## Supported Models

| Model | Parameters | Size (Q4_K_M) | Min RAM | TQ Compatible |
|-------|-----------|---------------|---------|---------------|
| Qwen 3.5 35B-A3B | 35B (3B active MoE) | 19 GB | 16 GB | Yes |
| Llama 3.3 70B | 70B | 40 GB | 48 GB | Yes |
| Gemma 3 27B | 27B | 15 GB | 16 GB | Yes |
| Phi-4 14B | 14B | 8 GB | 8 GB | Yes |
| Qwen 3 8B | 8B | 4.5 GB | 8 GB | Yes |

Place `.gguf` files in the models directory, or use the built-in download manager.

## Configuration

Settings are accessible via the gear icon in the sidebar:

| Setting | Default | Description |
|---------|---------|-------------|
| TQ Bits | 3 (TQ3) | KV cache quantization — 3-bit (5.3x) or 4-bit (4x) |
| Context Length | 65,536 | Maximum tokens in context window |
| Temperature | 0.7 | Sampling temperature |
| GPU Layers | All (-1) | Number of layers offloaded to GPU |

The settings panel includes a live memory estimator showing KV cache size with and without TurboQuant compression.

## Benchmarks

Run the benchmark suite to compare TQ cache types:

```bash
bash scripts/benchmark-tq.sh ./src-tauri/llama-cpp/build/bin/llama-cli models/your-model.gguf
```

Tests needle-in-haystack retrieval across cache types (f16, q8_0, q4_0, turbo3, turbo4) and context lengths (4K to 128K).

## Roadmap

- [x] Phase 1: Tauri app scaffold with React UI and TQ core module
- [x] Phase 2: llama.cpp FFI bindings, model download manager
- [x] Phase 3: RAG pipeline with TQ-compressed vector index
- [x] Phase 4: Agent tool-calling framework with approval gates
- [x] Phase 5: Persistent memory and conversation search
- [ ] Phase 6: Cross-platform builds (Windows .exe, Linux AppImage, CUDA/Vulkan)
- [ ] Phase 7: Launch — benchmarks, open-source TQ library, landing page

## Development

```bash
# TypeScript type check
npx tsc --noEmit

# Production build (frontend only)
npx vite build

# Full Tauri build (requires Rust)
cargo tauri build
```

### Code Stats

| Metric | Count |
|--------|-------|
| Total source files | 55 |
| Rust files | 25 (2,942 lines) |
| TypeScript files | 16 (1,687 lines) |
| Total code | ~4,600 lines |
| Production JS bundle | 342 KB |
| Production CSS bundle | 16 KB |

## License

MIT
