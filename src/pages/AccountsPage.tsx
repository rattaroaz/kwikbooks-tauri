import { useCallback, useEffect, useState } from "react";
import * as api from "../api/tauri";
import { formatMoneyMinor } from "../lib/money";
import { useToast } from "../context/ToastContext";
import { errorMessage } from "../types/errors";

type Row = {
  id: number;
  code: string;
  name: string;
  accountType: string;
  isBankCash: boolean;
  isActive: boolean;
};

export function AccountsPage() {
  const { push } = useToast();
  const [rows, setRows] = useState<Row[]>([]);
  const [filter, setFilter] = useState({ activeOnly: true });

  const load = useCallback(async () => {
    try {
      const data = (await api.accountList(filter)) as Row[];
      setRows(data);
    } catch (e) {
      push("error", errorMessage(e));
    }
  }, [filter, push]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className="kb-page">
      <h1>Chart of accounts</h1>
      <label className="kb-inline">
        <input
          type="checkbox"
          checked={filter.activeOnly}
          onChange={(e) =>
            setFilter({ activeOnly: e.currentTarget.checked })
          }
        />{" "}
        Active only
      </label>
      <table className="kb-table">
        <thead>
          <tr>
            <th>Code</th>
            <th>Name</th>
            <th>Type</th>
            <th>Bank</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.id}>
              <td>{r.code}</td>
              <td>{r.name}</td>
              <td>{r.accountType}</td>
              <td>{r.isBankCash ? "Yes" : ""}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="kb-muted">
        Display currency example: {formatMoneyMinor(123456)} (informational)
      </p>
    </div>
  );
}
