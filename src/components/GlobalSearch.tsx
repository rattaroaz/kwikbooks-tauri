import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useNavigate } from "react-router-dom";
import * as api from "../api/tauri";
import type { SearchHit } from "../api/tauri";
import { errorMessage } from "../types/errors";

const KIND_LABEL: Record<string, string> = {
  company: "Company",
  account: "Account",
  customer: "Customer",
  vendor: "Vendor",
  item: "Item",
  invoice: "Invoice",
  bill: "Bill",
  journal: "Journal",
  payment_customer: "Customer payment",
  payment_vendor: "Vendor payment",
};

function kindLabel(kind: string) {
  return KIND_LABEL[kind] ?? kind;
}

function shortcutLabel() {
  if (typeof navigator === "undefined") {
    return "Ctrl+K";
  }
  return /Mac|iPhone|iPad/i.test(navigator.userAgent) ? "⌘K" : "Ctrl+K";
}

export function GlobalSearch() {
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  const runSearch = useCallback(async (q: string) => {
    const t = q.trim();
    if (t.length === 0) {
      setHits([]);
      setError(null);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res = await api.globalSearch(t, 15);
      setHits(res.hits);
    } catch (e) {
      setError(errorMessage(e));
      setHits([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!open) {
      return;
    }
    const id = window.setTimeout(() => {
      void runSearch(query);
    }, 220);
    return () => window.clearTimeout(id);
  }, [query, open, runSearch]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen(true);
      }
      if (e.key === "Escape") {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    if (!open) {
      return;
    }
    inputRef.current?.focus();
    setQuery("");
    setHits([]);
    setError(null);
  }, [open]);

  const onPick = (h: SearchHit) => {
    navigate(h.path);
    setOpen(false);
  };

  const overlay =
    open &&
    createPortal(
      <div
        className="kb-search-overlay"
        role="dialog"
        aria-modal="true"
        aria-label="Search"
        onMouseDown={() => setOpen(false)}
      >
        <div
          className="kb-search-panel"
          ref={panelRef}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <div className="kb-search-field">
            <input
              ref={inputRef}
              type="search"
              autoComplete="off"
              placeholder="Search accounts, customers, invoices, memos, amounts…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
            {loading ? (
              <span className="kb-search-status" aria-live="polite">
                Searching…
              </span>
            ) : null}
          </div>
          {error ? <p className="kb-search-error">{error}</p> : null}
          <ul className="kb-search-hits" role="listbox">
            {hits.length === 0 && query.trim() && !loading && !error ? (
              <li className="kb-search-empty">No matches</li>
            ) : null}
            {hits.map((h) => (
              <li key={`${h.kind}-${h.id}`}>
                <button
                  type="button"
                  className="kb-search-hit"
                  role="option"
                  onClick={() => onPick(h)}
                >
                  <span className="kb-search-hit-kind">{kindLabel(h.kind)}</span>
                  <span className="kb-search-hit-title">{h.title}</span>
                  {h.subtitle ? (
                    <span className="kb-search-hit-sub">{h.subtitle}</span>
                  ) : null}
                </button>
              </li>
            ))}
          </ul>
          <p className="kb-search-hint kb-muted">
            Tip: search matches text in names, numbers, memos, line descriptions, and payment
            amounts. Press Esc to close.
          </p>
        </div>
      </div>,
      document.body,
    );

  return (
    <>
      <button
        type="button"
        className="kb-search-trigger"
        onClick={() => setOpen(true)}
        title="Search (Ctrl+K or ⌘K)"
      >
        Search…
        <kbd className="kb-kbd">{shortcutLabel()}</kbd>
      </button>
      {overlay}
    </>
  );
}
