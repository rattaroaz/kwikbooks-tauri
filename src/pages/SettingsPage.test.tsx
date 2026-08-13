// @vitest-environment happy-dom
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { save, open } from "@tauri-apps/plugin-dialog";
import { SettingsPage } from "./SettingsPage";
import { renderWithApp } from "../test/renderWithApp";
import * as api from "../api/tauri";

const openLogsMock = vi.fn();

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
  open: vi.fn(),
}));

vi.mock("../context/LogViewerContext", () => ({
  useLogViewer: () => ({
    open: false,
    openLogs: openLogsMock,
    closeLogs: vi.fn(),
  }),
}));

vi.mock("../services/updateService", () => ({
  checkForUpdatesAndApply: vi.fn(),
}));

vi.mock("../api/tauri", () => ({
  companyGet: vi.fn(),
  companyUpdate: vi.fn(),
  dbBackupVacuum: vi.fn(),
  dbRestoreValidate: vi.fn(),
  dbRestoreApply: vi.fn(),
  importQuickbooksFile: vi.fn(),
}));

vi.mock("../api/db", () => ({
  healthPing: vi.fn(() =>
    Promise.resolve({
      ok: true,
      sqliteOk: true,
      migrationVersion: 6,
      appVersion: "1.1.0",
      logLevel: "Info",
      slowIpcMs: 1500,
    }),
  ),
}));

describe("SettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    openLogsMock.mockReset();
    vi.mocked(api.companyGet).mockResolvedValue({
      name: "Acme Books",
      legalName: "Acme LLC",
      fiscalYearStartMonth: 4,
      baseCurrencyCode: "USD",
      nextInvoiceNumber: 2000,
      nextBillNumber: 3000,
    });
    vi.mocked(api.companyUpdate).mockResolvedValue(undefined);
    vi.mocked(save).mockResolvedValue("/tmp/kwikbooks-backup.sqlite");
    vi.mocked(api.dbBackupVacuum).mockResolvedValue(undefined);
  });

  it("loads company profile into the form", async () => {
    renderWithApp(<SettingsPage />);

    await waitFor(() => {
      expect(screen.getByLabelText(/^Company name/i)).toHaveValue("Acme Books");
    });
    expect(screen.getByLabelText(/^Legal name/i)).toHaveValue("Acme LLC");
    expect(screen.getByLabelText(/^Fiscal year/i)).toHaveValue(4);
    expect(screen.getByLabelText(/^Base currency/i)).toHaveValue("USD");
    expect(screen.getByLabelText(/^Next invoice/i)).toHaveValue("2000");
    expect(screen.getByLabelText(/^Next bill/i)).toHaveValue("3000");
    expect(api.companyGet).toHaveBeenCalled();
  });

  it("saves company profile on submit", async () => {
    renderWithApp(<SettingsPage />);
    await waitFor(() =>
      expect(screen.getByLabelText(/^Company name/i)).toHaveValue("Acme Books"),
    );

    fireEvent.change(screen.getByLabelText(/^Company name/i), {
      target: { value: "New Co" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(api.companyUpdate).toHaveBeenCalledWith(
        expect.objectContaining({
          name: "New Co",
          legalName: "Acme LLC",
          fiscalYearStartMonth: 4,
          baseCurrencyCode: "USD",
          nextInvoiceNumber: 2000,
          nextBillNumber: 3000,
        }),
      );
    });
  });

  it("backs up database when a destination is chosen", async () => {
    renderWithApp(<SettingsPage />);
    await screen.findByRole("button", { name: "Save" });

    fireEvent.click(screen.getByRole("button", { name: /Backup to file/i }));

    await waitFor(() => {
      expect(save).toHaveBeenCalled();
      expect(api.dbBackupVacuum).toHaveBeenCalledWith(
        "/tmp/kwikbooks-backup.sqlite",
      );
    });
  });

  it("skips backup when the save dialog is cancelled", async () => {
    vi.mocked(save).mockResolvedValue(null);

    renderWithApp(<SettingsPage />);
    await screen.findByRole("button", { name: "Save" });

    fireEvent.click(screen.getByRole("button", { name: /Backup to file/i }));

    await waitFor(() => expect(save).toHaveBeenCalled());
    expect(api.dbBackupVacuum).not.toHaveBeenCalled();
  });

  it("opens the log viewer from diagnostics", async () => {
    renderWithApp(<SettingsPage />);

    fireEvent.click(await screen.findByTestId("settings-view-logs"));

    expect(openLogsMock).toHaveBeenCalled();
  });

  it("does not save an invalid fiscal month", async () => {
    renderWithApp(<SettingsPage />);
    await waitFor(() =>
      expect(screen.getByLabelText(/^Company name/i)).toHaveValue("Acme Books"),
    );

    fireEvent.change(screen.getByLabelText(/^Fiscal year/i), {
      target: { value: "13" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(api.companyUpdate).not.toHaveBeenCalled();
  });

  it("imports QuickBooks export when a file is selected", async () => {
    vi.mocked(open).mockResolvedValue("/exports/accounts.iif");
    vi.mocked(api.importQuickbooksFile).mockResolvedValue({
      formatDetected: "iif",
      accountsCreated: 3,
      customersCreated: 1,
      vendorsCreated: 0,
      itemsCreated: 0,
      rowsSkipped: 0,
      warnings: [],
    });

    renderWithApp(<SettingsPage />);
    await screen.findByRole("button", { name: "Save" });

    fireEvent.click(
      screen.getByRole("button", { name: /Choose export file/i }),
    );

    await waitFor(() => {
      expect(open).toHaveBeenCalled();
      expect(api.importQuickbooksFile).toHaveBeenCalledWith(
        "/exports/accounts.iif",
      );
    });
  });
});
