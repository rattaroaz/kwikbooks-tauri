import { FormEvent, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import * as api from "../api/tauri";
import { todayISODate } from "../lib/dates";
import { parseMinorInt } from "../lib/money";
import { requireValidISODate } from "../lib/validateDate";
import { useToast } from "../context/ToastContext";
import { errorMessage } from "../types/errors";

type Vendor = { id: number; displayName: string };
type BankAccount = {
  id: number;
  code: string;
  name: string;
  isBankCash: boolean;
};

export function PayBillPage() {
  const { push } = useToast();
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [banks, setBanks] = useState<BankAccount[]>([]);
  const [vendorId, setVendorId] = useState(0);
  const [bankAccountId, setBankAccountId] = useState(0);
  const [paymentDate, setPaymentDate] = useState(todayISODate());
  const [amountStr, setAmountStr] = useState("");
  const [billIdStr, setBillIdStr] = useState("");
  const [memo, setMemo] = useState("");

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
        push("error", errorMessage(e));
      }
    })();
  }, [push]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
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
      if (billId !== undefined && !Number.isFinite(billId)) {
        push("error", "Bill id must be a number.");
        return;
      }
      const paymentId = await api.vendorPaymentCreate({
        vendorId,
        bankAccountId,
        paymentDate: paymentDate.trim(),
        amountMinor,
        memo: memo.trim() || undefined,
        billId,
      });
      await api.vendorPaymentPost(paymentId);
      push("success", "Vendor payment recorded and posted");
      setAmountStr("");
      setBillIdStr("");
      setMemo("");
    } catch (err) {
      push("error", errorMessage(err));
    }
  }

  return (
    <div className="kb-page">
      <h1>Pay vendor</h1>
      <p className="kb-muted">
        Records a vendor payment and posts it to the general ledger (AP debit,
        bank credit).
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
        <button type="submit" disabled={banks.length === 0}>
          Record &amp; post
        </button>
      </form>
    </div>
  );
}
