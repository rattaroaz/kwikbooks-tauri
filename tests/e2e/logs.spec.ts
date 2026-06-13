import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock";

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

test("settings view logs opens right panel with level filters", async ({
  page,
}) => {
  await page.goto("/settings");
  await page.getByTestId("settings-view-logs").click();
  const panel = page.getByTestId("logs-panel");
  await expect(panel).toBeVisible();
  await expect(page.getByTestId("logs-body")).toContainText(
    "Kwikbooks starting",
  );

  const mainBox = await page.locator(".kb-main").boundingBox();
  const panelBox = await panel.boundingBox();
  expect(mainBox).not.toBeNull();
  expect(panelBox).not.toBeNull();
  if (mainBox && panelBox) {
    expect(panelBox.x).toBeGreaterThan(mainBox.x + mainBox.width - 4);
  }

  await page.getByTestId("logs-level-select").selectOption("info");
  await expect(page.getByTestId("logs-body")).toContainText(
    "Kwikbooks starting",
  );
  await expect(page.getByTestId("logs-body")).not.toContainText(
    "check started",
  );

  await page.getByTestId("logs-search").fill("e2e-mock");
  await expect(page.getByTestId("logs-body")).toContainText("seq=99");
  await expect(page.getByTestId("logs-body")).not.toContainText(
    "Kwikbooks starting",
  );

  await page.getByTestId("logs-close").click();
  await expect(panel).toHaveCount(0);
});
