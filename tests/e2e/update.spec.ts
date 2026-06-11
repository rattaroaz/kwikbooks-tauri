import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

test("settings check for updates shows up to date in E2E mode", async ({
  page,
}) => {
  await page.goto("/settings");
  await expect(page.getByTestId("settings-app-version")).toBeVisible();
  await page.getByTestId("menu-check-updates").click();
  await expect(page.getByTestId("update-dialog")).toBeVisible();
  await expect(page.getByTestId("update-dialog-message")).toContainText(
    /up to date/i,
  );
  await page.getByTestId("update-dialog-close").click();
  await expect(page.getByTestId("update-dialog")).toHaveCount(0);
});
