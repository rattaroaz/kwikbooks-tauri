import { FormEvent, useCallback, useEffect, useState } from "react";
import * as api from "../api/tauri";
import { useToast } from "../context/ToastContext";
import { errorMessage } from "../types/errors";

type Customer = {
  id: number;
  displayName: string;
  email?: string | null;
  phone?: string | null;
  termsDays: number;
};

export function CustomersPage() {
  const { push } = useToast();
  const [rows, setRows] = useState<Customer[]>([]);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");

  const load = useCallback(async () => {
    try {
      const data = (await api.listCustomers()) as Customer[];
      setRows(data);
    } catch (e) {
      push("error", errorMessage(e));
    }
  }, [push]);

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
      push("error", errorMessage(err));
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
