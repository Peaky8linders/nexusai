import { useState, useRef, useEffect } from "react";

interface ChatInputProps {
  onSend: (content: string) => void;
  onStop: () => void;
  isGenerating: boolean;
}

export function ChatInput({ onSend, onStop, isGenerating }: ChatInputProps) {
  const [input, setInput] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height =
        Math.min(textareaRef.current.scrollHeight, 200) + "px";
    }
  }, [input]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (input.trim()) {
        onSend(input);
        setInput("");
      }
    }
  };

  return (
    <div className="border-t border-nexus-border bg-nexus-surface/50 backdrop-blur px-6 py-4">
      <div className="flex items-end gap-3 max-w-3xl mx-auto">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Send a message... (Shift+Enter for newline)"
          disabled={isGenerating}
          rows={1}
          className="flex-1 bg-nexus-surface2 border border-nexus-border rounded-lg px-4 py-3
                     text-sm text-nexus-text placeholder:text-nexus-dim resize-none
                     focus:outline-none focus:border-nexus-accent/40 transition-colors
                     disabled:opacity-50"
        />
        {isGenerating ? (
          <button
            onClick={onStop}
            className="px-4 py-3 bg-red-500/10 text-red-400 border border-red-500/20
                       rounded-lg hover:bg-red-500/20 transition-colors text-sm font-mono"
          >
            Stop
          </button>
        ) : (
          <button
            onClick={() => {
              if (input.trim()) {
                onSend(input);
                setInput("");
              }
            }}
            disabled={!input.trim()}
            className="px-4 py-3 bg-nexus-accent/10 text-nexus-accent border border-nexus-accent/20
                       rounded-lg hover:bg-nexus-accent/20 transition-colors text-sm font-mono
                       disabled:opacity-30 disabled:cursor-not-allowed"
          >
            Send
          </button>
        )}
      </div>
      <div className="flex items-center justify-center gap-4 mt-2 text-[10px] font-mono text-nexus-dim">
        <span>TQ3 KV Cache</span>
        <span>64K Context</span>
        <span>Local Inference</span>
      </div>
    </div>
  );
}
