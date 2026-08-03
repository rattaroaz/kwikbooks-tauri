import type { Page } from "@playwright/test";

export async function installTauriMock(page: Page): Promise<void> {
  await page.addInitScript(() => {
    type Customer = { id: number; displayName: string };
    type Vendor = { id: number; displayName: string };
    type Account = {
      id: number;
      code: string;
      name: string;
      accountType: string;
      isBankCash?: boolean;
    };
    type Invoice = {
      id: number;
      customerId: number;
      number: string;
      issueDate: string;
      status: "draft" | "sent" | "paid" | "void";
      journalId: number | null;
      subtotalMinor: number;
      taxMinor: number;
      totalMinor: number;
      customerName: string;
    };
    type Bill = {
      id: number;
      vendorId: number | null;
      payeeName: string | null;
      number: string;
      issueDate: string;
      status: "draft" | "open" | "paid" | "void";
      journalId: number | null;
      totalMinor: number;
      vendorName: string | null;
    };
    const COMMAND_ERRORS_KEY = "kwikbooks-e2e-command-errors";

    function readCommandErrors(): Record<string, string> {
      try {
        const raw = sessionStorage.getItem(COMMAND_ERRORS_KEY);
        if (!raw) {
          return {};
        }
        const parsed: unknown = JSON.parse(raw);
        return typeof parsed === "object" && parsed !== null
          ? (parsed as Record<string, string>)
          : {};
      } catch {
        return {};
      }
    }

    function writeCommandErrors(errors: Record<string, string>): void {
      sessionStorage.setItem(COMMAND_ERRORS_KEY, JSON.stringify(errors));
    }

    /** Mirrors Rust / src/lib/money.ts lineTotalMinor (6 dp scale). */
    function lineTotalMinor(qty: number, unitMinor: number): number {
      const SCALE = 1_000_000;
      if (!Number.isFinite(qty) || !Number.isFinite(unitMinor)) {
        return 0;
      }
      const qtyScaled = Math.round(qty * SCALE);
      const product = qtyScaled * Math.trunc(unitMinor);
      const half = SCALE / 2;
      const rounded =
        product >= 0
          ? Math.trunc((product + half) / SCALE)
          : Math.trunc((product - half) / SCALE);
      return rounded <= 0 ? 0 : rounded;
    }

    const state = {
      nextInvoiceId: 1,
      nextBillId: 1,
      nextJournalId: 1,
      nextPaymentId: 1,
      company: {
        name: "Mock Company",
        legalName: "Mock Company LLC",
        fiscalYearStartMonth: 1,
        baseCurrencyCode: "USD",
        nextInvoiceNumber: 1000,
        nextBillNumber: 2000,
        addressLine1: "123 Main St",
        addressLine2: "",
        city: "Springfield",
        region: "IL",
        postalCode: "62701",
        nextCheckNumber: 1000,
        defaultCheckStyle: "voucher_top",
      },
      customers: [{ id: 1, displayName: "Acme Corp" }] as Customer[],
      vendors: [{ id: 1, displayName: "Office Mart" }] as Vendor[],
      accounts: [
        {
          id: 1,
          code: "5000",
          name: "Expenses",
          accountType: "expense",
          isBankCash: false,
        },
        {
          id: 2,
          code: "1000",
          name: "Checking",
          accountType: "asset",
          isBankCash: true,
        },
      ] as Account[],
      invoices: [] as Invoice[],
      invoiceLines: new Map<number, Array<Record<string, unknown>>>(),
      bills: [] as Bill[],
      billLines: new Map<number, Array<Record<string, unknown>>>(),
      backupPath: "C:/tmp/kwikbooks-backup.sqlite",
      commandErrors: readCommandErrors(),
    };

    const controls = {
      setCommandError(command: string, message: string | null) {
        if (message === null) {
          delete state.commandErrors[command];
        } else {
          state.commandErrors[command] = message;
        }
        writeCommandErrors(state.commandErrors);
      },
    };

    function invoke(command: string, args: Record<string, unknown> = {}) {
      if (state.commandErrors[command]) {
        return Promise.reject(new Error(state.commandErrors[command]));
      }
      switch (command) {
        case "db_init":
          return Promise.resolve({
            dbPath: "mock.sqlite",
            migrationVersion: 6,
          });
        case "health_ping":
          return Promise.resolve({
            ok: true,
            sqliteOk: true,
            migrationVersion: 6,
            appVersion: "0.0.0-e2e",
            logLevel: "Info",
            slowIpcMs: 1500,
          });
        case "company_get":
          return Promise.resolve({ ...state.company });
        case "company_update":
          state.company = { ...state.company, ...(args.input as object) };
          return Promise.resolve();
        case "list_customers":
          return Promise.resolve(state.customers);
        case "list_vendors":
          return Promise.resolve(state.vendors);
        case "account_list":
          return Promise.resolve(state.accounts);
        case "invoice_create": {
          const input = args.input as Record<string, unknown>;
          const id = state.nextInvoiceId++;
          const lines = (input.lines as Array<Record<string, unknown>>) ?? [];
          const subtotal = lines.reduce(
            (acc, l) =>
              acc +
              lineTotalMinor(
                Number(l.quantity ?? 0),
                Number(l.unitPriceMinor ?? 0),
              ),
            0,
          );
          const tax = Number(input.taxMinor ?? 0);
          const customerId = Number(input.customerId);
          const customerName =
            state.customers.find((c) => c.id === customerId)?.displayName ??
            "Unknown";
          state.invoices.push({
            id,
            customerId,
            number: String(input.number ?? ""),
            issueDate: String(input.issueDate ?? ""),
            status: "draft",
            journalId: null,
            subtotalMinor: subtotal,
            taxMinor: tax,
            totalMinor: subtotal + tax,
            customerName,
          });
          state.invoiceLines.set(
            id,
            lines.map((l, idx) => ({
              lineNumber: idx + 1,
              description: String(l.description ?? ""),
              quantity: Number(l.quantity ?? 0),
              lineTotalMinor: lineTotalMinor(
                Number(l.quantity ?? 0),
                Number(l.unitPriceMinor ?? 0),
              ),
            })),
          );
          return Promise.resolve(id);
        }
        case "get_invoice": {
          const invoiceId = Number(args.invoiceId);
          const h = state.invoices.find((i) => i.id === invoiceId);
          if (!h) return Promise.reject(new Error("not found"));
          return Promise.resolve({
            header: h,
            lines: state.invoiceLines.get(invoiceId) ?? [],
          });
        }
        case "invoice_set_status": {
          const invoiceId = Number(args.invoiceId);
          const status = String(args.status) as Invoice["status"];
          const inv = state.invoices.find((i) => i.id === invoiceId);
          if (!inv) return Promise.reject(new Error("not found"));
          inv.status = status;
          return Promise.resolve();
        }
        case "invoice_post": {
          const invoiceId = Number(args.invoiceId);
          const inv = state.invoices.find((i) => i.id === invoiceId);
          if (!inv) return Promise.reject(new Error("not found"));
          inv.journalId = state.nextJournalId++;
          return Promise.resolve(inv.journalId);
        }
        case "bill_create": {
          const input = args.input as Record<string, unknown>;
          const id = state.nextBillId++;
          const lines = (input.lines as Array<Record<string, unknown>>) ?? [];
          const total = lines.reduce(
            (acc, l) => acc + Number(l.amountMinor ?? 0),
            0,
          );
          const vendorId =
            input.vendorId === undefined || input.vendorId === null
              ? null
              : Number(input.vendorId);
          const vendorName =
            vendorId === null
              ? null
              : (state.vendors.find((v) => v.id === vendorId)?.displayName ??
                null);
          state.bills.push({
            id,
            vendorId,
            payeeName: input.payeeName ? String(input.payeeName) : null,
            number: String(input.number ?? ""),
            issueDate: String(input.issueDate ?? ""),
            status: "draft",
            journalId: null,
            totalMinor: total,
            vendorName,
          });
          state.billLines.set(
            id,
            lines.map((l, idx) => ({
              lineNumber: idx + 1,
              description: String(l.description ?? ""),
              amountMinor: Number(l.amountMinor ?? 0),
            })),
          );
          return Promise.resolve(id);
        }
        case "get_bill": {
          const billId = Number(args.billId);
          const h = state.bills.find((b) => b.id === billId);
          if (!h) return Promise.reject(new Error("not found"));
          return Promise.resolve({
            header: h,
            lines: state.billLines.get(billId) ?? [],
          });
        }
        case "bill_set_status": {
          const billId = Number(args.billId);
          const status = String(args.status) as Bill["status"];
          const bill = state.bills.find((b) => b.id === billId);
          if (!bill) return Promise.reject(new Error("not found"));
          bill.status = status;
          return Promise.resolve();
        }
        case "bill_post": {
          const billId = Number(args.billId);
          const bill = state.bills.find((b) => b.id === billId);
          if (!bill) return Promise.reject(new Error("not found"));
          bill.journalId = state.nextJournalId++;
          return Promise.resolve(bill.journalId);
        }
        case "list_invoices":
          return Promise.resolve(
            state.invoices.map((i) => ({
              id: i.id,
              number: i.number,
              issueDate: i.issueDate,
              status: i.status,
              totalMinor: i.totalMinor,
              customerName: i.customerName,
            })),
          );
        case "list_bills":
          return Promise.resolve(
            state.bills.map((b) => ({
              id: b.id,
              number: b.number,
              issueDate: b.issueDate,
              status: b.status,
              totalMinor: b.totalMinor,
              vendorName: b.vendorName,
            })),
          );
        case "list_journals":
          return Promise.resolve([]);
        case "customer_create": {
          const input = args.input as Record<string, unknown>;
          const id = state.customers.length + 1;
          state.customers.push({
            id,
            displayName: String(input.displayName ?? ""),
          });
          return Promise.resolve(id);
        }
        case "vendor_create": {
          const input = args.input as Record<string, unknown>;
          const id = state.vendors.length + 1;
          state.vendors.push({
            id,
            displayName: String(input.displayName ?? ""),
          });
          return Promise.resolve(id);
        }
        case "account_create": {
          const input = args.input as Record<string, unknown>;
          const id = state.accounts.length + 1;
          state.accounts.push({
            id,
            code: String(input.code ?? ""),
            name: String(input.name ?? ""),
            accountType: String(input.accountType ?? "expense"),
            isBankCash: Boolean(input.isBankCash),
          });
          return Promise.resolve(id);
        }
        case "account_update":
          return Promise.resolve({ rowsAffected: 1 });
        case "account_deactivate":
          return Promise.resolve({ rowsAffected: 1 });
        case "customer_payment_create": {
          const id = state.nextPaymentId++;
          return Promise.resolve(id);
        }
        case "customer_payment_delete_unposted":
          return Promise.resolve();
        case "customer_payment_post": {
          return Promise.resolve(state.nextJournalId++);
        }
        case "vendor_payment_create": {
          const id = state.nextPaymentId++;
          return Promise.resolve(id);
        }
        case "vendor_payment_delete_unposted":
          return Promise.resolve();
        case "vendor_payment_post": {
          return Promise.resolve(state.nextJournalId++);
        }
        case "vendor_payment_mark_printed":
          return Promise.resolve();
        case "list_vendor_payments":
          return Promise.resolve([]);
        case "get_vendor_payment":
          return Promise.resolve({
            id: Number(args.paymentId ?? 1),
            paymentDate: "2026-01-15",
            amountMinor: 10000,
            paymentMethod: "check",
            method: "check",
            checkNumber: "1001",
            payeeName: "Office Mart",
            checkPayee: "Office Mart",
            checkMemo: null,
            checkStyle: "voucher_top",
            checkPrintedAt: null,
            status: "posted",
            allocations: [],
          });
        case "report_profit_loss":
          return Promise.resolve({
            incomeLines: [{ code: "4000", name: "Sales", amountMinor: 10000 }],
            expenseLines: [
              { code: "5000", name: "Expenses", amountMinor: 3000 },
            ],
            netIncomeMinor: 7000,
          });
        case "report_balance_sheet":
          return Promise.resolve({
            assets: [{ code: "1000", name: "Cash", balanceMinor: 5000 }],
            liabilities: [],
            equity: [{ code: "3000", name: "Equity", balanceMinor: 5000 }],
          });
        case "report_trial_balance":
          return Promise.resolve([
            { code: "1000", name: "Cash", debitMinor: 5000, creditMinor: 0 },
          ]);
        case "report_ar_open":
          return Promise.resolve([
            { customerName: "Acme Corp", openMinor: 5100, invoiceCount: 1 },
          ]);
        case "report_ap_open":
          return Promise.resolve([
            { vendorName: "Office Mart", openMinor: 1200, billCount: 1 },
          ]);
        case "report_general_ledger":
          return Promise.resolve([
            {
              entryDate: "2026-01-01",
              memo: "Opening",
              debitMinor: 100,
              creditMinor: 0,
            },
          ]);
        case "global_search":
          return Promise.resolve({ query: String(args.query ?? ""), hits: [] });
        case "logs_read":
          return Promise.resolve({
            logDir: "C:\\Mock\\Kwikbooks\\logs",
            lines: [
              {
                source: "app",
                level: "info",
                line: "[2026-01-01][12:00:00][INFO] Kwikbooks starting",
              },
              {
                source: "webview",
                level: "warn",
                line: "[2026-01-01][12:00:01][WARN] [Settings] check started",
              },
              {
                source: "app",
                level: "info",
                line: "[2026-01-01][12:00:02][INFO] invoke_ok seq=99 rid=e2e-mock",
              },
            ],
          });
        case "logs_export_support_bundle":
          return Promise.resolve({
            path: String(args.destinationPath ?? "C:\\Mock\\support.txt"),
            bytesWritten: 128,
            lineCount: 2,
          });
        case "db_backup_vacuum":
          state.backupPath = String(args.destinationPath ?? state.backupPath);
          return Promise.resolve();
        case "db_restore_validate":
          return Promise.resolve({ ok: true, migrationVersion: 6 });
        case "db_restore_apply":
          return Promise.resolve();
        case "ipc_context_set":
          return Promise.resolve();
        case "db_migrate":
          return Promise.resolve({
            migrationVersionBefore: 4,
            migrationVersionAfter: 4,
          });
        case "account_get":
          return Promise.resolve({
            id: Number(args.id),
            code: "1000",
            name: "Cash",
            accountType: "asset",
          });
        case "import_quickbooks_file":
          return Promise.resolve({
            formatDetected: "iif",
            accountsCreated: 0,
            customersCreated: 0,
            vendorsCreated: 0,
            itemsCreated: 0,
            rowsSkipped: 0,
            warnings: [],
          });
        case "plugin:dialog|save":
          return Promise.resolve(state.backupPath);
        case "plugin:dialog|open":
          return Promise.resolve(state.backupPath);
        default:
          if (command.startsWith("plugin:")) {
            return Promise.resolve(null);
          }
          return Promise.reject(
            new Error(`E2E mock: unhandled IPC command "${command}"`),
          );
      }
    }

    const win = window as Window & {
      __TAURI_INTERNALS__?: unknown;
      isTauri?: boolean;
    };
    win.isTauri = true;
    win.__TAURI_INTERNALS__ = {
      invoke,
      transformCallback: () => 1,
    };
    (win as Window & { __E2E_MOCK__?: unknown }).__E2E_MOCK__ = controls;
  });
}
