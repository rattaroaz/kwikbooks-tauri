import { FormEvent, useEffect, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import * as api from "../api/tauri";
import type { JsonObject } from "../api/tauri";
import { todayISODate } from "../lib/dates";
import { parseMinorInt, formatMoneyMinor } from "../lib/money";
import { requireValidISODate } from "../lib/validateDate";
import { useToast } from "../context/ToastContext";
import {
  CHECK_STOCK_PRESETS,
  layoutFromStyle,
  presetForLayout,
  type CheckLayout,
} from "../lib/checkStock";

type Vendor = { id: number; displayName: string };
type BankAccount = {
  id: number;
  code: string;
  name: string;
  isBankCash: boolean;
};
type VendorPaymentRow = {
  id: number;
  vendorName: string;
  paymentDate: string;
  amountMinor: number;
  checkNumber?: string | null;
  paymentMethod: string;
  payeeName?: string | null;
};

export function WriteCheckPage() {
  const { push, pushApiError } = useToast();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [banks, setBanks] = useState<BankAccount[]>([]);
  const [checks, setChecks] = useState<VendorPaymentRow[]>([]);
  const [vendorId, setVendorId] = useState(0);
  const [bankAccountId, setBankAccountId] = useState(0);
  const [paymentDate, setPaymentDate] = useState(todayISODate());
  const [amountStr, setAmountStr] = useState("");
  const [billIdStr, setBillIdStr] = useState("");
  const [memo, setMemo] = useState("");
  const [payeeOverride, setPayeeOverride] = useState("");
  const [checkNumber, setCheckNumber] = useState("");
  const [stockId, setStockId] = useState(
    () => CHECK_STOCK_PRESETS[1]?.id ?? "deluxe_qb_top",
  );
  const [currency, setCurrency] = useState("USD");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    void (async () => {
      try {
        const [v, accts, company, payments] = await Promise.all([
          api.listVendors(),
          api.accountList({ activeOnly: true }),
          api.companyGet(),
          api.listVendorPayments(),
        ]);
        const vend = v as Vendor[];
        setVendors(vend);
        const qVendor = Number(searchParams.get("vendorId") ?? "");
        if (Number.isFinite(qVendor) && qVendor > 0) {
          setVendorId(qVendor);
        } else if (vend[0]) {
          setVendorId(vend[0].id);
        }
        const bankRows = (accts as BankAccount[]).filter((a) => a.isBankCash);
        setBanks(bankRows);
        if (bankRows[0]) {
          setBankAccountId(bankRows[0].id);
        }
        const c = company as JsonObject;
        setCurrency(String(c.baseCurrencyCode ?? "USD"));
        setCheckNumber(String(c.nextCheckNumber ?? "1000"));
        const layout = layoutFromStyle(String(c.defaultCheckStyle ?? "voucher_top"));
        setStockId(presetForLayout(layout).id);
        const qBill = searchParams.get("billId");
        if (qBill) {
          setBillIdStr(qBill);
        }
        setChecks(
          (payments as VendorPaymentRow[]).filter(
            (p) => p.paymentMethod === "check",
          ),
        );
      } catch (e) {
        pushApiError(e, "WriteCheckPage");
      }
    })();
  }, [pushApiError, searchParams]);

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
              `Check amount exceeds bill total (${total} minor units) and will overpay. Continue?`,
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
        paymentMethod: "check",
        checkNumber: checkNumber.trim() || undefined,
        payeeName: payeeOverride.trim() || undefined,
      });
      await api.vendorPaymentPost(createdId);
      push("success", "Check payment recorded and posted");
      const preset = CHECK_STOCK_PRESETS.find((p) => p.id === stockId);
      const layout: CheckLayout = preset?.layout ?? "voucher_top";
      navigate(`/checks/print/${createdId}?style=${layout}`);
    } catch (err) {
      if (createdId !== undefined) {
        try {
          await api.vendorPaymentDeleteUnposted(createdId);
        } catch {
          push(
            "error",
            `Check payment #${createdId} was created but not posted; delete the draft or retry post.`,
          );
        }
      }
      pushApiError(err, "WriteCheckPage");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="kb-page">
      <h1>Write check</h1>
      <p className="kb-muted">
        Records a vendor payment as a check, posts it to the ledger, then opens
        a print preview for blank voucher stock. MICR lines stay on the paper —
        this app only prints payee, amount, date, and memo.
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
            data-testid="write-check-vendor"
          >
            {vendors.map((v) => (
              <option key={v.id} value={v.id}>
                {v.displayName}
              </option>
            ))}
          </select>
        </label>
        <label>
          Payee on check (optional override)
          <input
            value={payeeOverride}
            onChange={(e) => setPayeeOverride(e.target.value)}
            placeholder="Defaults to vendor name"
            data-testid="write-check-payee"
          />
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
          Check date
          <input
            type="date"
            value={paymentDate}
            onChange={(e) => setPaymentDate(e.target.value)}
            required
          />
        </label>
        <label>
          Check number
          <input
            value={checkNumber}
            onChange={(e) => setCheckNumber(e.target.value)}
            data-testid="write-check-number"
          />
        </label>
        <label>
          Amount (minor units)
          <input
            value={amountStr}
            onChange={(e) => setAmountStr(e.target.value)}
            required
            data-testid="write-check-amount"
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
        <label>
          Check stock
          <select
            value={stockId}
            onChange={(e) => setStockId(e.target.value)}
            data-testid="write-check-stock"
          >
            {CHECK_STOCK_PRESETS.map((p) => (
              <option key={p.id} value={p.id}>
                {p.label}
              </option>
            ))}
          </select>
        </label>
        <p className="kb-muted">
          {CHECK_STOCK_PRESETS.find((p) => p.id === stockId)?.help}
        </p>
        <button
          type="submit"
          disabled={banks.length === 0 || submitting}
          data-testid="write-check-submit"
        >
          {submitting ? "Posting…" : "Record, post & print"}
        </button>
      </form>

      <section className="kb-settings-extra">
        <h2>Reprint checks</h2>
        {checks.length === 0 ? (
          <p className="kb-muted">No check payments yet.</p>
        ) : (
          <table className="kb-table">
            <thead>
              <tr>
                <th>Date</th>
                <th>Check #</th>
                <th>Payee</th>
                <th>Amount</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {checks.map((p) => (
                <tr key={p.id}>
                  <td>{p.paymentDate}</td>
                  <td>{p.checkNumber ?? "—"}</td>
                  <td>{p.payeeName?.trim() || p.vendorName}</td>
                  <td>{formatMoneyMinor(p.amountMinor, currency)}</td>
                  <td>
                    <Link to={`/checks/print/${p.id}`}>Print</Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}
