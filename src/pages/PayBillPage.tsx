import { FormEvent, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import * as api from "../api/tauri";
import { todayISODate } from "../lib/dates";
import { parseMinorInt } from "../lib/money";
import { requireValidISODate } from "../lib/validateDate";
import { logContext } from "../lib/logContext";
import { useToast } from "../context/ToastContext";

type Vendor = { id: number; displayName: string };
type BankAccount = {
  id: number;
  code: string;
  name: string;
  isBankCash: boolean;
};

const PAGE = "PayBillPage";

export function PayBillPage() {
  const { push, pushApiError } = useToast();
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [banks, setBanks] = useState<BankAccount[]>([]);
  const [vendorId, setVendorId] = useState(0);
  const [bankAccountId, setBankAccountId] = useState(0);
  const [paymentDate, setPaymentDate] = useState(todayISODate());
  const [amountStr, setAmountStr] = useState("");
  const [billIdStr, setBillIdStr] = useState("");
  const [memo, setMemo] = useState("");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    void (async () => {
      try {
        const [v, accts] = await Promise.all([
          api.listVendors(),
          api.accountList({ activeOnly: true }),
        ]);
        const vend = v as Vendor[];
        setVendors(vend);
        if (vend[0]) {
          setVendorId(vend[0].id);
        }
        const bankRows = (accts as BankAccount[]).filter((a) => a.isBankCash);
        setBanks(bankRows);
        if (bankRows[0]) {
          setBankAccountId(bankRows[0].id);
        }
      } catch (e) {
        pushApiError(e, logContext(PAGE, "load"));
      }
    })();
  }, [pushApiError]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (submitting) {
      return;
    }
    const dateErr = requireValidISODate("Payment date", paymentDate);
    if (dateErr) {
      push("error", dateErr);
      return;
    }
    if (vendorId === 0 || bankAccountId === 0) {
      push("error", "Select a vendor and bank account.");
      return;
    }
    try {
      const amountMinor = parseMinorInt(amountStr);
      if (amountMinor <= 0) {
        push("error", "Amount must be greater than zero.");
        return;
      }
      const billId = billIdStr.trim() === "" ? undefined : Number(billIdStr);
      if (billId !== undefined && (!Number.isInteger(billId) || billId <= 0)) {
        push("error", "Bill id must be a positive whole number.");
        return;
      }
      setSubmitting(true);
      await api.vendorPaymentCreate({
        vendorId,
        bankAccountId,
        paymentDate: paymentDate.trim(),
        amountMinor,
        memo: memo.trim() || undefined,
        billId,
      });
      push("success", "Vendor payment recorded and posted");
      setAmountStr("");
      setBillIdStr("");
      setMemo("");
    } catch (err) {
      pushApiError(err, logContext(PAGE, "submit"));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="kb-page">
      <h1>Pay vendor</h1>
      <p className="kb-muted">
        Records a vendor payment and posts it to the general ledger (AP debit,
        bank credit). Applying to a bill reduces that bill's open balance and
        marks it paid when fully covered.
      </p>
      {banks.length === 0 && (
        <p className="kb-error-text">
          No bank/cash accounts found. Mark an account as bank/cash on the{" "}
          <Link to="/accounts">chart of accounts</Link>.
        </p>
      )}
      <form className="kb-form kb-form-stack" onSubmit={onSubmit}>
        <label>
          Vendor
          <select
            value={vendorId || ""}
            onChange={(e) => setVendorId(Number(e.target.value))}
          >
            {vendors.map((v) => (
              <option key={v.id} value={v.id}>
                {v.displayName}
              </option>
            ))}
          </select>
        </label>
        <label>
          Bank account
          <select
            value={bankAccountId || ""}
            onChange={(e) => setBankAccountId(Number(e.target.value))}
            disabled={banks.length === 0}
          >
            {banks.map((b) => (
              <option key={b.id} value={b.id}>
                {b.code} — {b.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          Payment date
          <input
            type="date"
            value={paymentDate}
            onChange={(e) => setPaymentDate(e.target.value)}
            required
          />
        </label>
        <label>
          Amount (minor units)
          <input
            value={amountStr}
            onChange={(e) => setAmountStr(e.target.value)}
            required
          />
        </label>
        <label>
          Apply to bill id (optional)
          <input
            value={billIdStr}
            onChange={(e) => setBillIdStr(e.target.value)}
            placeholder="e.g. 1"
          />
        </label>
        <label>
          Memo
          <input value={memo} onChange={(e) => setMemo(e.target.value)} />
        </label>
        <button
          type="submit"
          data-testid="pay-bill-submit"
          disabled={banks.length === 0 || submitting}
        >
          Record &amp; post
        </button>
      </form>
    </div>
  );
}
