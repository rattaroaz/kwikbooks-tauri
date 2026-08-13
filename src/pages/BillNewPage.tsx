import { FormEvent, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import * as api from "../api/tauri";
import { todayISODate } from "../lib/dates";
import { parseMinorInt } from "../lib/money";
import { useToast } from "../context/ToastContext";
import { requireValidISODate } from "../lib/validateDate";
import { logContext } from "../lib/logContext";
import { createScopedLogger } from "../lib/logger";

type Vendor = { id: number; displayName: string };
type AccountRow = {
  id: number;
  code: string;
  name: string;
  accountType: string;
};

const log = createScopedLogger("BillNew");
const PAGE = "BillNewPage";

export function BillNewPage() {
  const nav = useNavigate();
  const { push, pushApiError } = useToast();
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [expenseAccounts, setExpenseAccounts] = useState<AccountRow[]>([]);
  const [vendorId, setVendorId] = useState<number | "">("");
  const [payeeName, setPayeeName] = useState("");
  const [number, setNumber] = useState("");
  const [issueDate, setIssueDate] = useState(todayISODate());
  const [desc, setDesc] = useState("Expense");
  const [amountStr, setAmountStr] = useState("0");
  const [expenseId, setExpenseId] = useState<number>(0);

  useEffect(() => {
    void (async () => {
      try {
        const [v, accts] = await Promise.all([
          api.listVendors(),
          api.accountList({ accountType: "expense", activeOnly: true }),
        ]);
        setVendors(v as Vendor[]);
        const exp = (accts as AccountRow[]).filter(
          (a) => a.accountType === "expense",
        );
        setExpenseAccounts(exp);
        const first = exp[0];
        if (first) {
          setExpenseId(first.id);
        }
      } catch (e) {
        pushApiError(e, logContext(PAGE, "load"));
      }
    })();
  }, [pushApiError]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (expenseId === 0) {
      push("error", "No expense account available — check Chart of accounts.");
      return;
    }
    if (vendorId === "" && !payeeName.trim()) {
      push("error", "Select a vendor or enter a payee name.");
      return;
    }
    const dateErr = requireValidISODate("Issue date", issueDate);
    if (dateErr) {
      push("error", dateErr);
      return;
    }
    try {
      const amountMinor = parseMinorInt(amountStr);
      if (amountMinor <= 0) {
        push("error", "Amount must be greater than zero.");
        return;
      }
      const id = await api.billCreate({
        vendorId: vendorId === "" ? undefined : vendorId,
        payeeName: payeeName.trim() || undefined,
        number: number.trim(),
        issueDate,
        lines: [
          {
            description: desc.trim() || "Line",
            amountMinor,
            expenseAccountId: expenseId,
          },
        ],
      });
      void log.info(`draft bill created id=${id}`);
      push("success", "Bill created (draft)");
      nav(`/bills/${id}`);
    } catch (err) {
      pushApiError(err, logContext(PAGE, "create"));
    }
  }

  return (
    <div className="kb-page">
      <h1>New bill</h1>
      <form className="kb-form kb-form-stack" onSubmit={onSubmit}>
        <label>
          Vendor (optional)
          <select
            value={vendorId === "" ? "" : vendorId}
            onChange={(e) =>
              setVendorId(
                e.currentTarget.value === ""
                  ? ""
                  : Number(e.currentTarget.value),
              )
            }
          >
            <option value="">—</option>
            {vendors.map((v) => (
              <option key={v.id} value={v.id}>
                {v.displayName}
              </option>
            ))}
          </select>
        </label>
        <label>
          Payee name (if no vendor)
          <input
            value={payeeName}
            onChange={(e) => setPayeeName(e.target.value)}
          />
        </label>
        <label>
          Bill #
          <input
            value={number}
            onChange={(e) => setNumber(e.currentTarget.value)}
            required
          />
        </label>
        <label>
          Issue date
          <input
            type="date"
            value={issueDate}
            onChange={(e) => setIssueDate(e.currentTarget.value)}
            required
          />
        </label>
        <label>
          Expense account
          <select
            value={expenseId || ""}
            onChange={(e) => setExpenseId(Number(e.currentTarget.value))}
          >
            {expenseAccounts.map((a) => (
              <option key={a.id} value={a.id}>
                {a.code} — {a.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          Description
          <input value={desc} onChange={(e) => setDesc(e.target.value)} />
        </label>
        <label>
          Amount (minor units)
          <input
            value={amountStr}
            onChange={(e) => setAmountStr(e.target.value)}
          />
        </label>
        <button type="submit" data-testid="bill-new-save-draft">
          Save draft
        </button>
      </form>
    </div>
  );
}
