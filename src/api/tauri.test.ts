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

    expect(invokeMock).toHaveBeenNthCalledWith(1, "account_get", { id: 10 });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "account_create", { input: payload });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "account_update", {
      input: { id: 10, ...payload },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(4, "account_deactivate", { id: 10 });
    expect(invokeMock).toHaveBeenNthCalledWith(5, "company_get");
    expect(invokeMock).toHaveBeenNthCalledWith(6, "company_update", {
      input: { name: "Acme" },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(7, "list_customers");
    expect(invokeMock).toHaveBeenNthCalledWith(8, "list_vendors");
    expect(invokeMock).toHaveBeenNthCalledWith(9, "list_invoices");
    expect(invokeMock).toHaveBeenNthCalledWith(10, "list_bills");
    expect(invokeMock).toHaveBeenNthCalledWith(11, "list_journals", { limit: 15 });
    expect(invokeMock).toHaveBeenNthCalledWith(12, "get_invoice", { invoiceId: 100 });
    expect(invokeMock).toHaveBeenNthCalledWith(13, "get_bill", { billId: 200 });
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
    expect(invokeMock).toHaveBeenNthCalledWith(1, "customer_create", { input: customer });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "vendor_create", { input: vendor });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "invoice_create", { input: inv });
    expect(invokeMock).toHaveBeenNthCalledWith(4, "bill_create", { input: bill });
    expect(invokeMock).toHaveBeenNthCalledWith(5, "customer_payment_create", {
      input: customerPayment,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(6, "vendor_payment_create", {
      input: vendorPayment,
    });
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
    expect(invokeMock).toHaveBeenNthCalledWith(1, "invoice_post", { invoiceId: 7 });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "bill_post", { billId: 8 });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "customer_payment_post", { paymentId: 9 });
    expect(invokeMock).toHaveBeenNthCalledWith(4, "vendor_payment_post", { paymentId: 10 });
    expect(invokeMock).toHaveBeenNthCalledWith(5, "invoice_set_status", {
      invoiceId: 7,
      status: "sent",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(6, "bill_set_status", {
      billId: 8,
      status: "open",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(7, "report_trial_balance", {
      dateFrom: "2026-01-01",
      dateTo: "2026-01-31",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(8, "report_general_ledger", {
      accountId: 10,
      dateFrom: "2026-01-01",
      dateTo: "2026-01-31",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(9, "report_ar_open");
    expect(invokeMock).toHaveBeenNthCalledWith(10, "report_ap_open");
    expect(invokeMock).toHaveBeenNthCalledWith(11, "report_profit_loss", {
      dateFrom: "2026-01-01",
      dateTo: "2026-01-31",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(12, "report_balance_sheet", {
      asOfDate: "2026-01-31",
    });
  });

  it("uses backup/restore command names and keys", async () => {
    await dbBackupVacuum("C:/tmp/a.sqlite");
    await dbRestoreValidate("C:/tmp/a.sqlite");
    await dbRestoreApply("C:/tmp/a.sqlite");
    expect(invokeMock).toHaveBeenNthCalledWith(1, "db_backup_vacuum", {
      destinationPath: "C:/tmp/a.sqlite",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "db_restore_validate", {
      sourcePath: "C:/tmp/a.sqlite",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "db_restore_apply", {
      sourcePath: "C:/tmp/a.sqlite",
    });
  });
});
