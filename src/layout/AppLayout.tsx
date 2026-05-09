import { useEffect } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { useToast } from "../context/ToastContext";

const links = [
  { to: "/", label: "Dashboard" },
  { to: "/welcome", label: "Getting started" },
  { to: "/accounts", label: "Chart of accounts" },
  { to: "/customers", label: "Customers" },
  { to: "/vendors", label: "Vendors" },
  { to: "/invoices", label: "Invoices" },
  { to: "/bills", label: "Bills" },
  { to: "/register", label: "Journal register" },
  { to: "/reports", label: "Reports" },
  { to: "/settings", label: "Settings" },
];

export function AppLayout() {
  const navigate = useNavigate();
  const { toasts, dismiss } = useToast();

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
        <Outlet />
      </div>
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
