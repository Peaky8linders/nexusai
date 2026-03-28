import { useEffect, useRef, useState } from "react";
import { useAppStore } from "../../stores/appStore";
import { useFocusTrap } from "../../hooks/useKeyboard";
import { isTauriApp } from "../../lib/tauri";
import { useToastStore } from "../Toast";
import type { DownloadProgress } from "../../types";

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
  const backendConnected = useAppStore((s) => s.backendConnected);
  const models = useAppStore((s) => s.models);
  const addToast = useToastStore((s) => s.addToast);

  const [downloads, setDownloads] = useState<Record<string, DownloadProgress>>({});

  const modalRef = useRef<HTMLDivElement>(null);
  useFocusTrap(modalRef, true);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  // Listen for download progress events while this modal is open
  useEffect(() => {
    if (!isTauriApp()) return;
    let unlisten: (() => void) | null = null;
    let cancelled = false;

    import("../../lib/tauri")
      .then(({ listenToDownloadProgress }) =>
        listenToDownloadProgress((progress) => {
          setDownloads((prev) => ({ ...prev, [progress.modelId]: progress }));
          if (progress.status === "complete") {
            addToast("success", `${progress.modelId} downloaded`);
          } else if (progress.status === "error") {
            addToast("error", `Download failed: ${progress.error ?? "unknown error"}`);
          }
        })
      )
      .then((fn) => {
        if (cancelled) { fn(); return; }
        unlisten = fn;
      })
      .catch(console.error);

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [addToast]);

  const handleSelect = (id: string) => {
    setActiveModel(id);
    onClose();
  };

  const handleDownload = async (e: React.MouseEvent, modelId: string) => {
    e.stopPropagation();
    try {
      const { downloadModel } = await import("../../lib/tauri");
      await downloadModel(modelId);
      addToast("info", `Download started for ${modelId}`);
    } catch (err) {
      addToast("error", `Failed to start download: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  // Determine download/loaded status from backend model list when available
  const modelStatus = (id: string) => {
    const backendModel = models.find((m) => m.id === id);
    return {
      downloaded: backendModel?.downloaded ?? false,
      loaded: backendModel?.loaded ?? false,
    };
  };

  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
      role="dialog"
      aria-modal="true"
      aria-label="Select model"
    >
      <div ref={modalRef} className="bg-nexus-surface border border-nexus-border rounded-xl w-full max-w-2xl max-h-[80vh] overflow-hidden">
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
          {MODELS.map((model) => {
            const { downloaded, loaded } = modelStatus(model.id);
            const dl = downloads[model.id];
            const isDownloading = dl?.status === "downloading";

            return (
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
                    {loaded && (
                      <span className="text-[9px] font-mono px-1.5 py-0.5 rounded-full bg-green-500/20 text-green-400">
                        Loaded
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-mono text-nexus-dim">{model.size}</span>
                    {backendConnected && !downloaded && !isDownloading && (
                      <button
                        onClick={(e) => handleDownload(e, model.id)}
                        className="text-[10px] font-mono px-2 py-0.5 rounded
                                   bg-nexus-accent2/20 text-nexus-accent2 border border-nexus-accent2/20
                                   hover:bg-nexus-accent2/30 transition-colors"
                      >
                        Download
                      </button>
                    )}
                    {downloaded && !loaded && (
                      <span className="text-[9px] font-mono px-1.5 py-0.5 rounded-full bg-nexus-surface border border-nexus-border text-nexus-dim">
                        On disk
                      </span>
                    )}
                  </div>
                </div>

                {/* Download progress bar */}
                {isDownloading && dl && (
                  <div className="mt-2">
                    <div className="flex justify-between text-[10px] font-mono text-nexus-dim mb-1">
                      <span>{dl.percent.toFixed(1)}%</span>
                      <span>{dl.speedMbps.toFixed(1)} Mbps · {Math.ceil(dl.etaSeconds / 60)}m left</span>
                    </div>
                    <div className="h-1 bg-nexus-surface3 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-nexus-accent2 transition-all duration-300"
                        style={{ width: `${dl.percent}%` }}
                      />
                    </div>
                  </div>
                )}

                <div className="flex items-center gap-3 text-[10px] font-mono text-nexus-dim mt-1">
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
            );
          })}
        </div>

        <div className="px-6 py-3 border-t border-nexus-border bg-nexus-surface2/50">
          <p className="text-[10px] font-mono text-nexus-dim text-center">
            {backendConnected
              ? "Place .gguf files in the models directory or click Download"
              : "Running in stub mode — connect Tauri backend to download models"}
          </p>
        </div>
      </div>
    </div>
  );
}
