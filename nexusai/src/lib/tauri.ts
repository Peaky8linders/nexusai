/**
 * Tauri IPC bridge — wraps @tauri-apps/api invoke calls.
 *
 * In development without Tauri (plain vite dev server), these functions
 * return stub data so the UI can be developed independently.
 *
 * All types use camelCase to match frontend conventions.
 * The Rust backend uses #[serde(rename_all = "camelCase")] so the
 * wire format matches automatically — no mapping layer needed.
 */

import type { Conversation, ModelInfo, AppSettings, SystemInfo, StreamChunk } from "../types";

const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(cmd, args);
  }
  throw new Error(`Tauri not available — running in browser stub mode`);
}

/** Check if we're running inside Tauri */
export function isTauriApp(): boolean {
  return isTauri;
}

// ── Chat commands ──────────────────────────────────────────────

export async function sendMessage(conversationId: string, content: string, modelId?: string) {
  return invoke<void>("send_message", {
    request: { conversationId, content, modelId },
  });
}

export async function stopGeneration() {
  return invoke<void>("stop_generation");
}

export async function getConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>("get_conversations");
}

export async function createConversation(title: string, modelId: string): Promise<Conversation> {
  return invoke<Conversation>("create_conversation", { title, modelId });
}

export async function deleteConversation(id: string) {
  return invoke<void>("delete_conversation", { id });
}

// ── Model commands ─────────────────────────────────────────────

export async function listModels(): Promise<ModelInfo[]> {
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

// ── Settings commands ──────────────────────────────────────────

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: AppSettings) {
  return invoke<void>("update_settings", { settings });
}

// ── System commands ────────────────────────────────────────────

export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>("get_system_info");
}

// ── Event listening (for streaming) ────────────────────────────

export async function listenToStream(
  callback: (chunk: StreamChunk) => void,
): Promise<() => void> {
  if (isTauri) {
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<StreamChunk>("chat:stream", (event) => {
      callback(event.payload);
    });
    return unlisten;
  }
  return () => {};
}
