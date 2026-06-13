import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import * as api from "../api/tauri";
import { formatMoneyMinor } from "../lib/money";
import { logContext } from "../lib/logContext";
import { useToast } from "../context/ToastContext";

type InvRow = {
  id: number;
  number: string;
  status: string;
  customerName: string;
  issueDate: string;
  totalMinor: number;
  journalId?: number | null;
};

export function InvoicesPage() {
  const { pushApiError } = useToast();
  const [rows, setRows] = useState<InvRow[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = (await api.listInvoices()) as InvRow[];
      setRows(data);
    } catch (e) {
      pushApiError(e, logContext("InvoicesPage", "load"));
    } finally {
      setLoading(false);
    }
  }, [pushApiError]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className="kb-page">
      <div className="kb-page-head">
        <h1>Invoices</h1>
        <Link className="kb-button" to="/invoices/new">
          New invoice
        </Link>
      </div>
      {loading ? (
        <div className="kb-skeleton-table" aria-busy="true">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="kb-skeleton-line kb-skeleton-line-table" />
          ))}
        </div>
      ) : rows.length === 0 ? (
        <div className="kb-empty">
          <p>No invoices yet.</p>
          <p className="kb-muted">
            Add a customer first, then create a draft and mark it sent before
            posting to the ledger.
          </p>
          <Link className="kb-button" to="/invoices/new">
            Create invoice
          </Link>
        </div>
      ) : (
        <table className="kb-table">
          <thead>
            <tr>
              <th>#</th>
              <th>Customer</th>
              <th>Date</th>
              <th>Status</th>
              <th>Total</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id}>
                <td>{r.number}</td>
                <td>{r.customerName}</td>
                <td>{r.issueDate}</td>
                <td>{r.status}</td>
                <td>{formatMoneyMinor(r.totalMinor)}</td>
                <td>
                  <Link to={`/invoices/${r.id}`}>Open</Link>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
