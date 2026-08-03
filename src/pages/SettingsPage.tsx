import { save, open } from "@tauri-apps/plugin-dialog";
import { FormEvent, useEffect, useState } from "react";
import { healthPing, type HealthResponse } from "../api/db";
import * as api from "../api/tauri";
import type { JsonObject } from "../api/tauri";
import { useToast } from "../context/ToastContext";
import { logContext } from "../lib/logContext";
import { createScopedLogger } from "../lib/logger";
import { APP_VERSION } from "../lib/constants";
import { useLogViewer } from "../context/LogViewerContext";
import { checkForUpdatesAndApply } from "../services/updateService";

const SQLITE_FILTER = [{ name: "SQLite", extensions: ["sqlite", "db"] }];
const QB_IMPORT_FILTER = [
  { name: "QuickBooks export", extensions: ["iif", "csv", "txt"] },
];
const log = createScopedLogger("Settings");
const PAGE = "SettingsPage";

export function SettingsPage() {
  const { push, pushApiError } = useToast();
  const { openLogs } = useLogViewer();
  const [name, setName] = useState("");
  const [legalName, setLegalName] = useState("");
  const [fiscalMonth, setFiscalMonth] = useState("1");
  const [currency, setCurrency] = useState("USD");
  const [nextInv, setNextInv] = useState("1000");
  const [nextBill, setNextBill] = useState("1000");
  const [addressLine1, setAddressLine1] = useState("");
  const [addressLine2, setAddressLine2] = useState("");
  const [city, setCity] = useState("");
  const [region, setRegion] = useState("");
  const [postalCode, setPostalCode] = useState("");
  const [nextCheck, setNextCheck] = useState("1000");
  const [defaultCheckStyle, setDefaultCheckStyle] = useState("voucher_top");
  const [health, setHealth] = useState<HealthResponse | null>(null);

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
        setAddressLine1(String(c.addressLine1 ?? ""));
        setAddressLine2(String(c.addressLine2 ?? ""));
        setCity(String(c.city ?? ""));
        setRegion(String(c.region ?? ""));
        setPostalCode(String(c.postalCode ?? ""));
        setNextCheck(String(c.nextCheckNumber ?? "1000"));
        setDefaultCheckStyle(String(c.defaultCheckStyle ?? "voucher_top"));
      } catch (e) {
        pushApiError(e, logContext(PAGE, "load"));
      }
    })();
  }, [pushApiError]);

  useEffect(() => {
    void (async () => {
      try {
        setHealth(await healthPing());
      } catch {
        /* optional diagnostics — company load already surfaces hard failures */
      }
    })();
  }, []);

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
      pushApiError(e, logContext(PAGE, "backup"));
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
      pushApiError(e, logContext(PAGE, "importQuickbooks"));
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
      pushApiError(e, logContext(PAGE, "restore"));
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
        addressLine1: addressLine1.trim() || undefined,
        addressLine2: addressLine2.trim() || undefined,
        city: city.trim() || undefined,
        region: region.trim() || undefined,
        postalCode: postalCode.trim() || undefined,
        nextCheckNumber: Number(nextCheck),
        defaultCheckStyle,
      });
      void log.info("company profile saved");
      push("success", "Company saved");
    } catch (err) {
      pushApiError(err, logContext(PAGE, "save"));
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
        <label>
          Address line 1
          <input
            value={addressLine1}
            onChange={(e) => setAddressLine1(e.target.value)}
          />
        </label>
        <label>
          Address line 2
          <input
            value={addressLine2}
            onChange={(e) => setAddressLine2(e.target.value)}
          />
        </label>
        <label>
          City
          <input value={city} onChange={(e) => setCity(e.target.value)} />
        </label>
        <label>
          State / region
          <input value={region} onChange={(e) => setRegion(e.target.value)} />
        </label>
        <label>
          Postal code
          <input
            value={postalCode}
            onChange={(e) => setPostalCode(e.target.value)}
          />
        </label>
        <label>
          Next check #
          <input
            value={nextCheck}
            onChange={(e) => setNextCheck(e.target.value)}
            data-testid="settings-next-check"
          />
        </label>
        <label>
          Default check stock layout
          <select
            value={defaultCheckStyle}
            onChange={(e) => setDefaultCheckStyle(e.target.value)}
            data-testid="settings-check-style"
          >
            <option value="voucher_top">Voucher — check on top</option>
            <option value="voucher_middle">Voucher — check in middle</option>
            <option value="voucher_bottom">Voucher — check on bottom</option>
            <option value="generic">Generic (alignment guides)</option>
          </select>
        </label>
        <button type="submit" data-testid="settings-save">
          Save
        </button>
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
          Offline-only. Logs, panics, and exception captures stay on this
          computer. From the log viewer you can copy lines or export a redacted
          support bundle to share.
        </p>
        {health ? (
          <dl className="kb-muted" data-testid="settings-health">
            <div>
              Host {health.appVersion} · schema v{health.migrationVersion} ·
              SQLite {health.sqliteOk ? "ok" : "fail"}
            </div>
            <div>
              Log level {health.logLevel} · slow IPC {health.slowIpcMs}ms
            </div>
          </dl>
        ) : null}
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
            data-testid="settings-backup"
            onClick={onBackup}
          >
            Backup to file…
          </button>
          <button
            type="button"
            className="kb-button-secondary"
            data-testid="settings-restore"
            onClick={onRestore}
          >
            Restore from backup…
          </button>
        </div>
      </section>
    </div>
  );
}
