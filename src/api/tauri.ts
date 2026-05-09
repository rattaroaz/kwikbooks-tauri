import { invoke } from "./invoke";

export * from "./db";

/** Backup / restore (SQLite file) */
export async function dbBackupVacuum(destinationPath: string) {
  return invoke<void>("db_backup_vacuum", { destinationPath });
}

export async function dbRestoreValidate(sourcePath: string) {
  return invoke<{ ok: boolean; migrationVersion: number }>(
    "db_restore_validate",
    { sourcePath },
  );
}

export async function dbRestoreApply(sourcePath: string) {
  return invoke<void>("db_restore_apply", { sourcePath });
}

export type JsonObject = Record<string, unknown>;

/** Accounts */
export async function accountList(filter: JsonObject = {}) {
  return invoke<unknown[]>("account_list", { filter });
}

export async function accountGet(id: number) {
  return invoke<JsonObject>("account_get", { id });
}

export async function accountCreate(payload: JsonObject) {
  return invoke<number>("account_create", { input: payload });
}

export async function accountUpdate(payload: JsonObject) {
  return invoke<{ rowsAffected: number }>("account_update", { input: payload });
}

export async function accountDeactivate(id: number) {
  return invoke<{ rowsAffected: number }>("account_deactivate", { id });
}

/** Company */
export async function companyGet() {
  return invoke<JsonObject>("company_get");
}

export async function companyUpdate(payload: JsonObject) {
  return invoke<void>("company_update", { input: payload });
}

/** Lists */
export async function listCustomers() {
  return invoke<unknown[]>("list_customers");
}

export async function listVendors() {
  return invoke<unknown[]>("list_vendors");
}

export async function listInvoices() {
  return invoke<unknown[]>("list_invoices");
}

export async function listBills() {
  return invoke<unknown[]>("list_bills");
}

export async function listJournals(limit?: number) {
  return invoke<unknown[]>("list_journals", { limit });
}

export async function getInvoice(invoiceId: number) {
  return invoke<{ header: JsonObject; lines: unknown[] }>("get_invoice", {
    invoiceId,
  });
}

export async function getBill(billId: number) {
  return invoke<{ header: JsonObject; lines: unknown[] }>("get_bill", {
    billId,
  });
}

/** Documents */
export async function customerCreate(payload: JsonObject) {
  return invoke<number>("customer_create", { input: payload });
}

export async function vendorCreate(payload: JsonObject) {
  return invoke<number>("vendor_create", { input: payload });
}

export async function invoiceCreate(payload: JsonObject) {
  return invoke<number>("invoice_create", { input: payload });
}

export async function billCreate(payload: JsonObject) {
  return invoke<number>("bill_create", { input: payload });
}

export async function customerPaymentCreate(payload: JsonObject) {
  return invoke<number>("customer_payment_create", { input: payload });
}

export async function vendorPaymentCreate(payload: JsonObject) {
  return invoke<number>("vendor_payment_create", { input: payload });
}

/** Posting & lifecycle */
export async function invoicePost(invoiceId: number) {
  return invoke<number>("invoice_post", { invoiceId });
}

export async function billPost(billId: number) {
  return invoke<number>("bill_post", { billId });
}

export async function customerPaymentPost(paymentId: number) {
  return invoke<number>("customer_payment_post", { paymentId });
}

export async function vendorPaymentPost(paymentId: number) {
  return invoke<number>("vendor_payment_post", { paymentId });
}

export async function invoiceSetStatus(invoiceId: number, status: string) {
  return invoke<void>("invoice_set_status", { invoiceId, status });
}

export async function billSetStatus(billId: number, status: string) {
  return invoke<void>("bill_set_status", { billId, status });
}

/** Reports */
export async function reportTrialBalance(dateFrom: string, dateTo: string) {
  return invoke<unknown[]>("report_trial_balance", { dateFrom, dateTo });
}

export async function reportGeneralLedger(
  accountId: number,
  dateFrom: string,
  dateTo: string,
) {
  return invoke<unknown[]>("report_general_ledger", {
    accountId,
    dateFrom,
    dateTo,
  });
}

export async function reportArOpen() {
  return invoke<unknown[]>("report_ar_open");
}

export async function reportApOpen() {
  return invoke<unknown[]>("report_ap_open");
}

export async function reportProfitLoss(dateFrom: string, dateTo: string) {
  return invoke<JsonObject>("report_profit_loss", { dateFrom, dateTo });
}

export async function reportBalanceSheet(asOfDate: string) {
  return invoke<JsonObject>("report_balance_sheet", { asOfDate });
}
