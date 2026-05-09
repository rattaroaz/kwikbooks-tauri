import { FormEvent, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import * as api from "../api/tauri";
import { todayISODate } from "../lib/dates";
import { parseMinorInt } from "../lib/money";
import { useToast } from "../context/ToastContext";
import { errorMessage } from "../types/errors";
import { createScopedLogger } from "../lib/logger";

type Customer = { id: number; displayName: string };

const log = createScopedLogger("InvoiceNew");

export function InvoiceNewPage() {
  const nav = useNavigate();
  const { push } = useToast();
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [customerId, setCustomerId] = useState<number>(0);
  const [number, setNumber] = useState("");
  const [issueDate, setIssueDate] = useState(todayISODate());
  const [taxMinorStr, setTaxMinorStr] = useState("0");
  const [desc, setDesc] = useState("Line 1");
  const [qty, setQty] = useState("1");
  const [unitMinorStr, setUnitMinorStr] = useState("0");

  useEffect(() => {
    void (async () => {
      try {
        const c = (await api.listCustomers()) as Customer[];
        setCustomers(c);
        const first = c[0];
        if (first) {
          setCustomerId(first.id);
        }
      } catch (e) {
        push("error", errorMessage(e));
      }
    })();
  }, [push]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (customerId === 0) {
      push("error", "Add a customer first (Customers page).");
      return;
    }
    try {
      const taxMinor = parseMinorInt(taxMinorStr);
      const unitPriceMinor = parseMinorInt(unitMinorStr);
      const quantity = Number(qty);
      if (!Number.isFinite(quantity) || quantity <= 0) {
        push("error", "Invalid quantity");
        return;
      }
      const id = await api.invoiceCreate({
        customerId,
        number: number.trim(),
        issueDate,
        taxMinor,
        lines: [
          {
            description: desc.trim() || "Item",
            quantity,
            unitPriceMinor,
          },
        ],
      });
      void log.info(`draft invoice created id=${id}`);
      push("success", "Invoice created (draft)");
      nav(`/invoices/${id}`);
    } catch (err) {
      push("error", errorMessage(err));
    }
  }

  return (
    <div className="kb-page">
      <h1>New invoice</h1>
      <form className="kb-form kb-form-stack" onSubmit={onSubmit}>
        <label>
          Customer
          <select
            value={customerId || ""}
            onChange={(e) => setCustomerId(Number(e.currentTarget.value))}
          >
            {customers.map((c) => (
              <option key={c.id} value={c.id}>
                {c.displayName}
              </option>
            ))}
          </select>
        </label>
        <label>
          Invoice #
          <input
            value={number}
            onChange={(e) => setNumber(e.currentTarget.value)}
            required
          />
        </label>
        <label>
          Issue date (YYYY-MM-DD)
          <input
            value={issueDate}
            onChange={(e) => setIssueDate(e.currentTarget.value)}
            required
          />
        </label>
        <label>
          Tax (minor units)
          <input
            value={taxMinorStr}
            onChange={(e) => setTaxMinorStr(e.currentTarget.value)}
          />
        </label>
        <fieldset>
          <legend>Line 1</legend>
          <label>
            Description
            <input value={desc} onChange={(e) => setDesc(e.target.value)} />
          </label>
          <label>
            Qty
            <input value={qty} onChange={(e) => setQty(e.target.value)} />
          </label>
          <label>
            Unit price (minor units)
            <input
              value={unitMinorStr}
              onChange={(e) => setUnitMinorStr(e.target.value)}
            />
          </label>
        </fieldset>
        <button type="submit">Save draft</button>
      </form>
    </div>
  );
}
