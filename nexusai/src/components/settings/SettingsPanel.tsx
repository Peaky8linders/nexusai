import { useState } from "react";

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const [tqBits, setTqBits] = useState<3 | 4>(3);
  const [contextLength, setContextLength] = useState(65536);
  const [temperature, setTemperature] = useState(0.7);
  const [gpuLayers, setGpuLayers] = useState(-1);

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-nexus-surface border border-nexus-border rounded-xl w-full max-w-lg max-h-[80vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-nexus-border">
          <h2 className="text-lg font-bold">Settings</h2>
          <button
            onClick={onClose}
            className="text-nexus-dim hover:text-nexus-text transition-colors text-sm"
          >
            Close
          </button>
        </div>

        <div className="px-6 py-5 space-y-6">
          {/* TurboQuant Settings */}
          <section>
            <h3 className="text-xs font-mono text-nexus-accent uppercase tracking-widest mb-3">
              TurboQuant KV Cache
            </h3>
            <div className="space-y-4">
              <div>
                <label className="text-sm text-nexus-dim block mb-1">
                  Quantization Bits
                </label>
                <div className="flex gap-2">
                  {([3, 4] as const).map((bits) => (
                    <button
                      key={bits}
                      onClick={() => setTqBits(bits)}
                      className={`px-4 py-2 rounded-lg text-sm font-mono border transition-colors ${
                        tqBits === bits
                          ? "bg-nexus-accent/10 border-nexus-accent/30 text-nexus-accent"
                          : "bg-nexus-surface2 border-nexus-border text-nexus-dim hover:text-nexus-text"
                      }`}
                    >
                      TQ{bits}
                      <span className="block text-[10px] mt-0.5">
                        {bits === 3 ? "5.3x compress" : "4x compress"}
                      </span>
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <label className="text-sm text-nexus-dim block mb-1">
                  Context Length: {contextLength.toLocaleString()} tokens
                </label>
                <input
                  type="range"
                  min={4096}
                  max={131072}
                  step={4096}
                  value={contextLength}
                  onChange={(e) => setContextLength(Number(e.target.value))}
                  className="w-full accent-nexus-accent"
                />
                <div className="flex justify-between text-[10px] font-mono text-nexus-dim">
                  <span>4K</span>
                  <span>32K</span>
                  <span>64K</span>
                  <span>128K</span>
                </div>
              </div>
            </div>
          </section>

          {/* Inference Settings */}
          <section>
            <h3 className="text-xs font-mono text-nexus-accent2 uppercase tracking-widest mb-3">
              Inference
            </h3>
            <div className="space-y-4">
              <div>
                <label className="text-sm text-nexus-dim block mb-1">
                  Temperature: {temperature.toFixed(2)}
                </label>
                <input
                  type="range"
                  min={0}
                  max={2}
                  step={0.05}
                  value={temperature}
                  onChange={(e) => setTemperature(Number(e.target.value))}
                  className="w-full accent-nexus-accent2"
                />
              </div>

              <div>
                <label className="text-sm text-nexus-dim block mb-1">
                  GPU Layers: {gpuLayers === -1 ? "All" : gpuLayers}
                </label>
                <input
                  type="range"
                  min={-1}
                  max={100}
                  step={1}
                  value={gpuLayers}
                  onChange={(e) => setGpuLayers(Number(e.target.value))}
                  className="w-full accent-nexus-accent2"
                />
                <div className="flex justify-between text-[10px] font-mono text-nexus-dim">
                  <span>All GPU</span>
                  <span>CPU only (0)</span>
                </div>
              </div>
            </div>
          </section>

          {/* Memory Estimation */}
          <section>
            <h3 className="text-xs font-mono text-nexus-accent3 uppercase tracking-widest mb-3">
              Memory Estimate
            </h3>
            <div className="bg-nexus-surface2 rounded-lg p-4 text-sm space-y-2">
              <div className="flex justify-between">
                <span className="text-nexus-dim">KV Cache (TQ{tqBits})</span>
                <span className="font-mono text-nexus-accent">
                  {estimateKvMemory(contextLength, tqBits).toFixed(1)} GB
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-nexus-dim">KV Cache (FP16)</span>
                <span className="font-mono text-nexus-dim line-through">
                  {estimateKvMemory(contextLength, 16).toFixed(1)} GB
                </span>
              </div>
              <div className="flex justify-between border-t border-nexus-border pt-2">
                <span className="text-nexus-dim">Savings</span>
                <span className="font-mono text-nexus-accent font-bold">
                  {(
                    ((estimateKvMemory(contextLength, 16) -
                      estimateKvMemory(contextLength, tqBits)) /
                      estimateKvMemory(contextLength, 16)) *
                    100
                  ).toFixed(0)}
                  % smaller
                </span>
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

function estimateKvMemory(contextLength: number, bits: number): number {
  // Estimate for a ~35B model: 64 layers, 8 KV heads, head_dim 128
  const layers = 64;
  const kvHeads = 8;
  const headDim = 128;
  const elementsPerToken = 2 * layers * kvHeads * headDim;
  const totalBits = contextLength * elementsPerToken * bits;
  return totalBits / 8 / 1024 / 1024 / 1024; // bytes to GB
}
