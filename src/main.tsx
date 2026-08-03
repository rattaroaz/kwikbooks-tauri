import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import { ToastProvider } from "./context/ToastContext";
import { captureException } from "./config/telemetry";
import { env } from "./config/env";
import { installGlobalErrorHandlers } from "./lib/diagnostics";
import { initLoggingPipeline } from "./lib/logger";
import { ErrorBoundary } from "./components/ErrorBoundary";

void initLoggingPipeline().then(() => {
  installGlobalErrorHandlers(captureException);
});

if (env.isDev) {
  console.debug(`[kwikbooks] mode=${env.mode} diagnostics=${env.diagnostics}`);
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <BrowserRouter>
        <ToastProvider>
          <App />
        </ToastProvider>
      </BrowserRouter>
    </ErrorBoundary>
  </React.StrictMode>,
);
