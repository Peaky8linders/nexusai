import { useEffect } from "react";
import { create } from "zustand";

interface ToastMessage {
  id: string;
  type: "info" | "success" | "error" | "warning";
  message: string;
  duration?: number;
}

interface ToastStore {
  toasts: ToastMessage[];
  addToast: (type: ToastMessage["type"], message: string, duration?: number) => void;
  removeToast: (id: string) => void;
}

export const useToastStore = create<ToastStore>((set) => ({
  toasts: [],
  addToast: (type, message, duration = 4000) => {
    const id = crypto.randomUUID();
    set((s) => ({ toasts: [...s.toasts, { id, type, message, duration }] }));
    if (duration > 0) {
      setTimeout(() => {
        set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
      }, duration);
    }
  },
  removeToast: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 max-w-sm">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onDismiss={() => removeToast(toast.id)} />
      ))}
    </div>
  );
}

function ToastItem({ toast, onDismiss }: { toast: ToastMessage; onDismiss: () => void }) {
  const colors: Record<string, string> = {
    info: "border-nexus-accent2/30 bg-nexus-accent2/10 text-nexus-accent2",
    success: "border-nexus-accent/30 bg-nexus-accent/10 text-nexus-accent",
    error: "border-red-400/30 bg-red-400/10 text-red-400",
    warning: "border-nexus-warn/30 bg-nexus-warn/10 text-nexus-warn",
  };

  const icons: Record<string, string> = {
    info: "i",
    success: "✓",
    error: "!",
    warning: "⚠",
  };

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onDismiss();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onDismiss]);

  return (
    <div
      role="alert"
      aria-live="polite"
      className={`flex items-start gap-3 px-4 py-3 rounded-lg border backdrop-blur-sm
                  shadow-lg animate-slide-in ${colors[toast.type] ?? colors.info}`}
    >
      <span className="text-xs font-mono font-bold mt-0.5">{icons[toast.type]}</span>
      <p className="text-xs flex-1">{toast.message}</p>
      <button
        onClick={onDismiss}
        className="text-xs opacity-50 hover:opacity-100 transition-opacity"
        aria-label="Dismiss notification"
      >
        x
      </button>
    </div>
  );
}
