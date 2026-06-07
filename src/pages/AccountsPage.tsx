import { FormEvent, useCallback, useEffect, useState } from "react";
import * as api from "../api/tauri";
import { PageLoading } from "../components/PageLoading";
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

const ACCOUNT_TYPES = [
  "asset",
  "liability",
  "equity",
  "income",
  "expense",
] as const;

export function AccountsPage() {
  const { push } = useToast();
  const [rows, setRows] = useState<Row[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState({ activeOnly: true });
  const [editingId, setEditingId] = useState<number | null>(null);
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [accountType, setAccountType] =
    useState<(typeof ACCOUNT_TYPES)[number]>("expense");
  const [isBankCash, setIsBankCash] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = (await api.accountList(filter)) as Row[];
      setRows(data);
    } catch (e) {
      push("error", errorMessage(e));
    } finally {
      setLoading(false);
    }
  }, [filter, push]);

  useEffect(() => {
    void load();
  }, [load]);

  function resetForm() {
    setEditingId(null);
    setCode("");
    setName("");
    setAccountType("expense");
    setIsBankCash(false);
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    const trimmedCode = code.trim();
    const trimmedName = name.trim();
    if (!trimmedCode || !trimmedName) {
      push("error", "Code and name are required.");
      return;
    }
    try {
      if (editingId === null) {
        await api.accountCreate({
          code: trimmedCode,
          name: trimmedName,
          accountType,
          isBankCash,
        });
        push("success", "Account created");
      } else {
        await api.accountUpdate({
          id: editingId,
          code: trimmedCode,
          name: trimmedName,
          accountType,
          isBankCash,
        });
        push("success", "Account updated");
      }
      resetForm();
      await load();
    } catch (err) {
      push("error", errorMessage(err));
    }
  }

  function startEdit(row: Row) {
    setEditingId(row.id);
    setCode(row.code);
    setName(row.name);
    setAccountType(row.accountType as (typeof ACCOUNT_TYPES)[number]);
    setIsBankCash(row.isBankCash);
  }

  async function deactivate(id: number) {
    if (!window.confirm("Deactivate this account?")) {
      return;
    }
    try {
      await api.accountDeactivate(id);
      push("success", "Account deactivated");
      if (editingId === id) {
        resetForm();
      }
      await load();
    } catch (err) {
      push("error", errorMessage(err));
    }
  }

  if (loading && rows.length === 0) {
    return <PageLoading label="Loading accounts…" />;
  }

  return (
    <div className="kb-page">
      <h1>Chart of accounts</h1>
      <form className="kb-form kb-form-stack" onSubmit={onSubmit}>
        <h2 className="kb-h2">
          {editingId === null ? "Add account" : "Edit account"}
        </h2>
        <label>
          Code
          <input
            value={code}
            onChange={(e) => setCode(e.target.value)}
            required
          />
        </label>
        <label>
          Name
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
          />
        </label>
        <label>
          Type
          <select
            value={accountType}
            onChange={(e) =>
              setAccountType(e.target.value as (typeof ACCOUNT_TYPES)[number])
            }
          >
            {ACCOUNT_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </label>
        <label className="kb-inline">
          <input
            type="checkbox"
            checked={isBankCash}
            onChange={(e) => setIsBankCash(e.target.checked)}
          />{" "}
          Bank / cash account
        </label>
        <div className="kb-row">
          <button type="submit">
            {editingId === null ? "Create" : "Save changes"}
          </button>
          {editingId !== null && (
            <button type="button" onClick={resetForm}>
              Cancel edit
            </button>
          )}
        </div>
      </form>
      <label className="kb-inline">
        <input
          type="checkbox"
          checked={filter.activeOnly}
          onChange={(e) => setFilter({ activeOnly: e.currentTarget.checked })}
        />{" "}
        Active only
      </label>
      {rows.length === 0 ? (
        <p className="kb-muted">No accounts match this filter.</p>
      ) : (
        <table className="kb-table">
          <caption className="kb-sr-only">Chart of accounts</caption>
          <thead>
            <tr>
              <th scope="col">Code</th>
              <th scope="col">Name</th>
              <th scope="col">Type</th>
              <th scope="col">Bank</th>
              <th scope="col">Actions</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id}>
                <td>{r.code}</td>
                <td>{r.name}</td>
                <td>{r.accountType}</td>
                <td>{r.isBankCash ? "Yes" : ""}</td>
                <td>
                  <button type="button" onClick={() => startEdit(r)}>
                    Edit
                  </button>{" "}
                  {r.isActive && (
                    <button type="button" onClick={() => void deactivate(r.id)}>
                      Deactivate
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <p className="kb-muted">
        Display example: {formatMoneyMinor(123456)} (informational)
      </p>
    </div>
  );
}
