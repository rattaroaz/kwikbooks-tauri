import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import * as api from "../api/tauri";
import { logContext } from "../lib/logContext";
import { useToast } from "../context/ToastContext";

export function Dashboard() {
  const { pushApiError } = useToast();
  const [inv, setInv] = useState<number | null>(null);
  const [bill, setBill] = useState<number | null>(null);
  const [ar, setAr] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void (async () => {
      setLoading(true);
      try {
        const [i, b, a] = await Promise.all([
          api.listInvoices(),
          api.listBills(),
          api.reportArOpen(),
        ]);
        setInv(i.length);
        setBill(b.length);
        setAr(a.length);
      } catch (e) {
        pushApiError(e, logContext("Dashboard", "load"));
      } finally {
        setLoading(false);
      }
    })();
  }, [pushApiError]);

  return (
    <div className="kb-page">
      <h1>Dashboard</h1>
      <p className="kb-muted">
        Local QuickBooks-style books — SQLite + double-entry posting.
      </p>
      <div className="kb-cards">
        {loading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="kb-card kb-card-skeleton" aria-busy="true">
              <div className="kb-skeleton-line kb-skeleton-line-sm" />
              <div className="kb-skeleton-line kb-skeleton-line-lg" />
              <div className="kb-skeleton-line kb-skeleton-line-sm" />
            </div>
          ))
        ) : (
          <>
            <div className="kb-card">
              <div className="kb-card-label">Invoices</div>
              <div className="kb-card-value">{inv ?? "—"}</div>
              <Link to="/invoices">View</Link>
            </div>
            <div className="kb-card">
              <div className="kb-card-label">Bills</div>
              <div className="kb-card-value">{bill ?? "—"}</div>
              <Link to="/bills">View</Link>
            </div>
            <div className="kb-card">
              <div className="kb-card-label">Open AR (customers)</div>
              <div className="kb-card-value">{ar ?? "—"}</div>
              <Link to="/reports">Reports</Link>
            </div>
          </>
        )}
      </div>
      {!loading && (inv === 0 || bill === 0) ? (
        <div className="kb-empty kb-empty-inline">
          <p>
            New here? See <Link to="/welcome">Getting started</Link> for a short
            checklist.
          </p>
        </div>
      ) : null}
    </div>
  );
}
