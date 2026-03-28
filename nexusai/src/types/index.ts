export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  tokensPerSecond?: number;
}

export interface Conversation {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
  modelId: string;
  messages: Message[];
}

export interface ModelInfo {
  id: string;
  name: string;
  sizeBytes: number;
  quantization: string;
  parameters: string;
  family: string;
  downloaded: boolean;
  loaded: boolean;
  tqCompatible: boolean;
}

export interface StreamChunk {
  conversation_id: string;
  content: string;
  done: boolean;
  tokens_per_second: number | null;
}

export interface AppSettings {
  defaultModel: string | null;
  tqKvBits: 3 | 4;
  contextLength: number;
  temperature: number;
  topP: number;
  maxTokens: number;
  gpuLayers: number;
  theme: "dark" | "light";
  streamResponse: boolean;
  showTokenStats: boolean;
}

export interface SystemInfo {
  os: string;
  arch: string;
  totalMemoryGb: number;
  gpuName: string | null;
  gpuMemoryGb: number | null;
  metalSupported: boolean;
  cudaSupported: boolean;
  recommendedContext: number;
  recommendedTqBits: number;
}
