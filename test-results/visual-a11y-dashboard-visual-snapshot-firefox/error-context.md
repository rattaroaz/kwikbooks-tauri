# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: visual-a11y.spec.ts >> dashboard visual snapshot
- Location: tests\e2e\visual-a11y.spec.ts:8:1

# Error details

```
Error: A snapshot doesn't exist at C:\Users\kimri\OneDrive\Desktop\cloned repos\tauri apps\kwikbooks\tests\e2e\visual-a11y.spec.ts-snapshots\dashboard-firefox-win32.png, writing actual.
```

# Page snapshot

```yaml
- generic [ref=e3]:
  - complementary [ref=e4]:
    - generic [ref=e5]: Kwikbooks
    - navigation [ref=e6]:
      - link "Dashboard" [ref=e7] [cursor=pointer]:
        - /url: /
      - link "Getting started" [ref=e8] [cursor=pointer]:
        - /url: /welcome
      - link "Chart of accounts" [ref=e9] [cursor=pointer]:
        - /url: /accounts
      - link "Customers" [ref=e10] [cursor=pointer]:
        - /url: /customers
      - link "Vendors" [ref=e11] [cursor=pointer]:
        - /url: /vendors
      - link "Invoices" [ref=e12] [cursor=pointer]:
        - /url: /invoices
      - link "Bills" [ref=e13] [cursor=pointer]:
        - /url: /bills
      - link "Journal register" [ref=e14] [cursor=pointer]:
        - /url: /register
      - link "Reports" [ref=e15] [cursor=pointer]:
        - /url: /reports
      - link "Settings" [ref=e16] [cursor=pointer]:
        - /url: /settings
  - generic [ref=e18]:
    - heading "Dashboard" [level=1] [ref=e19]
    - paragraph [ref=e20]: Local QuickBooks-style books — SQLite + double-entry posting.
    - generic [ref=e21]:
      - generic [ref=e22]:
        - generic [ref=e23]: Invoices
        - generic [ref=e24]: —
        - link "View" [ref=e25] [cursor=pointer]:
          - /url: /invoices
      - generic [ref=e26]:
        - generic [ref=e27]: Bills
        - generic [ref=e28]: —
        - link "View" [ref=e29] [cursor=pointer]:
          - /url: /bills
      - generic [ref=e30]:
        - generic [ref=e31]: Open AR (customers)
        - generic [ref=e32]: —
        - link "Reports" [ref=e33] [cursor=pointer]:
          - /url: /reports
  - generic [ref=e34]:
    - status [ref=e35]:
      - generic [ref=e36]: can't access property "length", i is null
      - button "×" [ref=e37] [cursor=pointer]
    - status [ref=e38]:
      - generic [ref=e39]: can't access property "length", i is null
      - button "×" [ref=e40] [cursor=pointer]
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
> 13 |   await expect(page).toHaveScreenshot("dashboard.png", { maxDiffPixelRatio: 0.02 });
     |   ^ Error: A snapshot doesn't exist at C:\Users\kimri\OneDrive\Desktop\cloned repos\tauri apps\kwikbooks\tests\e2e\visual-a11y.spec.ts-snapshots\dashboard-firefox-win32.png, writing actual.
  14 | });
  15 | 
  16 | test("invoices list visual snapshot", async ({ page }) => {
  17 |   await page.goto("/invoices");
  18 |   // The page may render asynchronously; wait for any content container
  19 |   await page.waitForTimeout(300);
  20 |   await expect(page).toHaveScreenshot("invoices-list.png", { maxDiffPixelRatio: 0.02 });
  21 | });
  22 | 
```