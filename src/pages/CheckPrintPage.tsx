import { useEffect, useState } from "react";
import {
  Link,
  useNavigate,
  useParams,
  useSearchParams,
} from "react-router-dom";
import * as api from "../api/tauri";
import type { JsonObject } from "../api/tauri";
import { CheckPrintLayout } from "../components/CheckPrintLayout";
import { useToast } from "../context/ToastContext";
import {
  CHECK_STOCK_PRESETS,
  layoutFromStyle,
  presetForLayout,
  type CheckLayout,
} from "../lib/checkStock";
import { PageLoading } from "../components/PageLoading";

export function CheckPrintPage() {
  const { paymentId: paymentIdParam } = useParams();
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const { push, pushApiError } = useToast();
  const paymentId = Number(paymentIdParam);
  const [loading, setLoading] = useState(true);
  const [layout, setLayout] = useState<CheckLayout>("voucher_top");
  const [company, setCompany] = useState<JsonObject | null>(null);
  const [payment, setPayment] = useState<JsonObject | null>(null);

  useEffect(() => {
    if (!Number.isFinite(paymentId) || paymentId <= 0) {
      setLoading(false);
      return;
    }
    void (async () => {
      try {
        const [c, p] = await Promise.all([
          api.companyGet(),
          api.getVendorPayment(paymentId),
        ]);
        setCompany(c as JsonObject);
        setPayment(p as JsonObject);
        const styleParam = searchParams.get("style");
        if (styleParam) {
          setLayout(layoutFromStyle(styleParam));
        } else {
          setLayout(
            layoutFromStyle(
              String((c as JsonObject).defaultCheckStyle ?? "voucher_top"),
            ),
          );
        }
      } catch (e) {
        pushApiError(e, "CheckPrintPage");
      } finally {
        setLoading(false);
      }
    })();
  }, [paymentId, pushApiError, searchParams]);

  if (loading) {
    return <PageLoading />;
  }
  if (
    !company ||
    !payment ||
    !Number.isFinite(paymentId) ||
    paymentId <= 0 ||
    String(payment.paymentMethod ?? "") !== "check"
  ) {
    return (
      <div className="kb-page">
        <h1>Print check</h1>
        <p className="kb-error-text">
          {!payment
            ? "Payment not found."
            : String(payment.paymentMethod ?? "") !== "check"
              ? "This payment is not a check."
              : "Payment not found."}
        </p>
        <Link to="/checks/write">Back to write check</Link>
      </div>
    );
  }

  async function markPrinted() {
    try {
      await api.vendorPaymentMarkPrinted(paymentId);
      push("success", "Marked as printed");
    } catch (e) {
      pushApiError(e, "CheckPrintPage.markPrinted");
    }
  }

  const payee =
    String(payment.payeeName ?? "").trim() ||
    String(payment.vendorName ?? "Payee");

  function onStockChange(presetId: string) {
    const preset = CHECK_STOCK_PRESETS.find((p) => p.id === presetId);
    const next = preset?.layout ?? "voucher_top";
    setLayout(next);
    setSearchParams({ style: next });
  }

  return (
    <div className="kb-page kb-check-print-page">
      <div className="kb-check-print-toolbar kb-no-print">
        <h1>Print check</h1>
        <p className="kb-muted">
          Load blank check stock that matches the selected layout, then print.
          Do not scale the page — use 100% / actual size.
        </p>
        <div className="kb-actions">
          <label>
            Check stock
            <select
              value={presetForLayout(layout).id}
              onChange={(e) => onStockChange(e.target.value)}
              data-testid="check-print-stock"
            >
              {CHECK_STOCK_PRESETS.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.label}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            data-testid="check-print-button"
            onClick={() => window.print()}
          >
            Print
          </button>
          <button
            type="button"
            className="kb-button-secondary"
            data-testid="check-mark-printed"
            onClick={() => void markPrinted()}
          >
            Mark printed
          </button>
          <button
            type="button"
            className="kb-button-secondary"
            onClick={() => navigate("/checks/write")}
          >
            Done
          </button>
        </div>
      </div>

      <CheckPrintLayout
        layout={layout}
        data={{
          companyName: String(company.name ?? ""),
          legalName: (company.legalName as string | null | undefined) ?? null,
          addressLine1:
            (company.addressLine1 as string | null | undefined) ?? null,
          addressLine2:
            (company.addressLine2 as string | null | undefined) ?? null,
          city: (company.city as string | null | undefined) ?? null,
          region: (company.region as string | null | undefined) ?? null,
          postalCode: (company.postalCode as string | null | undefined) ?? null,
          payeeName: payee,
          paymentDate: String(payment.paymentDate ?? ""),
          amountMinor: Number(payment.amountMinor ?? 0),
          memo: (payment.memo as string | null | undefined) ?? null,
          checkNumber:
            (payment.checkNumber as string | null | undefined) ?? null,
          currencyCode: String(company.baseCurrencyCode ?? "USD"),
        }}
      />
    </div>
  );
}
