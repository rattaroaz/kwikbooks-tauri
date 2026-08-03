import { useState } from "react";
import * as api from "../api/tauri";
import type { JsonObject } from "../api/tauri";
import { downloadTextFile, rowsToCsv } from "../lib/csv";
import { todayISODate } from "../lib/dates";
import { formatMoneyMinor, sumMinor } from "../lib/money";
import { logContext } from "../lib/logContext";
import { requireValidISODate } from "../lib/validateDate";
import { useToast } from "../context/ToastContext";

type Tab = "pl" | "bs" | "tb" | "ar" | "ap" | "gl";

type AccountOption = { id: number; code: string; name: string };

const PAGE = "ReportsPage";

export function ReportsPage() {
  const { push, pushApiError } = useToast();
  const [tab, setTab] = useState<Tab>("pl");
  const [from, setFrom] = useState(todayISODate());
  const [to, setTo] = useState(todayISODate());
  const [asOf, setAsOf] = useState(todayISODate());
  const [pl, setPl] = useState<JsonObject | null>(null);
  const [bs, setBs] = useState<JsonObject | null>(null);
  const [tb, setTb] = useState<unknown[] | null>(null);
  const [ar, setAr] = useState<unknown[] | null>(null);
  const [ap, setAp] = useState<unknown[] | null>(null);
  const [glAccountId, setGlAccountId] = useState<number>(0);
  const [glAccounts, setGlAccounts] = useState<AccountOption[]>([]);
  const [gl, setGl] = useState<unknown[] | null>(null);

  function requireRange(): boolean {
    const fromErr = requireValidISODate("From date", from);
    if (fromErr) {
      push("error", fromErr);
      return false;
    }
    const toErr = requireValidISODate("To date", to);
    if (toErr) {
      push("error", toErr);
      return false;
    }
    return true;
  }

  async function loadPl() {
    if (!requireRange()) {
      return;
    }
    try {
      const r = await api.reportProfitLoss(from, to);
      setPl(r);
      push("success", "Profit & loss loaded");
    } catch (e) {
      pushApiError(e, logContext(PAGE, "loadPl"));
    }
  }

  async function loadBs() {
    const asOfErr = requireValidISODate("As-of date", asOf);
    if (asOfErr) {
      push("error", asOfErr);
      return;
    }
    try {
      const r = await api.reportBalanceSheet(asOf);
      setBs(r);
      push("success", "Balance sheet loaded");
    } catch (e) {
      pushApiError(e, logContext(PAGE, "loadBs"));
    }
  }

  async function loadTb() {
    if (!requireRange()) {
      return;
    }
    try {
      const r = await api.reportTrialBalance(from, to);
      setTb(r);
      push("success", "Trial balance loaded");
    } catch (e) {
      pushApiError(e, logContext(PAGE, "loadTb"));
    }
  }

  async function loadAr() {
    try {
      const r = await api.reportArOpen();
      setAr(r);
      push("success", "AR summary loaded");
    } catch (e) {
      pushApiError(e, logContext(PAGE, "loadAr"));
    }
  }

  async function loadAp() {
    try {
      const r = await api.reportApOpen();
      setAp(r);
      push("success", "AP summary loaded");
    } catch (e) {
      pushApiError(e, logContext(PAGE, "loadAp"));
    }
  }

  async function loadGlAccounts() {
    try {
      const rows = (await api.accountList({
        activeOnly: true,
      })) as AccountOption[];
      setGlAccounts(rows);
      if (rows[0] && glAccountId === 0) {
        setGlAccountId(rows[0].id);
      }
    } catch (e) {
      pushApiError(e, logContext(PAGE, "loadGlAccounts"));
    }
  }

  async function loadGl() {
    if (glAccountId === 0) {
      push("error", "Select an account for the general ledger.");
      return;
    }
    if (!requireRange()) {
      return;
    }
    try {
      const r = await api.reportGeneralLedger(glAccountId, from, to);
      setGl(r);
      push("success", "General ledger loaded");
    } catch (e) {
      pushApiError(e, logContext(PAGE, "loadGl"));
    }
  }

  function csvPl() {
    if (!pl) {
      return;
    }
    const income = (pl.incomeLines as JsonObject[]) ?? [];
    const expense = (pl.expenseLines as JsonObject[]) ?? [];
    const rows: string[][] = [
      ["section", "code", "name", "amountMinor"],
      ...income.map((x) => [
        "income",
        String(x.code ?? ""),
        String(x.name ?? ""),
        String(x.amountMinor ?? ""),
      ]),
      ...expense.map((x) => [
        "expense",
        String(x.code ?? ""),
        String(x.name ?? ""),
        String(x.amountMinor ?? ""),
      ]),
      ["total", "", "netIncomeMinor", String(pl.netIncomeMinor ?? "")],
    ];
    downloadTextFile(
      `profit-loss_${from}_${to}.csv`,
      rowsToCsv(rows),
      "text/csv;charset=utf-8",
    );
  }

  function csvBs() {
    if (!bs) {
      return;
    }
    const rows: string[][] = [["section", "code", "name", "balanceMinor"]];
    for (const sec of ["assets", "liabilities", "equity"] as const) {
      const lines = (bs[sec] as JsonObject[]) ?? [];
      for (const x of lines) {
        rows.push([
          sec,
          String(x.code ?? ""),
          String(x.name ?? ""),
          String(x.balanceMinor ?? ""),
        ]);
      }
    }
    downloadTextFile(
      `balance-sheet_${asOf}.csv`,
      rowsToCsv(rows),
      "text/csv;charset=utf-8",
    );
  }

  function csvTb() {
    if (!tb) {
      return;
    }
    const rows: string[][] = [
      ["code", "name", "accountType", "debitMinor", "creditMinor", "netMinor"],
    ];
    for (const x of tb as JsonObject[]) {
      rows.push([
        String(x.code ?? ""),
        String(x.name ?? ""),
        String(x.accountType ?? ""),
        String(x.debitMinor ?? ""),
        String(x.creditMinor ?? ""),
        String(x.netMinor ?? ""),
      ]);
    }
    downloadTextFile(
      `trial-balance_${from}_${to}.csv`,
      rowsToCsv(rows),
      "text/csv;charset=utf-8",
    );
  }

  function csvAr() {
    if (!ar) {
      return;
    }
    const rows: string[][] = [["customerId", "displayName", "openMinor"]];
    for (const x of ar as JsonObject[]) {
      rows.push([
        String(x.customerId ?? ""),
        String(x.displayName ?? ""),
        String(x.openMinor ?? ""),
      ]);
    }
    downloadTextFile(
      `ar-open_${todayISODate()}.csv`,
      rowsToCsv(rows),
      "text/csv;charset=utf-8",
    );
  }

  return (
    <div className="kb-page">
      <h1>Reports</h1>
      <p className="kb-muted">
        Dates are ISO <code>YYYY-MM-DD</code>. Amounts in sheets are{" "}
        <strong>minor units</strong> (e.g. cents); UI formats display currency
        only.
      </p>
      <div className="kb-tabs">
        {(
          [
            ["pl", "P / L"],
            ["bs", "Balance sheet"],
            ["tb", "Trial balance"],
            ["ar", "AR summary"],
            ["ap", "AP summary"],
            ["gl", "General ledger"],
          ] as const
        ).map(([k, label]) => (
          <button
            key={k}
            type="button"
            className={tab === k ? "kb-tab kb-tab-active" : "kb-tab"}
            onClick={() => setTab(k)}
          >
            {label}
          </button>
        ))}
      </div>

      {tab === "pl" && (
        <section className="kb-report">
          <div className="kb-row">
            <label>
              From
              <input value={from} onChange={(e) => setFrom(e.target.value)} />
            </label>
            <label>
              To
              <input value={to} onChange={(e) => setTo(e.target.value)} />
            </label>
            <button
              type="button"
              data-testid="reports-pl-run"
              onClick={() => void loadPl()}
            >
              Run
            </button>
            <button type="button" onClick={csvPl} disabled={!pl}>
              Export CSV
            </button>
          </div>
          {pl && (
            <div className="kb-report-body">
              <h3>Income</h3>
              <table className="kb-table">
                <tbody>
                  {((pl.incomeLines as JsonObject[]) ?? []).map((x, i) => (
                    <tr key={i}>
                      <td>{String(x.code)}</td>
                      <td>{String(x.name)}</td>
                      <td>{formatMoneyMinor(Number(x.amountMinor))}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <h3>Expenses</h3>
              <table className="kb-table">
                <tbody>
                  {((pl.expenseLines as JsonObject[]) ?? []).map((x, i) => (
                    <tr key={i}>
                      <td>{String(x.code)}</td>
                      <td>{String(x.name)}</td>
                      <td>{formatMoneyMinor(Number(x.amountMinor))}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <p>
                <strong>Net income:</strong>{" "}
                {formatMoneyMinor(Number(pl.netIncomeMinor ?? 0))}
              </p>
            </div>
          )}
        </section>
      )}

      {tab === "bs" && (
        <section className="kb-report">
          <div className="kb-row">
            <label>
              As of
              <input value={asOf} onChange={(e) => setAsOf(e.target.value)} />
            </label>
            <button type="button" onClick={() => void loadBs()}>
              Run
            </button>
            <button type="button" onClick={csvBs} disabled={!bs}>
              Export CSV
            </button>
          </div>
          {bs && (
            <div className="kb-report-body">
              {(["assets", "liabilities", "equity"] as const).map((sec) => (
                <div key={sec}>
                  <h3>{sec}</h3>
                  <table className="kb-table">
                    <tbody>
                      {((bs[sec] as JsonObject[]) ?? []).map((x, i) => (
                        <tr key={i}>
                          <td>{String(x.code)}</td>
                          <td>{String(x.name)}</td>
                          <td>{formatMoneyMinor(Number(x.balanceMinor))}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ))}
            </div>
          )}
        </section>
      )}

      {tab === "tb" && (
        <section className="kb-report">
          <div className="kb-row">
            <label>
              From
              <input value={from} onChange={(e) => setFrom(e.target.value)} />
            </label>
            <label>
              To
              <input value={to} onChange={(e) => setTo(e.target.value)} />
            </label>
            <button type="button" onClick={() => void loadTb()}>
              Run
            </button>
            <button type="button" onClick={csvTb} disabled={!tb}>
              Export CSV
            </button>
          </div>
          {tb && (
            <table className="kb-table">
              <thead>
                <tr>
                  <th>Code</th>
                  <th>Name</th>
                  <th>Type</th>
                  <th>Debit</th>
                  <th>Credit</th>
                  <th>Net</th>
                </tr>
              </thead>
              <tbody>
                {(tb as JsonObject[]).map((x, i) => (
                  <tr key={i}>
                    <td>{String(x.code)}</td>
                    <td>{String(x.name)}</td>
                    <td>{String(x.accountType)}</td>
                    <td>{formatMoneyMinor(Number(x.debitMinor))}</td>
                    <td>{formatMoneyMinor(Number(x.creditMinor))}</td>
                    <td>{formatMoneyMinor(Number(x.netMinor))}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {tb && (
            <p className="kb-muted">
              Check: debits − credits net across TB should reflect activity (
              {formatMoneyMinor(
                sumMinor(
                  (tb as JsonObject[]).map((x) => Number(x.netMinor ?? 0)),
                ),
              )}{" "}
              raw net sum of netMinor — informational)
            </p>
          )}
        </section>
      )}

      {tab === "ar" && (
        <section className="kb-report">
          <button type="button" onClick={() => void loadAr()}>
            Load AR (posted invoices − payments)
          </button>
          <button type="button" onClick={csvAr} disabled={!ar}>
            Export CSV
          </button>
          {ar && (
            <table className="kb-table">
              <thead>
                <tr>
                  <th>Customer</th>
                  <th>Open</th>
                </tr>
              </thead>
              <tbody>
                {(ar as JsonObject[]).map((x, i) => (
                  <tr key={i}>
                    <td>{String(x.displayName)}</td>
                    <td>{formatMoneyMinor(Number(x.openMinor))}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>
      )}

      {tab === "ap" && (
        <section className="kb-report">
          <button type="button" onClick={() => void loadAp()}>
            Load AP (posted bills − payments)
          </button>
          {ap && (
            <table className="kb-table">
              <caption className="kb-sr-only">
                Accounts payable open balances
              </caption>
              <thead>
                <tr>
                  <th scope="col">Vendor</th>
                  <th scope="col">Open</th>
                </tr>
              </thead>
              <tbody>
                {(ap as JsonObject[]).map((x, i) => (
                  <tr key={i}>
                    <td>{String(x.displayName)}</td>
                    <td>{formatMoneyMinor(Number(x.openMinor))}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>
      )}

      {tab === "gl" && (
        <section className="kb-report">
          <div className="kb-row">
            <label>
              Account
              <select
                value={glAccountId || ""}
                onFocus={() => void loadGlAccounts()}
                onChange={(e) => setGlAccountId(Number(e.target.value))}
              >
                {glAccounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.code} — {a.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              From
              <input value={from} onChange={(e) => setFrom(e.target.value)} />
            </label>
            <label>
              To
              <input value={to} onChange={(e) => setTo(e.target.value)} />
            </label>
            <button type="button" onClick={() => void loadGl()}>
              Run
            </button>
          </div>
          {gl && (
            <table className="kb-table">
              <caption className="kb-sr-only">General ledger lines</caption>
              <thead>
                <tr>
                  <th scope="col">Date</th>
                  <th scope="col">Memo</th>
                  <th scope="col">Debit</th>
                  <th scope="col">Credit</th>
                </tr>
              </thead>
              <tbody>
                {(gl as JsonObject[]).map((x, i) => (
                  <tr key={i}>
                    <td>{String(x.entryDate ?? "")}</td>
                    <td>{String(x.memo ?? x.description ?? "")}</td>
                    <td>{formatMoneyMinor(Number(x.debitMinor ?? 0))}</td>
                    <td>{formatMoneyMinor(Number(x.creditMinor ?? 0))}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>
      )}
    </div>
  );
}
