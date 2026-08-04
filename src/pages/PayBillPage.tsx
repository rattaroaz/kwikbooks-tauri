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
    setSubmitting(true);
    let createdId: number | undefined;
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
      if (billId !== undefined) {
        try {
          const bill = await api.getBill(billId);
          const total = Number(bill.header?.totalMinor ?? 0);
          if (Number.isFinite(total) && amountMinor > total) {
            const ok = window.confirm(
              `Payment exceeds bill total (${total} minor units) and will overpay. Continue?`,
            );
            if (!ok) {
              return;
            }
          }
        } catch {
          // Backend validates bill on create.
        }
      }
      createdId = await api.vendorPaymentCreate({
        vendorId,
        bankAccountId,
        paymentDate: paymentDate.trim(),
        amountMinor,
        memo: memo.trim() || undefined,
        billId,
      });
      await api.vendorPaymentPost(createdId);
      push("success", "Vendor payment recorded and posted");
      setAmountStr("");
      setBillIdStr("");
      setMemo("");
    } catch (err) {
      if (createdId !== undefined) {
        try {
          await api.vendorPaymentDeleteUnposted(createdId);
        } catch {
          push(
            "error",
            `Payment #${createdId} was created but not posted; delete the draft or retry post.`,
          );
        }
      }
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
        bank credit). Optional bill id applies the whole payment to one posted
        bill with a vendor assigned; use a separate payment per bill to split.
        To print a paper check, use <Link to="/checks/write">Write check</Link>
        {billIdStr.trim() ? ` with bill ${billIdStr.trim()}` : ""}.
      </p>
      <p className="kb-muted">
        <Link
          to={
            billIdStr.trim() || vendorId
              ? `/checks/write?${[
                  vendorId ? `vendorId=${vendorId}` : "",
                  billIdStr.trim() ? `billId=${billIdStr.trim()}` : "",
                ]
                  .filter(Boolean)
                  .join("&")}`
              : "/checks/write"
          }
        >
          Pay with check…
        </Link>
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
          {submitting ? "Posting…" : "Record & post"}
        </button>
      </form>
    </div>
  );
}
