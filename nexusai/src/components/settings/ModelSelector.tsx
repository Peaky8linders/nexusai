import { useAppStore } from "../../stores/appStore";

interface ModelSelectorProps {
  onClose: () => void;
}

const MODELS = [
  {
    id: "qwen3.5-35b-a3b-q4km",
    name: "Qwen 3.5 35B-A3B",
    params: "35B (3B active MoE)",
    size: "19 GB",
    family: "Qwen",
    tq: true,
    minRam: 16,
    tags: ["Fast", "Coding", "MoE"],
  },
  {
    id: "llama3.3-70b-q4km",
    name: "Llama 3.3 70B",
    params: "70B",
    size: "40 GB",
    family: "Llama",
    tq: true,
    minRam: 48,
    tags: ["Powerful", "General"],
  },
  {
    id: "gemma3-27b-q4km",
    name: "Gemma 3 27B",
    params: "27B",
    size: "15 GB",
    family: "Gemma",
    tq: true,
    minRam: 16,
    tags: ["Google", "Multimodal"],
  },
  {
    id: "phi4-14b-q4km",
    name: "Phi-4 14B",
    params: "14B",
    size: "8 GB",
    family: "Phi",
    tq: true,
    minRam: 8,
    tags: ["Efficient", "Reasoning"],
  },
  {
    id: "qwen3-8b-q4km",
    name: "Qwen 3 8B",
    params: "8B",
    size: "4.5 GB",
    family: "Qwen",
    tq: true,
    minRam: 8,
    tags: ["Lightweight", "Fast"],
  },
];

export function ModelSelector({ onClose }: ModelSelectorProps) {
  const activeModelId = useAppStore((s) => s.activeModelId);
  const setActiveModel = useAppStore((s) => s.setActiveModel);

  const handleSelect = (id: string) => {
    setActiveModel(id);
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-nexus-surface border border-nexus-border rounded-xl w-full max-w-2xl max-h-[80vh] overflow-hidden">
        <div className="flex items-center justify-between px-6 py-4 border-b border-nexus-border">
          <div>
            <h2 className="text-lg font-bold">Select Model</h2>
            <p className="text-xs text-nexus-dim mt-0.5">
              All models run locally with TurboQuant KV compression
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-nexus-dim hover:text-nexus-text transition-colors text-sm"
          >
            Close
          </button>
        </div>

        <div className="overflow-y-auto p-4 space-y-2">
          {MODELS.map((model) => (
            <button
              key={model.id}
              onClick={() => handleSelect(model.id)}
              className={`w-full text-left px-4 py-3 rounded-lg border transition-all ${
                activeModelId === model.id
                  ? "bg-nexus-accent/10 border-nexus-accent/30 ring-1 ring-nexus-accent/20"
                  : "bg-nexus-surface2 border-nexus-border hover:border-nexus-accent/20 hover:bg-nexus-surface3"
              }`}
            >
              <div className="flex items-center justify-between mb-1">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-semibold text-nexus-text">
                    {model.name}
                  </span>
                  {activeModelId === model.id && (
                    <span className="text-[9px] font-mono px-1.5 py-0.5 rounded-full bg-nexus-accent/20 text-nexus-accent">
                      Active
                    </span>
                  )}
                </div>
                <span className="text-xs font-mono text-nexus-dim">{model.size}</span>
              </div>

              <div className="flex items-center gap-3 text-[10px] font-mono text-nexus-dim">
                <span>{model.params}</span>
                <span>Q4_K_M</span>
                <span>Min {model.minRam}GB RAM</span>
                {model.tq && (
                  <span className="text-nexus-accent">TQ Compatible</span>
                )}
              </div>

              <div className="flex gap-1 mt-2">
                {model.tags.map((tag) => (
                  <span
                    key={tag}
                    className="text-[9px] font-mono px-1.5 py-0.5 rounded-full
                               bg-nexus-surface border border-nexus-border text-nexus-dim"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </button>
          ))}
        </div>

        <div className="px-6 py-3 border-t border-nexus-border bg-nexus-surface2/50">
          <p className="text-[10px] font-mono text-nexus-dim text-center">
            Place .gguf files in the models directory or download from the catalog above
          </p>
        </div>
      </div>
    </div>
  );
}
