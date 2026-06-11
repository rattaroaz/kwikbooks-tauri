import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

async function expectNoCriticalA11yViolations(
  page: import("@playwright/test").Page,
) {
  const results = await new AxeBuilder({ page })
    .disableRules(["color-contrast"])
    .analyze();
  expect(results.violations.filter((v) => v.impact === "critical")).toEqual([]);
}

test("dashboard has no critical a11y violations", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  await expectNoCriticalA11yViolations(page);
});

test("settings form has no critical a11y violations", async ({ page }) => {
  await page.goto("/settings");
  await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  await expectNoCriticalA11yViolations(page);
});

test("invoice form has no critical a11y violations", async ({ page }) => {
  await page.goto("/invoices/new");
  await expect(
    page.getByRole("heading", { name: /New invoice/i }),
  ).toBeVisible();
  await expectNoCriticalA11yViolations(page);
});
