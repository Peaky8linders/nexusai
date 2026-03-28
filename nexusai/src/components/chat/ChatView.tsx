import { useEffect, useRef } from "react";
import { MessageBubble } from "./MessageBubble";
import { ChatInput } from "./ChatInput";
import { AgentPanel } from "../agent/AgentPanel";
import { useAppStore } from "../../stores/appStore";
import { isTauriApp } from "../../lib/tauri";
import { useToastStore } from "../Toast";
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
  const backendConnected = useAppStore((s) => s.backendConnected);
  const addToast = useToastStore((s) => s.addToast);

  const conversation = conversations.find((c) => c.id === conversationId);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const stopRef = useRef(false);
  const unlistenRef = useRef<(() => void) | null>(null);

  const messageCount = conversation?.messages.length ?? 0;
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messageCount]);

  // Set up stream listener when backend is connected
  useEffect(() => {
    if (!isTauriApp()) return;

    let cancelled = false;
    import("../../lib/tauri").then(({ listenToStream }) => {
      if (cancelled) return;
      listenToStream((chunk) => {
        if (chunk.done) {
          setGenerating(false);
          return;
        }
        appendToLastMessage(chunk.conversationId, chunk.content);
      }).then((unlisten) => {
        unlistenRef.current = unlisten;
      });
    });

    return () => {
      cancelled = true;
      unlistenRef.current?.();
    };
  }, [appendToLastMessage, setGenerating]);

  const handleSend = async (content: string) => {
    if (!content.trim() || useAppStore.getState().isGenerating) return;

    stopRef.current = false;

    const userMsg: Message = {
      id: crypto.randomUUID(),
      role: "user",
      content: content.trim(),
      timestamp: Date.now(),
    };
    addMessage(conversationId, userMsg);
    setGenerating(true);

    const assistantMsg: Message = {
      id: crypto.randomUUID(),
      role: "assistant",
      content: "",
      timestamp: Date.now(),
    };
    addMessage(conversationId, assistantMsg);

    if (backendConnected) {
      // Use Tauri IPC — streaming comes via events
      try {
        const { sendMessage } = await import("../../lib/tauri");
        await sendMessage(conversationId, content.trim(), activeModelId ?? undefined);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendToLastMessage(conversationId, `**Error:** ${msg}`);
        addToast("error", `Send failed: ${msg}`);
        setGenerating(false);
      }
    } else {
      // Stub mode — simulate streaming locally
      const stubResponse = generateStubResponse(content, activeModelId);
      for (let i = 0; i < stubResponse.length; i += 3) {
        if (stopRef.current) break;
        await new Promise((r) => setTimeout(r, 15));
        if (stopRef.current) break;
        appendToLastMessage(conversationId, stubResponse.slice(i, i + 3));
      }
      setGenerating(false);
    }
  };

  const handleStop = async () => {
    stopRef.current = true;
    setGenerating(false);
    if (backendConnected) {
      try {
        const { stopGeneration } = await import("../../lib/tauri");
        await stopGeneration();
      } catch {
        // ignore
      }
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Chat header */}
      <div className="flex items-center justify-between px-6 py-2 border-b border-nexus-border bg-nexus-surface/30">
        <h2 className="font-semibold text-sm truncate">{conversation?.title ?? "Chat"}</h2>
        <div className="flex items-center gap-2">
          {backendConnected && (
            <span className="text-[9px] font-mono px-1.5 py-0.5 rounded-full bg-nexus-accent/10 text-nexus-accent border border-nexus-accent/20">
              Live
            </span>
          )}
          <span className="text-[10px] font-mono text-nexus-dim">
            {messageCount} messages
          </span>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
        {messageCount === 0 && (
          <div className="flex items-center justify-center h-full text-nexus-dim text-sm">
            Send a message to start the conversation.
          </div>
        )}
        {conversation?.messages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Agent panel (shown when tool calls are active) */}
      <AgentPanel
        steps={[]}
        onApprove={() => {}}
        onReject={() => {}}
        onApproveAll={() => {}}
      />

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
