import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("[NexusAI] Render error:", error, info.componentStack);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div className="flex items-center justify-center h-full p-8">
          <div className="max-w-md text-center">
            <div className="text-2xl font-extrabold mb-2">
              Nexus<span className="gradient-text">AI</span>
            </div>
            <p className="text-nexus-dim text-sm mb-4">Something went wrong.</p>
            <pre className="bg-nexus-surface2 border border-nexus-border rounded-lg p-4
                            text-xs font-mono text-red-400 text-left overflow-auto max-h-32 mb-4">
              {this.state.error?.message}
            </pre>
            <button
              onClick={() => this.setState({ hasError: false, error: null })}
              className="px-4 py-2 bg-nexus-accent/10 text-nexus-accent border border-nexus-accent/20
                         rounded-lg hover:bg-nexus-accent/20 transition-colors text-sm font-mono"
            >
              Try Again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
