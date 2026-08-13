import { FormEvent, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import * as api from "../api/tauri";
import { todayISODate } from "../lib/dates";
import { parseMinorInt } from "../lib/money";
import { requireValidISODate } from "../lib/validateDate";
import { logContext } from "../lib/logContext";
import { useToast } from "../context/ToastContext";

type Customer = { id: number; displayName: string };
type BankAccount = {
  id: number;
  code: string;
  name: string;
  isBankCash: boolean;
};

const PAGE = "ReceivePaymentPage";

export function ReceivePaymentPage() {
  const { push, pushApiError } = useToast();
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [banks, setBanks] = useState<BankAccount[]>([]);
  const [customerId, setCustomerId] = useState(0);
  const [bankAccountId, setBankAccountId] = useState(0);
  const [paymentDate, setPaymentDate] = useState(todayISODate());
  const [amountStr, setAmountStr] = useState("");
  const [invoiceIdStr, setInvoiceIdStr] = useState("");
  const [memo, setMemo] = useState("");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    void (async () => {
      try {
        const [c, accts] = await Promise.all([
          api.listCustomers(),
          api.accountList({ activeOnly: true }),
        ]);
        const cust = c as Customer[];
        setCustomers(cust);
        if (cust[0]) {
          setCustomerId(cust[0].id);
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
    if (customerId === 0 || bankAccountId === 0) {
      push("error", "Select a customer and bank account.");
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
      const invoiceId =
        invoiceIdStr.trim() === "" ? undefined : Number(invoiceIdStr);
      if (
        invoiceId !== undefined &&
        (!Number.isInteger(invoiceId) || invoiceId <= 0)
      ) {
        push("error", "Invoice id must be a positive whole number.");
        return;
      }
      createdId = await api.customerPaymentCreate({
        customerId,
        bankAccountId,
        paymentDate: paymentDate.trim(),
        amountMinor,
        memo: memo.trim() || undefined,
        invoiceId,
      });
      await api.customerPaymentPost(createdId);
      push("success", "Customer payment recorded and posted");
      setAmountStr("");
      setInvoiceIdStr("");
      setMemo("");
    } catch (err) {
      if (createdId !== undefined) {
        try {
          await api.customerPaymentDeleteUnposted(createdId);
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
      <h1>Receive payment</h1>
      <p className="kb-muted">
        Records a customer payment and posts it to the general ledger (bank
        debit, AR credit). Applying to an invoice reduces that invoice's open
        balance and marks it paid when fully covered.
      </p>
      {banks.length === 0 && (
        <p className="kb-error-text">
          No bank/cash accounts found. Mark an account as bank/cash on the{" "}
          <Link to="/accounts">chart of accounts</Link>.
        </p>
      )}
      <form className="kb-form kb-form-stack" onSubmit={onSubmit}>
        <label>
          Customer
          <select
            value={customerId || ""}
            onChange={(e) => setCustomerId(Number(e.target.value))}
          >
            {customers.map((c) => (
              <option key={c.id} value={c.id}>
                {c.displayName}
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
          Apply to invoice id (optional)
          <input
            value={invoiceIdStr}
            onChange={(e) => setInvoiceIdStr(e.target.value)}
            placeholder="e.g. 1"
          />
        </label>
        <label>
          Memo
          <input value={memo} onChange={(e) => setMemo(e.target.value)} />
        </label>
        <button
          type="submit"
          data-testid="receive-payment-submit"
          disabled={banks.length === 0 || submitting}
        >
          {submitting ? "Posting…" : "Record & post"}
        </button>
      </form>
    </div>
  );
}
