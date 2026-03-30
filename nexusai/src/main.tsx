import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { useAppStore } from "./stores/appStore";
import "./index.css";

// Initialize backend connection (loads conversations + models from SQLite).
// Retry briefly in case Tauri globals aren't injected yet at module load time.
function initBackend(retries = 10) {
  if ("__TAURI_INTERNALS__" in window) {
    useAppStore.getState().init();
  } else if (retries > 0) {
    setTimeout(() => initBackend(retries - 1), 100);
  } else {
    console.warn("[NexusAI] Tauri not detected after retries — running in stub mode");
    useAppStore.getState().init();
  }
}
initBackend();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
