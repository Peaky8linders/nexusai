import { useEffect, useRef } from "react";
import { MessageBubble } from "./MessageBubble";
import { ChatInput } from "./ChatInput";
import { useAppStore } from "../../stores/appStore";
import type { Message } from "../../types";

interface ChatViewProps {
  conversationId: string;
}

export function ChatView({ conversationId }: ChatViewProps) {
  const conversations = useAppStore((s) => s.conversations);
  const addMessage = useAppStore((s) => s.addMessage);
  const appendToLastMessage = useAppStore((s) => s.appendToLastMessage);
  const isGenerating = useAppStore((s) => s.isGenerating);
  const setGenerating = useAppStore((s) => s.setGenerating);
  const activeModelId = useAppStore((s) => s.activeModelId);

  const conversation = conversations.find((c) => c.id === conversationId);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [conversation?.messages]);

  const handleSend = async (content: string) => {
    if (!content.trim() || isGenerating) return;

    // Add user message
    const userMsg: Message = {
      id: crypto.randomUUID(),
      role: "user",
      content: content.trim(),
      timestamp: Date.now(),
    };
    addMessage(conversationId, userMsg);
    setGenerating(true);

    // Simulate streaming response (replaced by Tauri IPC in production)
    const assistantMsg: Message = {
      id: crypto.randomUUID(),
      role: "assistant",
      content: "",
      timestamp: Date.now(),
    };
    addMessage(conversationId, assistantMsg);

    // Stub: simulate token streaming
    const stubResponse = generateStubResponse(content, activeModelId);
    for (let i = 0; i < stubResponse.length; i += 3) {
      await new Promise((r) => setTimeout(r, 15));
      appendToLastMessage(conversationId, stubResponse.slice(i, i + 3));
    }

    setGenerating(false);
  };

  const handleStop = () => {
    setGenerating(false);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Chat header */}
      <div className="flex items-center justify-between px-6 py-2 border-b border-nexus-border bg-nexus-surface/30">
        <h2 className="font-semibold text-sm truncate">{conversation?.title ?? "Chat"}</h2>
        <span className="text-[10px] font-mono text-nexus-dim">
          {conversation?.messages.length ?? 0} messages
        </span>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
        {conversation?.messages.length === 0 && (
          <div className="flex items-center justify-center h-full text-nexus-dim text-sm">
            Send a message to start the conversation.
          </div>
        )}
        {conversation?.messages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <ChatInput
        onSend={handleSend}
        onStop={handleStop}
        isGenerating={isGenerating}
      />
    </div>
  );
}

function generateStubResponse(prompt: string, modelId: string | null): string {
  return `**NexusAI** (stub mode — no model loaded yet)

Model: \`${modelId ?? "none"}\` | TurboQuant TQ3 | Context: 64K tokens

Your prompt: "${prompt.slice(0, 80)}${prompt.length > 80 ? "..." : ""}"

---

This is a placeholder response. To get real inference:

1. Install Rust: \`rustup-init\`
2. Build: \`cargo tauri dev\`
3. Load a GGUF model from the model manager

The TurboQuant KV cache will compress your context window by ~5.3x, enabling 64K+ tokens on 16GB RAM.`;
}
