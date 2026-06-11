import { FormEvent, useCallback, useEffect, useState } from "react";
import * as api from "../api/tauri";
import { useToast } from "../context/ToastContext";

type Vendor = {
  id: number;
  displayName: string;
  email?: string | null;
};

export function VendorsPage() {
  const { push, pushApiError } = useToast();
  const [rows, setRows] = useState<Vendor[]>([]);
  const [name, setName] = useState("");

  const load = useCallback(async () => {
    try {
      const data = (await api.listVendors()) as Vendor[];
      setRows(data);
    } catch (e) {
      pushApiError(e, "VendorsPage");
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
      await api.vendorCreate({ displayName: name.trim() });
      push("success", "Vendor created");
      setName("");
      await load();
    } catch (err) {
      pushApiError(err, "VendorsPage");
    }
  }

  return (
    <div className="kb-page">
      <h1>Vendors</h1>
      <form className="kb-form" onSubmit={onSubmit}>
        <label>
          Display name
          <input
            value={name}
            onChange={(e) => setName(e.currentTarget.value)}
            required
          />
        </label>
        <button type="submit">Add vendor</button>
      </form>
      <table className="kb-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Email</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((v) => (
            <tr key={v.id}>
              <td>{v.displayName}</td>
              <td>{v.email ?? ""}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
