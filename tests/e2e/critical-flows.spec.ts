import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

test("invoice flow: create draft, mark sent, post GL", async ({ page }) => {
  await page.goto("/invoices/new");

  await page.getByLabel("Invoice #").fill("INV-E2E-1");
  await page.getByLabel("Tax (minor units)").fill("100");
  await page.getByLabel("Unit price (minor units)").fill("5000");
  await page.getByTestId("invoice-new-save-draft").click();

  await expect(page).toHaveURL(/\/invoices\/\d+$/);
  await expect(page.getByText(/Status: draft/i)).toBeVisible();

  await page.getByTestId("invoice-mark-sent").click();
  await expect(page.getByText(/Status: sent/i)).toBeVisible();

  await page.getByTestId("invoice-post-gl").click();
  await expect(page.getByTestId("invoice-post-gl")).toHaveCount(0);
});

test("bill flow: create draft, mark open, post GL", async ({ page }) => {
  await page.goto("/bills/new");

  await page.getByLabel("Bill #").fill("B-E2E-1");
  await page.getByLabel("Amount (minor units)").fill("1200");
  await page.getByTestId("bill-new-save-draft").click();

  await expect(page).toHaveURL(/\/bills\/\d+$/);
  await expect(page.getByText(/Status: draft/i)).toBeVisible();

  await page.getByTestId("bill-mark-open").click();
  await expect(page.getByText(/Status: open/i)).toBeVisible();

  await page.getByTestId("bill-post-gl").click();
  await expect(page.getByTestId("bill-post-gl")).toHaveCount(0);
});

test("settings backup and restore actions complete", async ({ page }) => {
  await page.goto("/settings");

  await page.getByLabel("Company name").fill("E2E Co");
  await page.getByTestId("settings-save").click();
  await expect(page.getByText(/Company saved/i)).toBeVisible();

  await page.getByTestId("settings-backup").click();
  await expect(page.getByText(/Backup saved/i)).toBeVisible();

  await page.evaluate(() => {
    window.confirm = () => true;
  });
  // Restore applies then reloads; toast is cleared by the reload.
  await Promise.all([
    page.waitForEvent("load"),
    page.getByTestId("settings-restore").click(),
  ]);
  await expect(page.getByTestId("settings-restore")).toBeVisible();
});
