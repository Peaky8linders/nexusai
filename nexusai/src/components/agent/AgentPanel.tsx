import { useState } from "react";

interface ToolCall {
  name: string;
  arguments: Record<string, unknown>;
  requiresApproval: boolean;
}

interface ToolResult {
  name: string;
  success: boolean;
  output: string;
  error?: string;
}

interface AgentStep {
  step: number;
  toolCall: ToolCall;
  result?: ToolResult;
  status: "pending" | "approved" | "rejected" | "executing" | "complete";
}

interface AgentPanelProps {
  steps: AgentStep[];
  onApprove: (step: number) => void;
  onReject: (step: number) => void;
  onApproveAll: () => void;
}

export function AgentPanel({ steps, onApprove, onReject, onApproveAll }: AgentPanelProps) {
  const pendingApprovals = steps.filter((s) => s.status === "pending");

  if (steps.length === 0) return null;

  return (
    <div className="border-t border-nexus-border bg-nexus-surface/50 px-6 py-3">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-xs font-mono text-nexus-accent uppercase tracking-widest">
          Agent Activity
        </h3>
        {pendingApprovals.length > 0 && (
          <button
            onClick={onApproveAll}
            className="text-[10px] font-mono px-2 py-1 rounded bg-nexus-accent/10
                       text-nexus-accent border border-nexus-accent/20
                       hover:bg-nexus-accent/20 transition-colors"
          >
            Approve All ({pendingApprovals.length})
          </button>
        )}
      </div>

      <div className="space-y-2 max-h-48 overflow-y-auto">
        {steps.map((step) => (
          <AgentStepRow
            key={step.step}
            step={step}
            onApprove={() => onApprove(step.step)}
            onReject={() => onReject(step.step)}
          />
        ))}
      </div>
    </div>
  );
}

function AgentStepRow({
  step,
  onApprove,
  onReject,
}: {
  step: AgentStep;
  onApprove: () => void;
  onReject: () => void;
}) {
  const [expanded, setExpanded] = useState(false);

  const statusColors: Record<string, string> = {
    pending: "text-nexus-warn border-nexus-warn/20 bg-nexus-warn/5",
    approved: "text-nexus-accent border-nexus-accent/20 bg-nexus-accent/5",
    rejected: "text-red-400 border-red-400/20 bg-red-400/5",
    executing: "text-nexus-accent2 border-nexus-accent2/20 bg-nexus-accent2/5",
    complete: "text-nexus-dim border-nexus-border bg-nexus-surface2",
  };

  return (
    <div
      className={`rounded-lg border px-3 py-2 ${statusColors[step.status] ?? ""}`}
    >
      <div className="flex items-center justify-between">
        <div
          className="flex items-center gap-2 cursor-pointer"
          onClick={() => setExpanded(!expanded)}
        >
          <span className="text-[10px] font-mono opacity-50">#{step.step}</span>
          <span className="text-xs font-mono font-bold">{step.toolCall.name}</span>
          <span className="text-[10px] opacity-50">
            {expanded ? "collapse" : "expand"}
          </span>
        </div>

        {step.status === "pending" && (
          <div className="flex gap-1">
            <button
              onClick={onApprove}
              className="text-[10px] font-mono px-2 py-0.5 rounded
                         bg-nexus-accent/20 text-nexus-accent hover:bg-nexus-accent/30
                         transition-colors"
            >
              Approve
            </button>
            <button
              onClick={onReject}
              className="text-[10px] font-mono px-2 py-0.5 rounded
                         bg-red-500/20 text-red-400 hover:bg-red-500/30
                         transition-colors"
            >
              Reject
            </button>
          </div>
        )}

        {step.status === "executing" && (
          <span className="text-[10px] font-mono animate-pulse">Running...</span>
        )}

        {step.status === "complete" && step.result && (
          <span
            className={`text-[10px] font-mono ${
              step.result.success ? "text-nexus-accent" : "text-red-400"
            }`}
          >
            {step.result.success ? "Success" : "Failed"}
          </span>
        )}
      </div>

      {expanded && (
        <div className="mt-2 text-[11px] font-mono">
          <div className="bg-nexus-surface2 rounded p-2 overflow-x-auto">
            <pre className="whitespace-pre-wrap text-nexus-dim">
              {JSON.stringify(step.toolCall.arguments, null, 2)}
            </pre>
          </div>
          {step.result && (
            <div className="mt-1 bg-nexus-surface2 rounded p-2 overflow-x-auto">
              <pre className="whitespace-pre-wrap text-nexus-dim">
                {step.result.success
                  ? step.result.output.slice(0, 500)
                  : step.result.error}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
