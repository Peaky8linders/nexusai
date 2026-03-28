/**
 * Tauri IPC bridge — wraps @tauri-apps/api invoke calls.
 *
 * In development without Tauri (plain vite dev server), these functions
 * return stub data so the UI can be developed independently.
 */

const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(cmd, args);
  }
  // Stub mode — return mock data for UI development
  console.log(`[stub] invoke(${cmd})`, args);
  throw new Error(`Tauri not available — running in browser stub mode`);
}

// Chat commands
export async function sendMessage(conversationId: string, content: string, modelId?: string) {
  return invoke<void>("send_message", {
    request: { conversation_id: conversationId, content, model_id: modelId },
  });
}

export async function stopGeneration() {
  return invoke<void>("stop_generation");
}

export async function getConversations() {
  return invoke<Conversation[]>("get_conversations");
}

export async function createConversation(title: string, modelId: string) {
  return invoke<Conversation>("create_conversation", { title, modelId });
}

export async function deleteConversation(id: string) {
  return invoke<void>("delete_conversation", { id });
}

// Model commands
export interface ModelInfo {
  id: string;
  name: string;
  size_bytes: number;
  quantization: string;
  parameters: string;
  family: string;
  downloaded: boolean;
  loaded: boolean;
  tq_compatible: boolean;
}

export interface Conversation {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
  model_id: string;
  messages: Array<{
    id: string;
    role: string;
    content: string;
    timestamp: number;
  }>;
}

export async function listModels() {
  return invoke<ModelInfo[]>("list_models");
}

export async function downloadModel(modelId: string) {
  return invoke<void>("download_model", { modelId });
}

export async function loadModel(modelId: string, tqBits?: number) {
  return invoke<void>("load_model", { modelId, tqBits });
}

export async function unloadModel() {
  return invoke<void>("unload_model");
}

// Settings
export interface AppSettings {
  default_model: string | null;
  tq_kv_bits: number;
  context_length: number;
  temperature: number;
  top_p: number;
  max_tokens: number;
  gpu_layers: number;
  theme: string;
  stream_response: boolean;
  show_token_stats: boolean;
}

export async function getSettings() {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: AppSettings) {
  return invoke<void>("update_settings", { settings });
}

// System
export interface SystemInfo {
  os: string;
  arch: string;
  total_memory_gb: number;
  gpu_name: string | null;
  gpu_memory_gb: number | null;
  metal_supported: boolean;
  cuda_supported: boolean;
  recommended_context: number;
  recommended_tq_bits: number;
}

export async function getSystemInfo() {
  return invoke<SystemInfo>("get_system_info");
}

// Event listening (for streaming)
export async function listenToStream(
  callback: (chunk: { conversation_id: string; content: string; done: boolean; tokens_per_second: number | null }) => void,
) {
  if (isTauri) {
    const { listen } = await import("@tauri-apps/api/event");
    return listen("chat:stream", (event) => {
      callback(event.payload as Parameters<typeof callback>[0]);
    });
  }
  return () => {}; // no-op unsubscribe in stub mode
}
