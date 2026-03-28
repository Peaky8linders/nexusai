import { useAppStore } from "../../stores/appStore";

type ViewMode = "chat" | "rag" | "memory";

interface SidebarProps {
  onOpenSettings: () => void;
  viewMode: ViewMode;
  onChangeView: (mode: ViewMode) => void;
}

export function Sidebar({ onOpenSettings, viewMode, onChangeView }: SidebarProps) {
  const conversations = useAppStore((s) => s.conversations);
  const activeId = useAppStore((s) => s.activeConversationId);
  const setActive = useAppStore((s) => s.setActiveConversation);
  const createConversation = useAppStore((s) => s.createConversation);
  const deleteConversation = useAppStore((s) => s.deleteConversation);

  return (
    <aside className="w-64 border-r border-nexus-border bg-nexus-surface flex flex-col h-full">
      {/* Logo */}
      <div className="px-4 py-4 border-b border-nexus-border">
        <h1 className="text-lg font-extrabold tracking-tight">
          Nexus<span className="gradient-text">AI</span>
        </h1>
        <p className="text-[10px] font-mono text-nexus-dim uppercase tracking-widest mt-0.5">
          Local Agent Runtime
        </p>
      </div>

      {/* View tabs */}
      <div className="flex border-b border-nexus-border">
        {(["chat", "rag", "memory"] as const).map((mode) => (
          <button
            key={mode}
            onClick={() => onChangeView(mode)}
            className={`flex-1 py-2 text-[10px] font-mono uppercase tracking-wider transition-colors
                        ${viewMode === mode
                          ? "text-nexus-accent border-b-2 border-nexus-accent bg-nexus-accent/5"
                          : "text-nexus-dim hover:text-nexus-text"
                        }`}
          >
            {mode === "chat" ? "Chat" : mode === "rag" ? "RAG" : "Memory"}
          </button>
        ))}
      </div>

      {/* View-specific content */}
      {viewMode === "chat" && (
        <>
          {/* New Chat button */}
          <div className="px-3 py-3">
            <button
              onClick={() => createConversation("New Chat")}
              className="w-full px-3 py-2 text-sm text-nexus-accent bg-nexus-accent/5
                         border border-nexus-accent/20 rounded-lg hover:bg-nexus-accent/10
                         transition-colors font-mono text-xs tracking-wider uppercase"
            >
              + New Chat
            </button>
          </div>

          {/* Conversation list */}
          <div className="flex-1 overflow-y-auto px-3 space-y-1">
            {conversations.map((conv) => (
              <div
                key={conv.id}
                className={`group flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer
                            transition-colors text-sm ${
                              conv.id === activeId
                                ? "bg-nexus-accent/10 text-nexus-accent"
                                : "text-nexus-dim hover:bg-nexus-surface2 hover:text-nexus-text"
                            }`}
                onClick={() => setActive(conv.id)}
              >
                <span className="truncate text-xs">{conv.title}</span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    deleteConversation(conv.id);
                  }}
                  className="opacity-0 group-hover:opacity-100 text-nexus-dim hover:text-red-400
                             transition-opacity text-xs ml-2"
                >
                  x
                </button>
              </div>
            ))}
          </div>
        </>
      )}

      {viewMode === "rag" && (
        <div className="flex-1 px-3 py-3">
          <div className="text-xs text-nexus-dim mb-3">
            Indexed sources appear here. Drop files in the main panel to index them.
          </div>
          <div className="space-y-2">
            <RagStatRow label="Documents" value="0" />
            <RagStatRow label="Chunks" value="0" />
            <RagStatRow label="Index Size" value="0 MB" />
            <RagStatRow label="TQ Savings" value="~83%" />
          </div>
        </div>
      )}

      {viewMode === "memory" && (
        <div className="flex-1 px-3 py-3">
          <div className="text-xs text-nexus-dim mb-3">
            NexusAI remembers facts, preferences, and context across conversations.
          </div>
          <div className="space-y-2">
            <RagStatRow label="Stored Facts" value="0" />
            <RagStatRow label="Preferences" value="0" />
            <RagStatRow label="Summaries" value="0" />
          </div>
        </div>
      )}

      {/* Bottom actions */}
      <div className="px-3 py-3 border-t border-nexus-border space-y-1">
        <button
          onClick={onOpenSettings}
          className="w-full px-3 py-2 text-xs text-nexus-dim hover:text-nexus-text
                     hover:bg-nexus-surface2 rounded-lg transition-colors text-left font-mono"
        >
          Settings
        </button>
        <div className="px-3 py-1 text-[10px] font-mono text-nexus-dim/50">
          v0.1.0 alpha
        </div>
      </div>
    </aside>
  );
}

function RagStatRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between items-center px-2 py-1.5 rounded bg-nexus-surface2">
      <span className="text-[10px] font-mono text-nexus-dim">{label}</span>
      <span className="text-[10px] font-mono text-nexus-accent">{value}</span>
    </div>
  );
}
