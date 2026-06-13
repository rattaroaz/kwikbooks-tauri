import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

test("dashboard survives transient IPC failure after reload", async ({
  page,
}) => {
  await page.goto("/");
  await page.evaluate(() => {
    const controls = (
      window as Window & {
        __E2E_MOCK__?: {
          setCommandError: (cmd: string, msg: string | null) => void;
        };
      }
    ).__E2E_MOCK__;
    controls?.setCommandError(
      "list_invoices",
      "mock transient list_invoices failure",
    );
  });
  await page.reload();
  await expect(page.locator(".kb-toast-error").first()).toBeVisible();

  await page.evaluate(() => {
    const controls = (
      window as Window & {
        __E2E_MOCK__?: {
          setCommandError: (cmd: string, msg: string | null) => void;
        };
      }
    ).__E2E_MOCK__;
    controls?.setCommandError("list_invoices", null);
  });
  await page.reload();
  await expect(page.locator(".kb-toast-error")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
});

test("invoice status action retries after IPC error clears", async ({
  page,
}) => {
  await page.goto("/invoices/new");
  await page.getByLabel("Invoice #").fill("INV-CHAOS-1");
  await page.getByTestId("invoice-new-save-draft").click();
  await expect(page).toHaveURL(/\/invoices\/\d+$/);

  await page.evaluate(() => {
    const controls = (
      window as Window & {
        __E2E_MOCK__?: {
          setCommandError: (cmd: string, msg: string | null) => void;
        };
      }
    ).__E2E_MOCK__;
    controls?.setCommandError(
      "invoice_set_status",
      "mock transient status failure",
    );
  });
  await page.getByTestId("invoice-mark-sent").click();
  await expect(page.getByText("mock transient status failure")).toBeVisible();
  await expect(page.getByText(/Status: draft/i)).toBeVisible();

  await page.evaluate(() => {
    const controls = (
      window as Window & {
        __E2E_MOCK__?: {
          setCommandError: (cmd: string, msg: string | null) => void;
        };
      }
    ).__E2E_MOCK__;
    controls?.setCommandError("invoice_set_status", null);
  });
  await page.getByTestId("invoice-mark-sent").click();
  await expect(page.getByText(/Status: sent/i)).toBeVisible();
});

test("backup action recovers from transient IPC failure", async ({ page }) => {
  await page.goto("/settings");
  await page.evaluate(() => {
    const controls = (
      window as Window & {
        __E2E_MOCK__?: {
          setCommandError: (cmd: string, msg: string | null) => void;
        };
      }
    ).__E2E_MOCK__;
    controls?.setCommandError(
      "db_backup_vacuum",
      "mock transient backup failure",
    );
  });
  await page.getByTestId("settings-backup").click();
  await expect(page.getByText("mock transient backup failure")).toBeVisible();

  await page.evaluate(() => {
    const controls = (
      window as Window & {
        __E2E_MOCK__?: {
          setCommandError: (cmd: string, msg: string | null) => void;
        };
      }
    ).__E2E_MOCK__;
    controls?.setCommandError("db_backup_vacuum", null);
  });
  await page.getByTestId("settings-backup").click();
  await expect(page.getByText("Backup saved")).toBeVisible();
});
