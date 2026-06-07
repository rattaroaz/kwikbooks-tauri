import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import * as api from "../api/tauri";
import { formatMoneyMinor } from "../lib/money";
import { useToast } from "../context/ToastContext";
import { errorMessage } from "../types/errors";

type Row = {
  id: number;
  number: string;
  status: string;
  vendorName?: string | null;
  payeeName?: string | null;
  issueDate: string;
  totalMinor: number;
};

export function BillsPage() {
  const { push } = useToast();
  const [rows, setRows] = useState<Row[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = (await api.listBills()) as Row[];
      setRows(data);
    } catch (e) {
      push("error", errorMessage(e));
    } finally {
      setLoading(false);
    }
  }, [push]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className="kb-page">
      <div className="kb-page-head">
        <h1>Bills</h1>
        <Link className="kb-button" to="/bills/new">
          New bill
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
          <p>No bills yet.</p>
          <p className="kb-muted">
            Bills can tie to vendors or standalone payees. Posting requires
            status <strong>open</strong>.
          </p>
          <Link className="kb-button" to="/bills/new">
            Create bill
          </Link>
        </div>
      ) : (
        <table className="kb-table">
          <thead>
            <tr>
              <th>#</th>
              <th>Vendor / payee</th>
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
                <td>{r.vendorName ?? r.payeeName ?? "—"}</td>
                <td>{r.issueDate}</td>
                <td>{r.status}</td>
                <td>{formatMoneyMinor(r.totalMinor)}</td>
                <td>
                  <Link to={`/bills/${r.id}`}>Open</Link>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
