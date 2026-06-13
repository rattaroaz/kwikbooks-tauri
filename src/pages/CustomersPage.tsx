import { FormEvent, useCallback, useEffect, useState } from "react";
import * as api from "../api/tauri";
import { logContext } from "../lib/logContext";
import { useToast } from "../context/ToastContext";

type Customer = {
  id: number;
  displayName: string;
  email?: string | null;
  phone?: string | null;
  termsDays: number;
};

const PAGE = "CustomersPage";

export function CustomersPage() {
  const { push, pushApiError } = useToast();
  const [rows, setRows] = useState<Customer[]>([]);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");

  const load = useCallback(async () => {
    try {
      const data = (await api.listCustomers()) as Customer[];
      setRows(data);
    } catch (e) {
      pushApiError(e, logContext(PAGE, "load"));
    }
  }, [pushApiError]);

  useEffect(() => {
    void load();
  }, [load]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!name.trim()) {
      push("error", "Name required");
      return;
    }
    try {
      await api.customerCreate({
        displayName: name.trim(),
        email: email.trim() || undefined,
      });
      push("success", "Customer created");
      setName("");
      setEmail("");
      await load();
    } catch (err) {
      pushApiError(err, logContext(PAGE, "create"));
    }
  }

  return (
    <div className="kb-page">
      <h1>Customers</h1>
      <form className="kb-form" onSubmit={onSubmit}>
        <label>
          Display name
          <input
            value={name}
            onChange={(e) => setName(e.currentTarget.value)}
            required
          />
        </label>
        <label>
          Email (optional)
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.currentTarget.value)}
          />
        </label>
        <button type="submit">Add customer</button>
      </form>
      <table className="kb-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Email</th>
            <th>Terms (days)</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((c) => (
            <tr key={c.id}>
              <td>{c.displayName}</td>
              <td>{c.email ?? ""}</td>
              <td>{c.termsDays}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
