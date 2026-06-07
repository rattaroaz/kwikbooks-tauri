import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

test("dashboard visual snapshot", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();

  // Visual regression (update baseline on first run with --update-snapshots)
  await expect(page).toHaveScreenshot("dashboard.png", {
    maxDiffPixelRatio: 0.02,
  });
});

test("invoices list visual snapshot", async ({ page }) => {
  await page.goto("/invoices");
  // The page may render asynchronously; wait for any content container
  await page.waitForTimeout(300);
  await expect(page).toHaveScreenshot("invoices-list.png", {
    maxDiffPixelRatio: 0.02,
  });
});
