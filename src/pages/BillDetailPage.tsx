import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import * as api from "../api/tauri";
import type { JsonObject } from "../api/tauri";
import { formatMoneyMinor } from "../lib/money";
import { logContext } from "../lib/logContext";
import { useToast } from "../context/ToastContext";

type BillDetailData = {
  header: JsonObject;
  lines: JsonObject[];
};

const PAGE = "BillDetailPage";

export function BillDetailPage() {
  const { id } = useParams<{ id: string }>();
  const billId = Number(id);
  const { push, pushApiError } = useToast();
  const [data, setData] = useState<BillDetailData | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const loadSeq = useRef(0);

  const load = useCallback(async () => {
    const seq = ++loadSeq.current;
    setData(null);
    setLoadError(null);
    if (!Number.isInteger(billId) || billId <= 0) {
      setLoadError("Invalid bill id.");
      return;
    }
    try {
      const d = await api.getBill(billId);
      if (seq !== loadSeq.current) {
        return;
      }
      setData({
        header: d.header,
        lines: d.lines as JsonObject[],
      });
    } catch (e) {
      if (seq !== loadSeq.current) {
        return;
      }
      setLoadError("Could not load this bill.");
      pushApiError(e, logContext(PAGE, "load"));
    }
  }, [billId, pushApiError]);

  useEffect(() => {
    void load();
  }, [load]);

  if (loadError) {
    return (
      <div className="kb-page">
        <p className="kb-error-text">{loadError}</p>
        <p>
          <Link to="/bills">← Bills</Link>
        </p>
      </div>
    );
  }

  if (!data) {
    return <div className="kb-page">Loading…</div>;
  }

  const h = data.header as Record<string, unknown>;
  const status = String(h.status ?? "");

  return (
    <div className="kb-page">
      <p>
        <Link to="/bills">← Bills</Link>
      </p>
      <h1>Bill {String(h.number ?? "")}</h1>
      <p className="kb-muted">
        Status: {status} · Total: {formatMoneyMinor(Number(h.totalMinor ?? 0))}
      </p>
      <div className="kb-actions">
        {status === "draft" && (
          <button
            type="button"
            data-testid="bill-mark-open"
            onClick={async () => {
              try {
                await api.billSetStatus(billId, "open");
                push("success", "Marked open");
                await load();
              } catch (e) {
                pushApiError(e, logContext(PAGE, "setStatus"));
              }
            }}
          >
            Mark open
          </button>
        )}
        {status === "open" && !h.journalId && (
          <button
            type="button"
            data-testid="bill-post-gl"
            onClick={async () => {
              try {
                await api.billPost(billId);
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
            <th>Amount</th>
          </tr>
        </thead>
        <tbody>
          {data.lines.map((line, i) => {
            const l = line as Record<string, unknown>;
            return (
              <tr key={i}>
                <td>{String(l.lineNumber ?? "")}</td>
                <td>{String(l.description ?? "")}</td>
                <td>{formatMoneyMinor(Number(l.amountMinor ?? 0))}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
