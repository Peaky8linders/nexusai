import { useState } from "react";
import { Sidebar } from "./components/sidebar/Sidebar";
import { ChatView } from "./components/chat/ChatView";
import { SettingsPanel } from "./components/settings/SettingsPanel";
import { ModelSelector } from "./components/settings/ModelSelector";
import { RagExplorer } from "./components/rag/RagExplorer";
import { MemoryPanel } from "./components/settings/MemoryPanel";
import { useAppStore } from "./stores/appStore";

type ViewMode = "chat" | "rag" | "memory";

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [showModelSelector, setShowModelSelector] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>("chat");
  const activeConversation = useAppStore((s) => s.activeConversationId);

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar
        onOpenSettings={() => setShowSettings(true)}
        viewMode={viewMode}
        onChangeView={setViewMode}
      />
      <main className="flex-1 flex flex-col min-w-0">
        {/* Top bar with model selector */}
        <TopBar
          onOpenModelSelector={() => setShowModelSelector(true)}
          viewMode={viewMode}
        />

        {viewMode === "chat" && (
          activeConversation ? (
            <ChatView conversationId={activeConversation} />
          ) : (
            <WelcomeScreen />
          )
        )}

        {viewMode === "rag" && (
          <RagExplorer
            files={[]}
            totalChunks={0}
            indexSizeMb={0}
            tqSavingsPercent={0}
            onDropFiles={() => {}}
            onRemoveFile={() => {}}
            onReindex={() => {}}
          />
        )}

        {viewMode === "memory" && <MemoryPanel />}
      </main>

      {showSettings && (
        <SettingsPanel onClose={() => setShowSettings(false)} />
      )}
      {showModelSelector && (
        <ModelSelector onClose={() => setShowModelSelector(false)} />
      )}
    </div>
  );
}

function TopBar({
  onOpenModelSelector,
}: {
  onOpenModelSelector: () => void;
  viewMode: ViewMode;
}) {
  const activeModelId = useAppStore((s) => s.activeModelId);

  return (
    <div className="flex items-center justify-between px-4 py-2 border-b border-nexus-border bg-nexus-surface/30">
      <div className="flex items-center gap-3">
        <button
          onClick={onOpenModelSelector}
          className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-mono
                     bg-nexus-surface2 border border-nexus-border hover:border-nexus-accent/30
                     text-nexus-dim hover:text-nexus-text transition-colors"
        >
          <span className="w-2 h-2 rounded-full bg-nexus-accent/50" />
          {activeModelId || "Select Model"}
          <span className="text-[10px] opacity-50">v</span>
        </button>
      </div>

      <div className="flex items-center gap-2">
        <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-nexus-accent/10 text-nexus-accent border border-nexus-accent/20">
          TQ3 Active
        </span>
        <span className="text-[10px] font-mono text-nexus-dim">
          64K ctx
        </span>
      </div>
    </div>
  );
}

function WelcomeScreen() {
  const createConversation = useAppStore((s) => s.createConversation);

  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center max-w-md">
        <h1 className="text-4xl font-extrabold mb-2">
          Nexus<span className="gradient-text">AI</span>
        </h1>
        <p className="text-nexus-dim mb-8 text-sm">
          Local-first AI agent runtime powered by TurboQuant compression.
          <br />
          Your models. Your hardware. Your data.
        </p>

        <div className="grid grid-cols-2 gap-3 mb-8 text-left">
          <StatCard label="KV Compression" value="5.3x" sub="TurboQuant TQ3" />
          <StatCard label="Context Window" value="64K+" sub="On 16GB RAM" />
          <StatCard label="Per-Token Cost" value="$0" sub="100% local" />
          <StatCard label="Privacy" value="Full" sub="No cloud calls" />
        </div>

        <button
          onClick={() => createConversation("New Chat")}
          className="px-6 py-3 bg-nexus-accent/10 text-nexus-accent border border-nexus-accent/20
                     rounded-lg hover:bg-nexus-accent/20 transition-colors font-mono text-sm
                     tracking-wider uppercase"
        >
          Start Chat
        </button>
      </div>
    </div>
  );
}

function StatCard({ label, value, sub }: { label: string; value: string; sub: string }) {
  return (
    <div className="bg-nexus-surface border border-nexus-border rounded-lg p-4">
      <div className="text-xs font-mono text-nexus-dim uppercase tracking-wider mb-1">
        {label}
      </div>
      <div className="text-2xl font-extrabold gradient-text">{value}</div>
      <div className="text-xs text-nexus-dim">{sub}</div>
    </div>
  );
}

export default App;
