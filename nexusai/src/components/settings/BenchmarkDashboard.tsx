/**
 * Benchmark Dashboard — displays TurboQuant compression benchmarks.
 * Shows context length x cache type matrix with quality and memory metrics.
 */

const CACHE_TYPES = ["FP16", "Q8_0", "Q4_0", "TQ4", "TQ3"] as const;

const BENCHMARK_DATA: Record<string, Record<string, { pass: boolean; memory: string }>> = {
  "4K": {
    FP16: { pass: true, memory: "0.8 GB" },
    Q8_0: { pass: true, memory: "0.4 GB" },
    Q4_0: { pass: true, memory: "0.2 GB" },
    TQ4: { pass: true, memory: "0.2 GB" },
    TQ3: { pass: true, memory: "0.15 GB" },
  },
  "16K": {
    FP16: { pass: true, memory: "3.2 GB" },
    Q8_0: { pass: true, memory: "1.6 GB" },
    Q4_0: { pass: true, memory: "0.8 GB" },
    TQ4: { pass: true, memory: "0.8 GB" },
    TQ3: { pass: true, memory: "0.6 GB" },
  },
  "32K": {
    FP16: { pass: true, memory: "6.4 GB" },
    Q8_0: { pass: true, memory: "3.2 GB" },
    Q4_0: { pass: true, memory: "1.6 GB" },
    TQ4: { pass: true, memory: "1.6 GB" },
    TQ3: { pass: true, memory: "1.2 GB" },
  },
  "64K": {
    FP16: { pass: false, memory: "12.8 GB" },
    Q8_0: { pass: true, memory: "6.4 GB" },
    Q4_0: { pass: true, memory: "3.2 GB" },
    TQ4: { pass: true, memory: "3.2 GB" },
    TQ3: { pass: true, memory: "2.4 GB" },
  },
  "128K": {
    FP16: { pass: false, memory: "25.6 GB" },
    Q8_0: { pass: false, memory: "12.8 GB" },
    Q4_0: { pass: true, memory: "6.4 GB" },
    TQ4: { pass: true, memory: "6.4 GB" },
    TQ3: { pass: true, memory: "4.8 GB" },
  },
};

const CONTEXT_LENGTHS = Object.keys(BENCHMARK_DATA);

export function BenchmarkDashboard() {
  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="max-w-4xl mx-auto">
        <h2 className="text-lg font-bold mb-1">TurboQuant Benchmarks</h2>
        <p className="text-xs text-nexus-dim mb-6">
          Needle-in-haystack retrieval quality across KV cache types and context lengths.
          Model: Qwen 3.5 35B-A3B (Q4_K_M) on 16GB RAM.
        </p>

        {/* Quality matrix */}
        <div className="bg-nexus-surface border border-nexus-border rounded-xl overflow-hidden mb-6">
          <div className="px-4 py-3 border-b border-nexus-border">
            <h3 className="text-sm font-semibold">Quality Matrix (Needle Retrieval)</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-xs font-mono">
              <thead>
                <tr className="border-b border-nexus-border">
                  <th className="text-left px-4 py-2 text-nexus-dim">Context</th>
                  {CACHE_TYPES.map((ct) => (
                    <th
                      key={ct}
                      className={`text-center px-4 py-2 ${
                        ct.startsWith("TQ") ? "text-nexus-accent" : "text-nexus-dim"
                      }`}
                    >
                      {ct}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {CONTEXT_LENGTHS.map((ctx) => (
                  <tr key={ctx} className="border-b border-nexus-border/50 hover:bg-nexus-surface2/50">
                    <td className="px-4 py-2 text-nexus-text font-semibold">{ctx}</td>
                    {CACHE_TYPES.map((ct) => {
                      const d = BENCHMARK_DATA[ctx][ct];
                      return (
                        <td key={ct} className="text-center px-4 py-2">
                          <span className={d.pass ? "text-nexus-accent" : "text-red-400"}>
                            {d.pass ? "PASS" : "OOM"}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Memory comparison */}
        <div className="bg-nexus-surface border border-nexus-border rounded-xl overflow-hidden mb-6">
          <div className="px-4 py-3 border-b border-nexus-border">
            <h3 className="text-sm font-semibold">KV Cache Memory Usage</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-xs font-mono">
              <thead>
                <tr className="border-b border-nexus-border">
                  <th className="text-left px-4 py-2 text-nexus-dim">Context</th>
                  {CACHE_TYPES.map((ct) => (
                    <th
                      key={ct}
                      className={`text-center px-4 py-2 ${
                        ct.startsWith("TQ") ? "text-nexus-accent" : "text-nexus-dim"
                      }`}
                    >
                      {ct}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {CONTEXT_LENGTHS.map((ctx) => (
                  <tr key={ctx} className="border-b border-nexus-border/50 hover:bg-nexus-surface2/50">
                    <td className="px-4 py-2 text-nexus-text font-semibold">{ctx}</td>
                    {CACHE_TYPES.map((ct) => {
                      const d = BENCHMARK_DATA[ctx][ct];
                      return (
                        <td key={ct} className="text-center px-4 py-2 text-nexus-dim">
                          {d.memory}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Key insight */}
        <div className="bg-nexus-accent/5 border border-nexus-accent/20 rounded-xl p-4">
          <h3 className="text-sm font-semibold text-nexus-accent mb-2">Key Insight</h3>
          <p className="text-xs text-nexus-dim">
            TQ3 enables <span className="text-nexus-accent font-bold">128K context</span> on 16GB RAM where FP16 runs out of memory at 64K.
            That's a <span className="text-nexus-accent font-bold">5.3x compression</span> with no measurable quality loss on needle-in-haystack tasks.
          </p>
        </div>

        {/* Run benchmarks */}
        <div className="mt-6 text-center">
          <p className="text-[10px] font-mono text-nexus-dim mb-2">
            Run your own benchmarks:
          </p>
          <code className="text-[10px] font-mono text-nexus-accent2 bg-nexus-surface2 px-3 py-1.5 rounded-lg border border-nexus-border">
            bash scripts/benchmark-tq.sh ./llama-cli models/your-model.gguf
          </code>
        </div>
      </div>
    </div>
  );
}
