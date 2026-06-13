import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

test("receive payment flow posts successfully", async ({ page }) => {
  await page.goto("/payments/receive");
  await expect(
    page.getByRole("heading", { name: "Receive payment" }),
  ).toBeVisible();
  await page.getByLabel("Amount (minor units)").fill("2500");
  await page.getByTestId("receive-payment-submit").click();
  await expect(page.getByText(/Customer payment recorded/i)).toBeVisible();
});

test("pay vendor flow posts successfully", async ({ page }) => {
  await page.goto("/payments/pay");
  await expect(page.getByRole("heading", { name: "Pay vendor" })).toBeVisible();
  await page.getByLabel("Amount (minor units)").fill("1200");
  await page.getByTestId("pay-bill-submit").click();
  await expect(page.getByText(/Vendor payment recorded/i)).toBeVisible();
});

test("reports profit and loss loads", async ({ page }) => {
  await page.goto("/reports");
  await page.getByTestId("reports-pl-run").click();
  await expect(page.getByText(/Profit & loss loaded/i)).toBeVisible();
});

test("accounts page lists and creates account", async ({ page }) => {
  await page.goto("/accounts");
  await expect(
    page.getByRole("heading", { name: "Chart of accounts" }),
  ).toBeVisible();
  await page.getByLabel("Code").fill("6100");
  await page.getByLabel("Name").fill("Rent");
  await page.getByTestId("accounts-submit").click();
  await expect(page.getByText(/Account created/i)).toBeVisible();
});
