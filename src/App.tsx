import { useEffect, useState, type ReactNode } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { Navigate, Route, Routes } from "react-router-dom";
import { dbInit, healthPing } from "./api/db";
import { captureException } from "./config/telemetry";
import { createScopedLogger } from "./lib/logger";
import { AppLayout } from "./layout/AppLayout";
import { BillsPage } from "./pages/BillsPage";
import { BillDetailPage } from "./pages/BillDetailPage";
import { BillNewPage } from "./pages/BillNewPage";
import { AccountsPage } from "./pages/AccountsPage";
import { CustomersPage } from "./pages/CustomersPage";
import { Dashboard } from "./pages/Dashboard";
import { InvoiceDetailPage } from "./pages/InvoiceDetailPage";
import { InvoiceNewPage } from "./pages/InvoiceNewPage";
import { InvoicesPage } from "./pages/InvoicesPage";
import { RegisterPage } from "./pages/RegisterPage";
import { ReportsPage } from "./pages/ReportsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { VendorsPage } from "./pages/VendorsPage";
import { WelcomePage } from "./pages/WelcomePage";
import { ReceivePaymentPage } from "./pages/ReceivePaymentPage";
import { PayBillPage } from "./pages/PayBillPage";
import { UpdateDialog } from "./components/UpdateDialog";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { LogViewerProvider } from "./context/LogViewerContext";
import { errorMessage } from "./types/errors";
import "./App.css";

const logBoot = createScopedLogger("DbGate");

function TauriGate({ children }: { children: ReactNode }) {
  const e2eBypass = import.meta.env.VITE_E2E === "true";
  if (!isTauri() && !e2eBypass) {
    return (
      <div className="kb-page kb-error-screen">
        <h1>Open the desktop app</h1>
        <p>
          Kwikbooks talks to SQLite through Tauri. A browser tab only running
          Vite cannot load the backend.
        </p>
        <p>
          From the project root, run <code>npm start</code> or{" "}
          <code>npm run tauri:dev</code> (not <code>npm run dev</code> alone).
        </p>
      </div>
    );
  }
  return <>{children}</>;
}

function DbGate({ children }: { children: ReactNode }) {
  const e2eBypass = import.meta.env.VITE_E2E === "true";
  const [err, setErr] = useState<string | null>(null);
  const [ready, setReady] = useState(e2eBypass);

  useEffect(() => {
    if (e2eBypass) {
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        await dbInit();
        await healthPing();
        if (!cancelled) {
          setReady(true);
          setErr(null);
          void logBoot.info("database connection ready");
        }
      } catch (e) {
        if (!cancelled) {
          captureException(e, "DbGate.init");
          setErr(errorMessage(e));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [e2eBypass]);

  if (e2eBypass) {
    return <>{children}</>;
  }

  if (err !== null) {
    return (
      <div className="kb-page kb-error-screen">
        <h1>Database</h1>
        <p>{err}</p>
      </div>
    );
  }
  if (!ready) {
    return (
      <div className="kb-page kb-loading-root" aria-busy="true">
        <p className="kb-muted">Connecting to database…</p>
        <div className="kb-skeleton-stack" aria-hidden="true">
          <div className="kb-skeleton-line kb-skeleton-line-lg" />
          <div className="kb-skeleton-line" />
          <div className="kb-skeleton-line kb-skeleton-line-sm" />
        </div>
      </div>
    );
  }
  return <>{children}</>;
}

export default function App() {
  return (
    <TauriGate>
      <UpdateDialog />
      <DbGate>
        <ErrorBoundary>
          <LogViewerProvider>
            <Routes>
              <Route element={<AppLayout />}>
                <Route path="/" element={<Dashboard />} />
                <Route path="/accounts" element={<AccountsPage />} />
                <Route path="/customers" element={<CustomersPage />} />
                <Route path="/vendors" element={<VendorsPage />} />
                <Route path="/invoices" element={<InvoicesPage />} />
                <Route path="/invoices/new" element={<InvoiceNewPage />} />
                <Route path="/invoices/:id" element={<InvoiceDetailPage />} />
                <Route path="/bills" element={<BillsPage />} />
                <Route path="/bills/new" element={<BillNewPage />} />
                <Route path="/bills/:id" element={<BillDetailPage />} />
                <Route
                  path="/payments/receive"
                  element={<ReceivePaymentPage />}
                />
                <Route path="/payments/pay" element={<PayBillPage />} />
                <Route path="/register" element={<RegisterPage />} />
                <Route path="/reports" element={<ReportsPage />} />
                <Route path="/settings" element={<SettingsPage />} />
                <Route path="/welcome" element={<WelcomePage />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Route>
            </Routes>
          </LogViewerProvider>
        </ErrorBoundary>
      </DbGate>
    </TauriGate>
  );
}
