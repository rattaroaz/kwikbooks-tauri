import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

test("invoice new surfaces backend create failure as toast", async ({
  page,
}) => {
  await page.goto("/invoices/new");
  await page.evaluate(() => {
    const controls = (
      window as Window & {
        __E2E_MOCK__?: {
          setCommandError: (cmd: string, msg: string | null) => void;
        };
      }
    ).__E2E_MOCK__;
    controls?.setCommandError("invoice_create", "mock invoice create failure");
  });
  await page.getByLabel("Invoice #").fill("INV-ERR-BE");
  await page.getByRole("button", { name: "Save draft" }).click();
  await expect(page.getByText("mock invoice create failure")).toBeVisible();
});

test("invoice new blocks invalid quantity", async ({ page }) => {
  await page.goto("/invoices/new");
  await page.getByLabel("Invoice #").fill("INV-ERR-2");
  await page.getByLabel("Qty").fill("0");
  await page.getByRole("button", { name: "Save draft" }).click();

  await expect(page.getByText("Invalid quantity")).toBeVisible();
  await expect(page).toHaveURL(/\/invoices\/new$/);
});

test("restore cancelled by confirm does not run restore", async ({ page }) => {
  await page.goto("/settings");
  await page.evaluate(() => {
    window.confirm = () => false;
  });
  await page.getByRole("button", { name: "Restore from backup…" }).click();

  await expect(page.getByText(/Database restored/i)).toHaveCount(0);
});

test("restore validate failure is surfaced as toast", async ({ page }) => {
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
      "db_restore_validate",
      "mock restore validation failure",
    );
  });
  await page.getByRole("button", { name: "Restore from backup…" }).click();

  await expect(page.getByText("mock restore validation failure")).toBeVisible();
});
