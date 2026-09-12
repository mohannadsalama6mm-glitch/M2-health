import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource/ibm-plex-sans-arabic/400.css";
import "@fontsource/ibm-plex-sans-arabic/500.css";
import "@fontsource/ibm-plex-sans-arabic/600.css";
import { App } from "./app/App";
import "./design-system/tokens/tokens.css";
import "./design-system/components/components.css";
import "./app/app.css";
import "./app/desktop.css";
import "./design-system/patterns/workspace-patterns.css";
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
