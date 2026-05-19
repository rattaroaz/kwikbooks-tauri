import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import {
  accountCreate,
  accountDeactivate,
  accountGet,
  accountList,
  accountUpdate,
  billCreate,
  billPost,
  billSetStatus,
  companyGet,
  companyUpdate,
  customerCreate,
  customerPaymentCreate,
  customerPaymentPost,
  dbBackupVacuum,
  dbRestoreApply,
  dbRestoreValidate,
  getBill,
  getInvoice,
  invoiceCreate,
  invoicePost,
  invoiceSetStatus,
  listBills,
  listCustomers,
  listInvoices,
  listJournals,
  listVendors,
  reportApOpen,
  reportArOpen,
  reportBalanceSheet,
  reportGeneralLedger,
  reportProfitLoss,
  reportTrialBalance,
  vendorCreate,
  vendorPaymentCreate,
  vendorPaymentPost,
} from "./tauri";

function ipcCommandCalls(): unknown[][] {
  return invokeMock.mock.calls.filter((c) => c[0] !== "ipc_context_set");
}

describe("tauri API contract wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it("sends account_list with filter payload", async () => {
    await accountList({ accountType: "expense", activeOnly: true });
    expect(invokeMock).toHaveBeenCalledWith("account_list", {
      filter: { accountType: "expense", activeOnly: true },
    });
  });

  it("maps account/company/list/get wrappers", async () => {
    const payload = { code: "1000" };
    await accountGet(10);
    await accountCreate(payload);
    await accountUpdate({ id: 10, ...payload });
    await accountDeactivate(10);
    await companyGet();
    await companyUpdate({ name: "Acme" });
    await listCustomers();
    await listVendors();
    await listInvoices();
    await listBills();
    await listJournals(15);
    await getInvoice(100);
    await getBill(200);

    const calls = ipcCommandCalls();
    expect(calls[0]).toEqual(["account_get", { id: 10 }]);
    expect(calls[1]).toEqual(["account_create", { input: payload }]);
    expect(calls[2]).toEqual(["account_update", { input: { id: 10, ...payload } }]);
    expect(calls[3]).toEqual(["account_deactivate", { id: 10 }]);
    expect(calls[4]).toEqual(["company_get"]);
    expect(calls[5]).toEqual(["company_update", { input: { name: "Acme" } }]);
    expect(calls[6]).toEqual(["list_customers"]);
    expect(calls[7]).toEqual(["list_vendors"]);
    expect(calls[8]).toEqual(["list_invoices"]);
    expect(calls[9]).toEqual(["list_bills"]);
    expect(calls[10]).toEqual(["list_journals", { limit: 15 }]);
    expect(calls[11]).toEqual(["get_invoice", { invoiceId: 100 }]);
    expect(calls[12]).toEqual(["get_bill", { billId: 200 }]);
  });

  it("sends document create commands under input key", async () => {
    const inv = { number: "INV-1" };
    const bill = { number: "B-1" };
    const customer = { displayName: "Acme" };
    const vendor = { displayName: "Office Mart" };
    const customerPayment = { customerId: 1, amountMinor: 1000 };
    const vendorPayment = { vendorId: 1, amountMinor: 500 };
    await customerCreate(customer);
    await vendorCreate(vendor);
    await invoiceCreate(inv);
    await billCreate(bill);
    await customerPaymentCreate(customerPayment);
    await vendorPaymentCreate(vendorPayment);
    const calls = ipcCommandCalls();
    expect(calls[0]).toEqual(["customer_create", { input: customer }]);
    expect(calls[1]).toEqual(["vendor_create", { input: vendor }]);
    expect(calls[2]).toEqual(["invoice_create", { input: inv }]);
    expect(calls[3]).toEqual(["bill_create", { input: bill }]);
    expect(calls[4]).toEqual(["customer_payment_create", { input: customerPayment }]);
    expect(calls[5]).toEqual(["vendor_payment_create", { input: vendorPayment }]);
  });

  it("sends posting/lifecycle/report args in camelCase", async () => {
    await invoicePost(7);
    await billPost(8);
    await customerPaymentPost(9);
    await vendorPaymentPost(10);
    await invoiceSetStatus(7, "sent");
    await billSetStatus(8, "open");
    await reportTrialBalance("2026-01-01", "2026-01-31");
    await reportGeneralLedger(10, "2026-01-01", "2026-01-31");
    await reportArOpen();
    await reportApOpen();
    await reportProfitLoss("2026-01-01", "2026-01-31");
    await reportBalanceSheet("2026-01-31");
    const calls = ipcCommandCalls();
    expect(calls[0]).toEqual(["invoice_post", { invoiceId: 7 }]);
    expect(calls[1]).toEqual(["bill_post", { billId: 8 }]);
    expect(calls[2]).toEqual(["customer_payment_post", { paymentId: 9 }]);
    expect(calls[3]).toEqual(["vendor_payment_post", { paymentId: 10 }]);
    expect(calls[4]).toEqual(["invoice_set_status", { invoiceId: 7, status: "sent" }]);
    expect(calls[5]).toEqual(["bill_set_status", { billId: 8, status: "open" }]);
    expect(calls[6]).toEqual([
      "report_trial_balance",
      { dateFrom: "2026-01-01", dateTo: "2026-01-31" },
    ]);
    expect(calls[7]).toEqual([
      "report_general_ledger",
      { accountId: 10, dateFrom: "2026-01-01", dateTo: "2026-01-31" },
    ]);
    expect(calls[8]).toEqual(["report_ar_open"]);
    expect(calls[9]).toEqual(["report_ap_open"]);
    expect(calls[10]).toEqual([
      "report_profit_loss",
      { dateFrom: "2026-01-01", dateTo: "2026-01-31" },
    ]);
    expect(calls[11]).toEqual(["report_balance_sheet", { asOfDate: "2026-01-31" }]);
  });

  it("uses backup/restore command names and keys", async () => {
    await dbBackupVacuum("C:/tmp/a.sqlite");
    await dbRestoreValidate("C:/tmp/a.sqlite");
    await dbRestoreApply("C:/tmp/a.sqlite");
    const calls = ipcCommandCalls();
    expect(calls[0]).toEqual([
      "db_backup_vacuum",
      { destinationPath: "C:/tmp/a.sqlite" },
    ]);
    expect(calls[1]).toEqual([
      "db_restore_validate",
      { sourcePath: "C:/tmp/a.sqlite" },
    ]);
    expect(calls[2]).toEqual([
      "db_restore_apply",
      { sourcePath: "C:/tmp/a.sqlite" },
    ]);
  });
});
