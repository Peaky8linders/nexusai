# NexusAI

**TurboQuant-powered local-first AI agent runtime.**

Run 35B+ parameter models with 64K+ context windows on consumer hardware. No cloud. No API keys. Your models, your hardware, your data.

[![CI](https://github.com/Peaky8linders/nexusai/actions/workflows/ci.yml/badge.svg)](https://github.com/Peaky8linders/nexusai/actions/workflows/ci.yml)

## What is this?

NexusAI is a desktop application that runs large language models locally using [TurboQuant](https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression/) — Google's KV cache compression algorithm that achieves **5.3x memory reduction** without meaningful quality loss.

A model that normally needs 48 GB of RAM for a 64K context window now fits in 16 GB.

```
Traditional KV Cache (FP16):  12.8 GB for 64K context on a 35B model
TurboQuant TQ3 (3-bit):        2.4 GB for the same setup
                              ─────────
                              5.3x smaller
```

## Features

- **Chat UI** — Streaming responses, markdown rendering, conversation management, error boundary with recovery
- **Model Manager** — Curated catalog of GGUF models (Qwen, Llama, Gemma, Phi) with TurboQuant compatibility flags
- **TurboQuant KV Cache** — TQ3 (5.3×) and TQ4 (4×) compression modes, configurable per-session
- **RAG Pipeline** — Drag-drop document indexing with structure-aware chunking and TQ-compressed vector search
- **Agent Framework** — Multi-format tool call parsing (Llama 3.x, Qwen/ChatML, generic JSON), approval gates, multi-step execution
- **Persistent Memory** — SQLite-backed store for user facts, preferences, and conversation history with full-text search
- **MCP Support** — Model Context Protocol plugin registry for extensible tool ecosystem
- **Accessibility** — Full keyboard navigation, focus trap in modals, ARIA live regions, Escape-to-close

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
| Desktop | Tauri v2 (5 MB binary vs 150 MB Electron) |
| Backend | Rust, SQLite (rusqlite), tokio async |
| Inference | llama.cpp fork with TurboQuant GGML types |
| Embeddings | nomic-embed-text-v1.5 (local) |
| Compression | TurboQuant TQ3/TQ4 (Walsh-Hadamard + Lloyd-Max + QJL) |

---

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) stable
- 16 GB RAM recommended (8 GB minimum for Phi-4 14B)

### 1. Clone and install

```bash
git clone https://github.com/Peaky8linders/nexusai.git
cd nexusai/nexusai
npm install
```

### 2. Frontend-only mode (no Rust required)

The UI runs fully standalone with simulated streaming responses — useful for UI development or just exploring the interface:

```bash
npm run dev
# Open http://localhost:1420
```

All views (Chat, RAG, Memory, Benchmark) are functional. The "Live" badge in the chat header is absent in this mode, and responses come from a local stub.

### 3. Full Tauri app (real inference)

```bash
# Build llama.cpp with TurboQuant KV cache support
bash scripts/setup-llama-cpp.sh

# Run development build with hot reload
cargo tauri dev

# Production build → native installer
cargo tauri build
```

Output: `.dmg` on macOS, `.msi`/`.exe` on Windows, `.AppImage` on Linux.

---

## Usage Guide

### Chat

1. Click **+ New Chat** in the sidebar (or press `Ctrl+N` / `Cmd+N`)
2. Select a model by clicking the model name at the top of the sidebar — the **Select Model** dialog shows all available models with their RAM requirements
3. Type a message and press `Enter` to send (`Shift+Enter` for newlines)
4. Click **Stop** to interrupt generation at any time
5. Conversations are auto-saved to SQLite. They persist across restarts when running in full Tauri mode.

**Keyboard shortcuts:**

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + N` | New conversation |
| `Ctrl/Cmd + ,` | Open Settings |
| `Ctrl/Cmd + M` | Open Model Selector |
| `Escape` | Close open modal |

### Model Selection

Open the **Select Model** dialog (`Ctrl+M`) to switch between:

| Model | Active Parameters | Disk Size | Min RAM |
|-------|-----------------|-----------|---------|
| Qwen 3.5 35B-A3B | 3B (MoE) | 19 GB | 16 GB |
| Llama 3.3 70B | 70B | 40 GB | 48 GB |
| Gemma 3 27B | 27B | 15 GB | 16 GB |
| Phi-4 14B | 14B | 8 GB | 8 GB |
| Qwen 3 8B | 8B | 4.5 GB | 8 GB |

Place `.gguf` files in the app data `models/` directory. The path is printed to the console on startup.

### RAG (Retrieval-Augmented Generation)

Switch to the **RAG** tab in the sidebar:

1. Drag and drop files onto the drop zone (`.txt`, `.md`, `.pdf`, `.docx`)
2. Files are chunked with structure-aware splitting and indexed
3. The search bar lets you query the index directly
4. When a model is loaded, retrieved context is automatically injected into the system prompt

### Memory

Switch to the **Memory** tab to view facts and preferences the model has extracted from your conversations. Entries are stored per-category (`fact`, `preference`, `general`) and injected into the system prompt on each turn.

### Settings

Open the gear icon or press `Ctrl+,`:

| Setting | Default | Notes |
|---------|---------|-------|
| TQ Bits | 3 (TQ3) | `3` = 5.3× compression, `4` = 4× compression |
| Context Length | 65,536 | Reduce if you run out of RAM |
| Temperature | 0.7 | Higher = more creative, lower = more deterministic |
| GPU Layers | All (−1) | Set to `0` to force CPU-only inference |

The live memory estimator in Settings shows exact KV cache RAM usage before and after TurboQuant compression for your current configuration.

### Benchmark

Switch to the **Benchmark** tab to see the quality/memory trade-off matrix across all TQ cache modes and context lengths.

Run the full needle-in-haystack benchmark suite from the terminal:

```bash
bash scripts/benchmark-tq.sh \
  ./src-tauri/llama-cpp/build/bin/llama-cli \
  models/your-model.gguf
```

---

## How TurboQuant Works

TurboQuant compresses the KV cache — the memory bottleneck that limits context length in transformer models.

**Step 1 — Random Rotation**: Apply a Walsh-Hadamard Transform (WHT) to each key/value vector. After rotation, each coordinate follows a predictable Beta distribution regardless of model architecture.

**Step 2 — Scalar Quantization**: Pre-computed Lloyd-Max codebooks (computed once at startup) quantize each coordinate from 16 bits down to 3 bits (TQ3) or 4 bits (TQ4).

**Step 3 — QJL Residual**: A 1-bit Quantized Johnson-Lindenstrauss projection corrects the quantization error, ensuring unbiased inner product estimation for attention scores.

The result is **online, calibration-free** compression — no training data, no warm-up, works on any GGUF model out of the box.

---

## Project Structure

```
nexusai/
├── src/                          # React frontend (TypeScript)
│   ├── components/
│   │   ├── chat/                 # ChatView, MessageBubble, ChatInput
│   │   ├── agent/                # AgentPanel (tool call approval UI)
│   │   ├── rag/                  # RagExplorer (drag-drop document indexing)
│   │   ├── settings/             # SettingsPanel, ModelSelector, MemoryPanel,
│   │   │                         #   BenchmarkDashboard
│   │   ├── sidebar/              # Sidebar with nav tabs + conversation list
│   │   ├── ErrorBoundary.tsx     # React error boundary with recovery
│   │   └── Toast.tsx             # Toast notification system
│   ├── hooks/
│   │   ├── useKeyboard.ts        # Global shortcuts + focus trap
│   │   └── useConversationSearch.ts
│   ├── stores/appStore.ts        # Zustand state — conversations, models, backend
│   ├── lib/
│   │   ├── tauri.ts              # Tauri IPC bridge (invoke + event stream)
│   │   └── export.ts             # Conversation export utilities
│   └── types/index.ts            # Shared TypeScript types
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/             # Tauri IPC handlers
│   │   │   ├── chat.rs           # send_message, streaming, conversation CRUD
│   │   │   ├── models.rs         # list_models, load_model, download_model
│   │   │   ├── settings.rs       # get_settings, update_settings
│   │   │   └── system.rs         # get_system_info (RAM, GPU, recommended config)
│   │   ├── inference/
│   │   │   ├── mod.rs            # InferenceEngine stub (llama.cpp FFI target)
│   │   │   ├── llama_ffi.rs      # Safe Rust wrappers for llama.cpp C API
│   │   │   ├── download.rs       # HuggingFace downloader with resume + SHA256
│   │   │   └── tq_config.rs      # TurboQuantConfig, compression math, unit tests
│   │   ├── rag/
│   │   │   ├── mod.rs            # RagPipeline: ingest, search, build_context
│   │   │   ├── chunker.rs        # Structure-aware paragraph/line/word chunking
│   │   │   ├── embedder.rs       # Embedding stub (nomic-embed-text target)
│   │   │   ├── index.rs          # VectorIndex: keyword search + cosine stub
│   │   │   └── parsers.rs        # File parsers: plain text, Markdown, PDF stub
│   │   ├── tools/
│   │   │   ├── mod.rs            # Tool executor (read_file, list_directory)
│   │   │   ├── agent.rs          # AgentSession execution loop
│   │   │   └── parser.rs         # Multi-format tool call parser
│   │   ├── memory/
│   │   │   ├── mod.rs            # SQLite schema, conversation/message CRUD
│   │   │   └── search.rs         # Full-text search, memory store/retrieve
│   │   ├── mcp/mod.rs            # MCP server registry
│   │   ├── models/mod.rs         # GGUF model catalog helpers
│   │   └── lib.rs                # Tauri app setup, plugin init, state management
│   └── capabilities/default.json # Tauri v2 permission manifest
│
├── scripts/
│   ├── setup-llama-cpp.sh        # Clone + patch + build llama.cpp with TQ support
│   └── benchmark-tq.sh           # Needle-in-haystack benchmark suite
│
└── public/
    └── landing.html              # Self-contained marketing landing page
```

---

## Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| React UI | Complete | All views functional, fully typed |
| Tauri IPC | Complete | Frontend ↔ Rust wired, streaming events |
| SQLite persistence | Complete | Conversations, messages, memory |
| Conversation management | Complete | Create, delete, search, export |
| Keyboard shortcuts | Complete | Ctrl+N, Ctrl+,, Ctrl+M, Escape |
| Error handling | Complete | Error boundary, toast notifications |
| Model catalog | Complete | 5 models listed, hardcoded |
| llama.cpp inference | Stub | Returns placeholder text; real FFI ready to wire |
| Model download | Stub | HuggingFace downloader code exists; IPC not wired |
| RAG embeddings | Stub | Zero-vector fallback; nomic-embed-text target |
| MCP plugins | Stub | Registry scaffolded; no plugins registered |
| GPU detection | Stub | Hardcoded RAM values; sysinfo integration pending |

---

## Next Steps

### Phase 1 — Wire real inference (highest impact)

The llama.cpp FFI bindings are fully written in `src-tauri/src/inference/llama_ffi.rs`. The remaining work is linking the C library:

1. **Build the TurboQuant llama.cpp fork**
   ```bash
   bash scripts/setup-llama-cpp.sh
   ```
   The script handles cloning, patching, and building. The TQ GGML types (`TQ3_0`, `TQ4_0`) map directly to `KvCacheType::Turbo3/Turbo4` in the FFI layer.

2. **Uncomment the FFI calls** in `llama_ffi.rs`
   All actual `llama_*` calls are present but commented out in the `generate()` and `load_model()` bodies. Uncomment them and add the C library to `Cargo.toml` build deps.

3. **Update `InferenceEngine::generate`** in `inference/mod.rs`
   Replace the stub `Ok(format!(...))` with a call to `LlamaSession::generate()` passing the real message history.

### Phase 2 — Real embeddings for RAG

Replace `src-tauri/src/rag/embedder.rs`'s zero-vector stub with nomic-embed-text-v1.5 via the same llama.cpp context. The index already supports cosine similarity; it just needs non-zero vectors.

### Phase 3 — Model download manager

`src-tauri/src/inference/download.rs` has a complete HuggingFace downloader with range-request resume, progress callbacks, and SHA256 verification. Wire it to the `download_model` Tauri command (currently returns an error stub) and emit `download:progress` events.

### Phase 4 — Settings persistence

`src-tauri/src/commands/settings.rs` returns hardcoded defaults. Persist to a JSON file in the app data directory using `serde_json`. The frontend `SettingsPanel` already reads and displays these values.

### Phase 5 — Real system info

`src-tauri/src/commands/system.rs` returns hardcoded RAM values. Add the [`sysinfo`](https://crates.io/crates/sysinfo) crate to detect actual system memory and use that to drive `recommended_context` and `recommended_tq_bits`.

### Phase 6 — GPU detection

Extend `system.rs` with actual GPU queries:
- **macOS**: `Metal::MTLCopyAllDevices()` via objc crate
- **Windows/Linux with Nvidia**: `nvml-wrapper` crate
- **Vulkan fallback**: `ash` crate

---

## Development

```bash
# TypeScript type check (must pass before commit)
npx tsc --noEmit

# Frontend-only dev server
npm run dev

# Rust check (must pass before commit)
cargo check

# Full Tauri dev build
cargo tauri dev

# Production build
cargo tauri build
```

### Code Stats

| Metric | Count |
|--------|-------|
| TypeScript/TSX files | 20 |
| Rust files | 23 |
| Total source lines | ~5,200 |
| Production JS bundle | ~342 KB |

---

## License

MIT
