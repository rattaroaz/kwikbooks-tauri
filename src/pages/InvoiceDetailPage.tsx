import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import * as api from "../api/tauri";
import type { JsonObject } from "../api/tauri";
import { formatMoneyMinor } from "../lib/money";
import { logContext } from "../lib/logContext";
import { useToast } from "../context/ToastContext";

type InvDetailData = {
  header: JsonObject;
  lines: JsonObject[];
};

const PAGE = "InvoiceDetailPage";

export function InvoiceDetailPage() {
  const { id } = useParams<{ id: string }>();
  const invoiceId = Number(id);
  const { push, pushApiError } = useToast();
  const [data, setData] = useState<InvDetailData | null>(null);

  const load = useCallback(async () => {
    try {
      const d = await api.getInvoice(invoiceId);
      setData({
        header: d.header,
        lines: d.lines as JsonObject[],
      });
    } catch (e) {
      pushApiError(e, logContext(PAGE, "load"));
    }
  }, [invoiceId, pushApiError]);

  useEffect(() => {
    void load();
  }, [load]);

  if (!data) {
    return <div className="kb-page">Loading…</div>;
  }

  const h = data.header as Record<string, unknown>;
  const status = String(h.status ?? "");

  return (
    <div className="kb-page">
      <p>
        <Link to="/invoices">← Invoices</Link>
      </p>
      <h1>Invoice {String(h.number ?? "")}</h1>
      <p className="kb-muted">
        Customer: {String(h.customerName ?? "")} · Status: {status} · Total:{" "}
        {formatMoneyMinor(Number(h.totalMinor ?? 0))}
      </p>
      <div className="kb-actions">
        {status === "draft" && (
          <button
            type="button"
            data-testid="invoice-mark-sent"
            onClick={async () => {
              try {
                await api.invoiceSetStatus(invoiceId, "sent");
                push("success", "Marked sent");
                await load();
              } catch (e) {
                pushApiError(e, logContext(PAGE, "setStatus"));
              }
            }}
          >
            Mark sent
          </button>
        )}
        {status === "sent" && !h.journalId && (
          <button
            type="button"
            data-testid="invoice-post-gl"
            onClick={async () => {
              try {
                await api.invoicePost(invoiceId);
                push("success", "Posted to GL");
                await load();
              } catch (e) {
                pushApiError(e, logContext(PAGE, "post"));
              }
            }}
          >
            Post to GL
          </button>
        )}
      </div>
      <table className="kb-table">
        <thead>
          <tr>
            <th>#</th>
            <th>Description</th>
            <th>Qty</th>
            <th>Line total</th>
          </tr>
        </thead>
        <tbody>
          {data.lines.map((line, i) => {
            const l = line as Record<string, unknown>;
            return (
              <tr key={i}>
                <td>{String(l.lineNumber ?? "")}</td>
                <td>{String(l.description ?? "")}</td>
                <td>{String(l.quantity ?? "")}</td>
                <td>{formatMoneyMinor(Number(l.lineTotalMinor ?? 0))}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
