import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

/** 捕获渲染期错误，避免黑屏/白屏，显示可读错误信息。 */
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { error: Error | null }
> {
  state = { error: null as Error | null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <main className="flex min-h-screen items-center justify-center bg-background p-6 text-foreground">
          <div className="max-w-xl rounded border border-red-300 bg-red-50 p-4 text-sm text-red-800">
            <p className="mb-2 font-semibold">界面发生错误（请把以下信息发给开发者）</p>
            <pre className="whitespace-pre-wrap break-all font-mono text-xs">
              {String(this.state.error?.message ?? this.state.error)}
            </pre>
          </div>
        </main>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
