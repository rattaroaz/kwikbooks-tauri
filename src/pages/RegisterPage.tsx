import { useCallback, useEffect, useState } from "react";
import * as api from "../api/tauri";
import { useToast } from "../context/ToastContext";
import { errorMessage } from "../types/errors";

type JRow = {
  id: number;
  entryDate: string;
  memo?: string | null;
  sourceKind?: string | null;
  sourceId?: number | null;
};

export function RegisterPage() {
  const { push } = useToast();
  const [rows, setRows] = useState<JRow[]>([]);

  const load = useCallback(async () => {
    try {
      const data = (await api.listJournals(300)) as JRow[];
      setRows(data);
    } catch (e) {
      push("error", errorMessage(e));
    }
  }, [push]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className="kb-page">
      <h1>Journal register</h1>
      <p className="kb-muted">
        Recent journals (newest first). Source links tie back to invoices,
        bills, or payments.
      </p>
      <table className="kb-table">
        <thead>
          <tr>
            <th>Date</th>
            <th>ID</th>
            <th>Memo</th>
            <th>Source</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.id}>
              <td>{r.entryDate}</td>
              <td>{r.id}</td>
              <td>{r.memo ?? ""}</td>
              <td>
                {r.sourceKind ?? ""}
                {r.sourceId != null ? ` #${r.sourceId}` : ""}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
