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
  await page.getByRole("button", { name: "Save draft" }).click();

  await expect(page).toHaveURL(/\/invoices\/\d+$/);
  await expect(page.getByText(/Status: draft/i)).toBeVisible();

  await page.getByRole("button", { name: "Mark sent" }).click();
  await expect(page.getByText(/Status: sent/i)).toBeVisible();

  await page.getByRole("button", { name: "Post to GL" }).click();
  await expect(page.getByRole("button", { name: "Post to GL" })).toHaveCount(0);
});

test("bill flow: create draft, mark open, post GL", async ({ page }) => {
  await page.goto("/bills/new");

  await page.getByLabel("Bill #").fill("B-E2E-1");
  await page.getByLabel("Amount (minor units)").fill("1200");
  await page.getByRole("button", { name: "Save draft" }).click();

  await expect(page).toHaveURL(/\/bills\/\d+$/);
  await expect(page.getByText(/Status: draft/i)).toBeVisible();

  await page.getByRole("button", { name: "Mark open" }).click();
  await expect(page.getByText(/Status: open/i)).toBeVisible();

  await page.getByRole("button", { name: "Post to GL" }).click();
  await expect(page.getByRole("button", { name: "Post to GL" })).toHaveCount(0);
});

test("settings backup and restore actions complete", async ({ page }) => {
  await page.goto("/settings");

  await page.getByLabel("Company name").fill("E2E Co");
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByText(/Company saved/i)).toBeVisible();

  await page.getByRole("button", { name: "Backup to file…" }).click();
  await expect(page.getByText(/Backup saved/i)).toBeVisible();

  await page.evaluate(() => {
    window.confirm = () => true;
  });
  await page.getByRole("button", { name: "Restore from backup…" }).click();
  await expect(page.getByText(/Database restored/i)).toBeVisible();
});
