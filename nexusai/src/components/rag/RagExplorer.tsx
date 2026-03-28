import { useState } from "react";

interface IndexedFile {
  path: string;
  name: string;
  chunks: number;
  size: string;
  lastIndexed: string;
}

interface RagExplorerProps {
  files: IndexedFile[];
  totalChunks: number;
  indexSizeMb: number;
  tqSavingsPercent: number;
  onDropFiles: (paths: string[]) => void;
  onRemoveFile: (path: string) => void;
  onReindex: () => void;
}

export function RagExplorer({
  files,
  totalChunks,
  indexSizeMb,
  tqSavingsPercent,
  onDropFiles,
  onRemoveFile,
  onReindex,
}: RagExplorerProps) {
  const [isDragOver, setDragOver] = useState(false);

  return (
    <div className="h-full flex flex-col">
      {/* Stats bar */}
      <div className="flex items-center gap-4 px-4 py-3 border-b border-nexus-border bg-nexus-surface/50">
        <div className="text-xs font-mono">
          <span className="text-nexus-dim">Files:</span>{" "}
          <span className="text-nexus-text">{files.length}</span>
        </div>
        <div className="text-xs font-mono">
          <span className="text-nexus-dim">Chunks:</span>{" "}
          <span className="text-nexus-text">{totalChunks.toLocaleString()}</span>
        </div>
        <div className="text-xs font-mono">
          <span className="text-nexus-dim">Index:</span>{" "}
          <span className="text-nexus-accent">{indexSizeMb.toFixed(1)} MB</span>
        </div>
        {tqSavingsPercent > 0 && (
          <div className="text-xs font-mono">
            <span className="text-nexus-dim">TQ savings:</span>{" "}
            <span className="text-nexus-accent">{tqSavingsPercent.toFixed(0)}%</span>
          </div>
        )}
        <div className="flex-1" />
        <button
          onClick={onReindex}
          className="text-[10px] font-mono px-2 py-1 rounded
                     text-nexus-dim hover:text-nexus-text
                     border border-nexus-border hover:border-nexus-accent/30
                     transition-colors"
        >
          Re-index All
        </button>
      </div>

      {/* Drop zone */}
      <div
        className={`flex-1 overflow-y-auto p-4 transition-colors ${
          isDragOver ? "bg-nexus-accent/5" : ""
        }`}
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDragOver(false);
          const items = Array.from(e.dataTransfer.files).map((f) => f.name);
          onDropFiles(items);
        }}
      >
        {files.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="text-4xl mb-4 opacity-20">+</div>
              <p className="text-nexus-dim text-sm mb-1">
                Drop files or folders here to index
              </p>
              <p className="text-nexus-dim/50 text-xs font-mono">
                Supports: code, markdown, text, PDF, DOCX
              </p>
              <p className="text-nexus-dim/50 text-xs font-mono mt-1">
                Embeddings compressed with TurboQuant (6x smaller)
              </p>
            </div>
          </div>
        ) : (
          <div className="space-y-1">
            {files.map((file) => (
              <div
                key={file.path}
                className="group flex items-center justify-between px-3 py-2
                           rounded-lg hover:bg-nexus-surface2 transition-colors"
              >
                <div className="flex items-center gap-3 min-w-0">
                  <FileIcon name={file.name} />
                  <div className="min-w-0">
                    <div className="text-xs text-nexus-text truncate">{file.name}</div>
                    <div className="text-[10px] font-mono text-nexus-dim">
                      {file.chunks} chunks | {file.size} | {file.lastIndexed}
                    </div>
                  </div>
                </div>
                <button
                  onClick={() => onRemoveFile(file.path)}
                  className="opacity-0 group-hover:opacity-100 text-nexus-dim
                             hover:text-red-400 transition-all text-xs"
                >
                  Remove
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function FileIcon({ name }: { name: string }) {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  const colors: Record<string, string> = {
    ts: "text-blue-400",
    tsx: "text-blue-400",
    js: "text-yellow-400",
    jsx: "text-yellow-400",
    rs: "text-orange-400",
    py: "text-green-400",
    md: "text-nexus-dim",
    pdf: "text-red-400",
    docx: "text-blue-300",
  };

  return (
    <span className={`text-[10px] font-mono uppercase ${colors[ext] ?? "text-nexus-dim"}`}>
      .{ext}
    </span>
  );
}
