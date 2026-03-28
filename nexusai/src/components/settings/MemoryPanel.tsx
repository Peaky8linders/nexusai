import { useState } from "react";

interface MemoryEntry {
  id: string;
  key: string;
  value: string;
  category: "fact" | "preference" | "context" | "summary";
  createdAt: string;
  updatedAt: string;
}

const STUB_MEMORIES: MemoryEntry[] = [];

export function MemoryPanel() {
  const [memories, setMemories] = useState<MemoryEntry[]>(STUB_MEMORIES);
  const [searchQuery, setSearchQuery] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");

  const filteredMemories = memories.filter(
    (m) =>
      m.key.toLowerCase().includes(searchQuery.toLowerCase()) ||
      m.value.toLowerCase().includes(searchQuery.toLowerCase()),
  );

  const categoryColors: Record<string, string> = {
    fact: "text-nexus-accent border-nexus-accent/20 bg-nexus-accent/5",
    preference: "text-nexus-accent2 border-nexus-accent2/20 bg-nexus-accent2/5",
    context: "text-nexus-accent3 border-nexus-accent3/20 bg-nexus-accent3/5",
    summary: "text-nexus-warn border-nexus-warn/20 bg-nexus-warn/5",
  };

  const handleDelete = (id: string) => {
    setMemories((prev) => prev.filter((m) => m.id !== id));
  };

  const handleSaveEdit = (id: string) => {
    setMemories((prev) =>
      prev.map((m) => (m.id === id ? { ...m, value: editValue } : m)),
    );
    setEditingId(null);
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="px-6 py-4 border-b border-nexus-border">
        <h2 className="text-lg font-bold mb-1">Memory Store</h2>
        <p className="text-xs text-nexus-dim">
          NexusAI remembers facts, preferences, and context from your conversations.
          Memories persist across sessions and are injected into prompts for continuity.
        </p>
      </div>

      {/* Search and stats */}
      <div className="px-6 py-3 border-b border-nexus-border flex items-center gap-4">
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search memories..."
          className="flex-1 bg-nexus-surface2 border border-nexus-border rounded-lg px-3 py-2
                     text-xs text-nexus-text placeholder:text-nexus-dim
                     focus:outline-none focus:border-nexus-accent/40 transition-colors"
        />
        <div className="flex gap-3 text-[10px] font-mono text-nexus-dim">
          <span>{memories.length} total</span>
          <span>{memories.filter((m) => m.category === "fact").length} facts</span>
          <span>{memories.filter((m) => m.category === "preference").length} prefs</span>
        </div>
      </div>

      {/* Memory list */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {filteredMemories.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="text-3xl mb-3 opacity-20">~</div>
              <p className="text-nexus-dim text-sm mb-1">
                {memories.length === 0
                  ? "No memories yet"
                  : "No memories match your search"}
              </p>
              <p className="text-nexus-dim/50 text-xs">
                {memories.length === 0
                  ? "As you chat, NexusAI will learn facts and preferences automatically"
                  : "Try a different search term"}
              </p>
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            {filteredMemories.map((memory) => (
              <div
                key={memory.id}
                className="group bg-nexus-surface border border-nexus-border rounded-lg px-4 py-3
                           hover:border-nexus-accent/20 transition-colors"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span
                        className={`text-[9px] font-mono uppercase px-1.5 py-0.5 rounded-full border
                                    ${categoryColors[memory.category] ?? ""}`}
                      >
                        {memory.category}
                      </span>
                      <span className="text-xs font-semibold text-nexus-text truncate">
                        {memory.key}
                      </span>
                    </div>

                    {editingId === memory.id ? (
                      <div className="flex gap-2 mt-1">
                        <input
                          type="text"
                          value={editValue}
                          onChange={(e) => setEditValue(e.target.value)}
                          className="flex-1 bg-nexus-surface2 border border-nexus-accent/30 rounded px-2 py-1
                                     text-xs text-nexus-text focus:outline-none"
                          autoFocus
                        />
                        <button
                          onClick={() => handleSaveEdit(memory.id)}
                          className="text-[10px] font-mono px-2 py-1 rounded bg-nexus-accent/20 text-nexus-accent"
                        >
                          Save
                        </button>
                        <button
                          onClick={() => setEditingId(null)}
                          className="text-[10px] font-mono px-2 py-1 rounded text-nexus-dim"
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <p className="text-xs text-nexus-dim">{memory.value}</p>
                    )}
                  </div>

                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => {
                        setEditingId(memory.id);
                        setEditValue(memory.value);
                      }}
                      className="text-[10px] font-mono text-nexus-dim hover:text-nexus-accent transition-colors"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => handleDelete(memory.id)}
                      className="text-[10px] font-mono text-nexus-dim hover:text-red-400 transition-colors"
                    >
                      Del
                    </button>
                  </div>
                </div>

                <div className="text-[9px] font-mono text-nexus-dim/50 mt-1">
                  Created {memory.createdAt} | Updated {memory.updatedAt}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
