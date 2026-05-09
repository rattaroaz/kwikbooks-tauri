# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: visual-a11y.spec.ts >> invoices list visual snapshot
- Location: tests\e2e\visual-a11y.spec.ts:16:1

# Error details

```
Error: A snapshot doesn't exist at C:\Users\kimri\OneDrive\Desktop\cloned repos\tauri apps\kwikbooks\tests\e2e\visual-a11y.spec.ts-snapshots\invoices-list-firefox-win32.png, writing actual.
```

# Test source

```ts
  1  | import { expect, test } from "@playwright/test";
  2  | import { installTauriMock } from "./tauriMock";
  3  | 
  4  | test.beforeEach(async ({ page }) => {
  5  |   await installTauriMock(page);
  6  | });
  7  | 
  8  | test("dashboard visual snapshot", async ({ page }) => {
  9  |   await page.goto("/");
  10 |   await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  11 | 
  12 |   // Visual regression (update baseline on first run with --update-snapshots)
  13 |   await expect(page).toHaveScreenshot("dashboard.png", { maxDiffPixelRatio: 0.02 });
  14 | });
  15 | 
  16 | test("invoices list visual snapshot", async ({ page }) => {
  17 |   await page.goto("/invoices");
  18 |   // The page may render asynchronously; wait for any content container
  19 |   await page.waitForTimeout(300);
> 20 |   await expect(page).toHaveScreenshot("invoices-list.png", { maxDiffPixelRatio: 0.02 });
     |   ^ Error: A snapshot doesn't exist at C:\Users\kimri\OneDrive\Desktop\cloned repos\tauri apps\kwikbooks\tests\e2e\visual-a11y.spec.ts-snapshots\invoices-list-firefox-win32.png, writing actual.
  21 | });
  22 | 
```