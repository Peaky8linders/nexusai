import ReactMarkdown from "react-markdown";
import type { Message } from "../../types";

interface MessageBubbleProps {
  message: Message;
}

export function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === "user";

  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
      <div
        className={`max-w-[80%] rounded-xl px-4 py-3 ${
          isUser
            ? "bg-nexus-accent/10 border border-nexus-accent/20 text-nexus-text"
            : "bg-nexus-surface border border-nexus-border text-nexus-text"
        }`}
      >
        {isUser ? (
          <p className="text-sm whitespace-pre-wrap">{message.content}</p>
        ) : (
          <div className="prose prose-invert prose-sm max-w-none text-sm
                          prose-headings:text-nexus-accent prose-strong:text-nexus-text
                          prose-code:text-nexus-accent2 prose-code:bg-nexus-surface2
                          prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded
                          prose-code:font-mono prose-code:text-xs
                          prose-pre:bg-nexus-surface2 prose-pre:border prose-pre:border-nexus-border
                          prose-a:text-nexus-accent2 prose-a:no-underline hover:prose-a:underline">
            <ReactMarkdown>{message.content}</ReactMarkdown>
          </div>
        )}

        {message.tokensPerSecond && (
          <div className="mt-2 text-[10px] font-mono text-nexus-dim">
            {message.tokensPerSecond.toFixed(1)} tok/s
          </div>
        )}
      </div>
    </div>
  );
}
