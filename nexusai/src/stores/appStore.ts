import { create } from "zustand";
import type { Conversation, Message, ModelInfo } from "../types";

interface AppState {
  // Conversations
  conversations: Conversation[];
  activeConversationId: string | null;
  isGenerating: boolean;

  // Models
  models: ModelInfo[];
  activeModelId: string | null;

  // Actions
  createConversation: (title: string) => void;
  setActiveConversation: (id: string | null) => void;
  deleteConversation: (id: string) => void;
  addMessage: (conversationId: string, message: Message) => void;
  appendToLastMessage: (conversationId: string, content: string) => void;
  setGenerating: (generating: boolean) => void;
  setModels: (models: ModelInfo[]) => void;
  setActiveModel: (id: string | null) => void;
  updateConversationTitle: (id: string, title: string) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  isGenerating: false,
  models: [],
  activeModelId: null,

  createConversation: (title: string) => {
    const id = crypto.randomUUID();
    const now = Date.now();
    const conversation: Conversation = {
      id,
      title,
      createdAt: now,
      updatedAt: now,
      modelId: get().activeModelId ?? "",
      messages: [],
    };
    set((s) => ({
      conversations: [conversation, ...s.conversations],
      activeConversationId: id,
    }));
  },

  setActiveConversation: (id) => set({ activeConversationId: id }),

  deleteConversation: (id) =>
    set((s) => ({
      conversations: s.conversations.filter((c) => c.id !== id),
      activeConversationId:
        s.activeConversationId === id ? null : s.activeConversationId,
    })),

  addMessage: (conversationId, message) =>
    set((s) => ({
      conversations: s.conversations.map((c) =>
        c.id === conversationId
          ? { ...c, messages: [...c.messages, message], updatedAt: Date.now() }
          : c,
      ),
    })),

  appendToLastMessage: (conversationId, content) =>
    set((s) => ({
      conversations: s.conversations.map((c) => {
        if (c.id !== conversationId || c.messages.length === 0) return c;
        const msgs = [...c.messages];
        const last = msgs[msgs.length - 1];
        msgs[msgs.length - 1] = { ...last, content: last.content + content };
        return { ...c, messages: msgs };
      }),
    })),

  setGenerating: (generating) => set({ isGenerating: generating }),

  setModels: (models) => set({ models }),

  setActiveModel: (id) => set({ activeModelId: id }),

  updateConversationTitle: (id, title) =>
    set((s) => ({
      conversations: s.conversations.map((c) =>
        c.id === id ? { ...c, title } : c,
      ),
    })),
}));
