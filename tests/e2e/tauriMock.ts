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
    const state = {
      nextInvoiceId: 1,
      nextBillId: 1,
      nextJournalId: 1,
      company: {
        name: "Mock Company",
        legalName: "Mock Company LLC",
        fiscalYearStartMonth: 1,
        baseCurrencyCode: "USD",
        nextInvoiceNumber: 1000,
        nextBillNumber: 2000,
      },
      customers: [{ id: 1, displayName: "Acme Corp" }] as Customer[],
      vendors: [{ id: 1, displayName: "Office Mart" }] as Vendor[],
      accounts: [
        { id: 1, code: "5000", name: "Expenses", accountType: "expense" },
      ] as Account[],
      invoices: [] as Invoice[],
      invoiceLines: new Map<number, Array<Record<string, unknown>>>(),
      bills: [] as Bill[],
      billLines: new Map<number, Array<Record<string, unknown>>>(),
      backupPath: "C:/tmp/kwikbooks-backup.sqlite",
      commandErrors: {} as Record<string, string>,
    };

    const controls = {
      setCustomers(customers: Array<{ id: number; displayName: string }>) {
        state.customers = customers;
      },
      setVendors(vendors: Array<{ id: number; displayName: string }>) {
        state.vendors = vendors;
      },
      setCommandError(command: string, message: string | null) {
        if (message === null) {
          delete state.commandErrors[command];
          return;
        }
        state.commandErrors[command] = message;
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
            migrationVersion: 4,
          });
        case "health_ping":
          return Promise.resolve({
            ok: true,
            sqliteOk: true,
            migrationVersion: 4,
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
              Math.round(
                Number(l.quantity ?? 0) * Number(l.unitPriceMinor ?? 0),
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
              lineTotalMinor: Math.round(
                Number(l.quantity ?? 0) * Number(l.unitPriceMinor ?? 0),
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
        case "db_backup_vacuum":
          state.backupPath = String(args.destinationPath ?? state.backupPath);
          return Promise.resolve();
        case "db_restore_validate":
          return Promise.resolve({ ok: true, migrationVersion: 4 });
        case "db_restore_apply":
          return Promise.resolve();
        case "plugin:dialog|save":
          return Promise.resolve(state.backupPath);
        case "plugin:dialog|open":
          return Promise.resolve(state.backupPath);
        default:
          return Promise.resolve(null);
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
