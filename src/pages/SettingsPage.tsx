import { save, open } from "@tauri-apps/plugin-dialog";
import { FormEvent, useEffect, useState } from "react";
import * as api from "../api/tauri";
import type { JsonObject } from "../api/tauri";
import { useToast } from "../context/ToastContext";
import { createScopedLogger } from "../lib/logger";
import { APP_VERSION } from "../lib/constants";
import { useLogViewer } from "../context/LogViewerContext";
import { checkForUpdatesAndApply } from "../services/updateService";

const SQLITE_FILTER = [{ name: "SQLite", extensions: ["sqlite", "db"] }];
const QB_IMPORT_FILTER = [
  { name: "QuickBooks export", extensions: ["iif", "csv", "txt"] },
];
const log = createScopedLogger("Settings");

export function SettingsPage() {
  const { push, pushApiError } = useToast();
  const { openLogs } = useLogViewer();
  const [name, setName] = useState("");
  const [legalName, setLegalName] = useState("");
  const [fiscalMonth, setFiscalMonth] = useState("1");
  const [currency, setCurrency] = useState("USD");
  const [nextInv, setNextInv] = useState("1000");
  const [nextBill, setNextBill] = useState("1000");

  useEffect(() => {
    void (async () => {
      try {
        const c = (await api.companyGet()) as JsonObject;
        setName(String(c.name ?? ""));
        setLegalName(String(c.legalName ?? ""));
        setFiscalMonth(String(c.fiscalYearStartMonth ?? "1"));
        setCurrency(String(c.baseCurrencyCode ?? "USD"));
        setNextInv(String(c.nextInvoiceNumber ?? "1000"));
        setNextBill(String(c.nextBillNumber ?? "1000"));
      } catch (e) {
        pushApiError(e, "SettingsPage");
      }
    })();
  }, [pushApiError]);

  async function onBackup() {
    try {
      const dest = await save({
        filters: SQLITE_FILTER,
        defaultPath: `kwikbooks-backup-${new Date().toISOString().slice(0, 10)}.sqlite`,
      });
      if (dest === null) {
        return;
      }
      await api.dbBackupVacuum(dest);
      void log.info("backup completed (vacuum into)");
      push("success", "Backup saved");
    } catch (e) {
      pushApiError(e, "SettingsPage");
    }
  }

  async function onImportQuickbooks() {
    try {
      const picked = await open({
        title: "Import QuickBooks export",
        filters: QB_IMPORT_FILTER,
        multiple: false,
      });
      const path =
        picked === null ? null : Array.isArray(picked) ? picked[0] : picked;
      if (path === null || path === undefined) {
        return;
      }
      const s = await api.importQuickbooksFile(path);
      const parts = [
        `${s.formatDetected.toUpperCase()}`,
        s.accountsCreated ? `${s.accountsCreated} account(s)` : null,
        s.customersCreated ? `${s.customersCreated} customer(s)` : null,
        s.vendorsCreated ? `${s.vendorsCreated} vendor(s)` : null,
        s.itemsCreated ? `${s.itemsCreated} item(s)` : null,
        s.rowsSkipped ? `${s.rowsSkipped} row(s) skipped` : null,
      ].filter(Boolean);
      void log.info("quickbooks import completed");
      push("success", `Imported: ${parts.join(" · ")}`);
      if (s.warnings.length > 0) {
        push("info", s.warnings.slice(0, 5).join(" "));
      }
    } catch (e) {
      pushApiError(e, "SettingsPage");
    }
  }

  async function onRestore() {
    try {
      const picked = await open({
        title: "Restore from backup",
        filters: SQLITE_FILTER,
        multiple: false,
      });
      const path =
        picked === null ? null : Array.isArray(picked) ? picked[0] : picked;
      if (path === null || path === undefined) {
        return;
      }
      const meta = await api.dbRestoreValidate(path);
      const ok = window.confirm(
        `Replace the current company file with this backup (migration head v${String(meta.migrationVersion)})? This cannot be undone without another backup.`,
      );
      if (!ok) {
        return;
      }
      await api.dbRestoreApply(path);
      void log.warn("database restored from backup file");
      push(
        "success",
        "Database restored. Reload the app if numbers look stale.",
      );
    } catch (e) {
      pushApiError(e, "SettingsPage");
    }
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    try {
      await api.companyUpdate({
        name: name.trim(),
        legalName: legalName.trim() || undefined,
        fiscalYearStartMonth: Number(fiscalMonth),
        baseCurrencyCode: currency.trim().toUpperCase(),
        nextInvoiceNumber: Number(nextInv),
        nextBillNumber: Number(nextBill),
      });
      void log.info("company profile saved");
      push("success", "Company saved");
    } catch (err) {
      pushApiError(err, "SettingsPage");
    }
  }

  return (
    <div className="kb-page">
      <h1>Settings</h1>
      <p className="kb-muted">
        Single-company file · numbering hints for new documents (automation can
        be wired later).
      </p>
      <form className="kb-form kb-form-stack" onSubmit={onSubmit}>
        <label>
          Company name
          <input value={name} onChange={(e) => setName(e.target.value)} />
        </label>
        <label>
          Legal name
          <input
            value={legalName}
            onChange={(e) => setLegalName(e.target.value)}
          />
        </label>
        <label>
          Fiscal year starts (month 1–12)
          <input
            type="number"
            min={1}
            max={12}
            value={fiscalMonth}
            onChange={(e) => setFiscalMonth(e.target.value)}
          />
        </label>
        <label>
          Base currency (ISO code)
          <input
            value={currency}
            onChange={(e) => setCurrency(e.target.value)}
          />
        </label>
        <label>
          Next invoice # (suggested)
          <input value={nextInv} onChange={(e) => setNextInv(e.target.value)} />
        </label>
        <label>
          Next bill # (suggested)
          <input
            value={nextBill}
            onChange={(e) => setNextBill(e.target.value)}
          />
        </label>
        <button type="submit">Save</button>
      </form>

      <section className="kb-settings-extra">
        <h2>Import from QuickBooks</h2>
        <p className="kb-muted">
          Bring over lists from QuickBooks Desktop exports: tab-separated{" "}
          <code>.iif</code> (chart of accounts, customers, vendors, items) or{" "}
          <code>.csv</code> / tab-delimited text from lists and reports.
          Existing rows with the same account code or name are skipped.
        </p>
        <div className="kb-actions">
          <button
            type="button"
            className="kb-button-secondary"
            onClick={() => void onImportQuickbooks()}
          >
            Choose export file…
          </button>
        </div>
      </section>

      <section className="kb-settings-extra">
        <h2>Application updates</h2>
        <p className="kb-muted">
          Updates are checked manually only (not on startup). When a newer
          signed release is published on GitHub, download and install it here.
        </p>
        <p className="kb-muted" data-testid="settings-app-version">
          Installed version: <strong>{APP_VERSION}</strong>
        </p>
        <div className="kb-actions">
          <button
            type="button"
            className="kb-button-secondary"
            data-testid="menu-check-updates"
            onClick={() => void checkForUpdatesAndApply()}
          >
            Check for updates
          </button>
        </div>
      </section>

      <section className="kb-settings-extra">
        <h2>Diagnostics</h2>
        <p className="kb-muted">
          View recent application and webview log entries written by Kwikbooks on
          this computer.
        </p>
        <div className="kb-actions">
          <button
            type="button"
            className="kb-button-secondary"
            data-testid="settings-view-logs"
            onClick={() => void openLogs()}
          >
            View logs
          </button>
        </div>
      </section>

      <section className="kb-settings-extra">
        <h2>Backup &amp; restore</h2>
        <p className="kb-muted">
          Saves a consistent SQLite snapshot (<code>VACUUM INTO</code>). Restore
          replaces your live books file — keep copies elsewhere (USB, cloud) as
          you see fit.
        </p>
        <div className="kb-actions">
          <button
            type="button"
            className="kb-button-secondary"
            onClick={onBackup}
          >
            Backup to file…
          </button>
          <button
            type="button"
            className="kb-button-secondary"
            onClick={onRestore}
          >
            Restore from backup…
          </button>
        </div>
      </section>
    </div>
  );
}
