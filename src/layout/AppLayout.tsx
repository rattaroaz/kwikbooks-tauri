import { useEffect } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { GlobalSearch } from "../components/GlobalSearch";
import { LogViewerPanel } from "../components/LogViewerPanel";
import { useLogViewer } from "../context/LogViewerContext";
import { useToast } from "../context/ToastContext";

const links = [
  { to: "/", label: "Dashboard" },
  { to: "/welcome", label: "Getting started" },
  { to: "/accounts", label: "Chart of accounts" },
  { to: "/customers", label: "Customers" },
  { to: "/vendors", label: "Vendors" },
  { to: "/invoices", label: "Invoices" },
  { to: "/bills", label: "Bills" },
  { to: "/payments/receive", label: "Receive payment" },
  { to: "/payments/pay", label: "Pay vendor" },
  { to: "/register", label: "Journal register" },
  { to: "/reports", label: "Reports" },
  { to: "/settings", label: "Settings" },
];

export function AppLayout() {
  const navigate = useNavigate();
  const { toasts, dismiss } = useToast();
  const { open: logsOpen, closeLogs } = useLogViewer();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.altKey || e.ctrlKey || e.metaKey || e.repeat) {
        return;
      }
      const targets: Partial<Record<string, string>> = {
        Digit1: "/",
        Digit2: "/accounts",
        Digit3: "/customers",
        Digit4: "/vendors",
        Digit5: "/invoices",
        Digit6: "/bills",
        Digit7: "/register",
        Digit8: "/reports",
        Digit9: "/settings",
        Digit0: "/welcome",
      };
      const to = targets[e.code];
      if (!to) {
        return;
      }
      e.preventDefault();
      navigate(to);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [navigate]);

  return (
    <div className="kb-shell">
      <aside className="kb-sidebar">
        <div className="kb-brand">Kwikbooks</div>
        <GlobalSearch />
        <nav className="kb-nav">
          {links.map(({ to, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                isActive ? "kb-nav-link kb-nav-link-active" : "kb-nav-link"
              }
            >
              {label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <div className="kb-main">
        <a href="#kb-main-content" className="kb-skip-link">
          Skip to content
        </a>
        <div id="kb-main-content">
          <Outlet />
        </div>
      </div>
      {logsOpen ? <LogViewerPanel onClose={closeLogs} /> : null}
      <div className="kb-toasts" aria-live="polite">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={`kb-toast kb-toast-${t.kind}`}
            role="status"
          >
            <span>{t.message}</span>
            <button type="button" onClick={() => dismiss(t.id)}>
              ×
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
