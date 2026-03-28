import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { useAppStore } from "./stores/appStore";
import "./index.css";

// Initialize backend connection (loads conversations + models from SQLite)
useAppStore.getState().init();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
