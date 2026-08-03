-- Phase 3: single-company books, chart of accounts, journals, AR/AP documents.
-- Amounts are INTEGER minor units (e.g. cents) everywhere except line quantity (REAL).

-- ---------------------------------------------------------------------------
-- Company (single row for v1; multi-tenant can add rows later)
-- ---------------------------------------------------------------------------
CREATE TABLE company (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT NOT NULL,
  legal_name TEXT,
  fiscal_year_start_month INTEGER NOT NULL DEFAULT 1
    CHECK (fiscal_year_start_month BETWEEN 1 AND 12),
  base_currency_code TEXT NOT NULL DEFAULT 'USD',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ---------------------------------------------------------------------------
-- Chart of accounts
-- ---------------------------------------------------------------------------
CREATE TABLE account (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  code TEXT NOT NULL,
  name TEXT NOT NULL,
  account_type TEXT NOT NULL CHECK (
    account_type IN ('asset', 'liability', 'equity', 'income', 'expense')
  ),
  parent_id INTEGER REFERENCES account (id),
  is_bank_cash INTEGER NOT NULL DEFAULT 0 CHECK (is_bank_cash IN (0, 1)),
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  sort_order INTEGER NOT NULL DEFAULT 0,
  UNIQUE (company_id, code)
);

CREATE INDEX idx_account_company ON account (company_id);
CREATE INDEX idx_account_parent ON account (parent_id);
CREATE INDEX idx_account_type ON account (company_id, account_type);

-- ---------------------------------------------------------------------------
-- Journals (balanced debits/credits enforced in application layer)
-- ---------------------------------------------------------------------------
CREATE TABLE journal (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  entry_date TEXT NOT NULL,
  memo TEXT,
  source_kind TEXT CHECK (
    source_kind IS NULL OR source_kind IN (
      'manual',
      'invoice',
      'bill',
      'payment_customer',
      'payment_vendor',
      'opening_balance',
      'adjustment'
    )
  ),
  source_id INTEGER,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX idx_journal_source ON journal (company_id, source_kind, source_id)
  WHERE source_kind IS NOT NULL AND source_id IS NOT NULL;

CREATE INDEX idx_journal_company_date ON journal (company_id, entry_date);

CREATE TABLE journal_line (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  journal_id INTEGER NOT NULL REFERENCES journal (id) ON DELETE CASCADE,
  account_id INTEGER NOT NULL REFERENCES account (id),
  line_number INTEGER NOT NULL,
  description TEXT,
  debit_minor INTEGER NOT NULL DEFAULT 0 CHECK (debit_minor >= 0),
  credit_minor INTEGER NOT NULL DEFAULT 0 CHECK (credit_minor >= 0),
  UNIQUE (journal_id, line_number)
);

CREATE INDEX idx_journal_line_journal ON journal_line (journal_id);
CREATE INDEX idx_journal_line_account ON journal_line (account_id);

-- ---------------------------------------------------------------------------
-- Customers & vendors
-- ---------------------------------------------------------------------------
CREATE TABLE customer (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  display_name TEXT NOT NULL,
  email TEXT,
  phone TEXT,
  terms_days INTEGER NOT NULL DEFAULT 30 CHECK (terms_days >= 0),
  notes TEXT,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_customer_company ON customer (company_id);

CREATE TABLE vendor (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  display_name TEXT NOT NULL,
  email TEXT,
  phone TEXT,
  notes TEXT,
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_vendor_company ON vendor (company_id);

-- ---------------------------------------------------------------------------
-- Items (products / services)
-- ---------------------------------------------------------------------------
CREATE TABLE item (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  sku TEXT,
  unit_price_minor INTEGER NOT NULL DEFAULT 0,
  default_income_account_id INTEGER REFERENCES account (id),
  default_expense_account_id INTEGER REFERENCES account (id),
  is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_item_company ON item (company_id);

-- ---------------------------------------------------------------------------
-- Sales: invoices
-- ---------------------------------------------------------------------------
CREATE TABLE invoice (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  customer_id INTEGER NOT NULL REFERENCES customer (id),
  number TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft' CHECK (
    status IN ('draft', 'sent', 'paid', 'void')
  ),
  issue_date TEXT NOT NULL,
  due_date TEXT,
  memo TEXT,
  journal_id INTEGER UNIQUE REFERENCES journal (id),
  subtotal_minor INTEGER NOT NULL DEFAULT 0,
  tax_minor INTEGER NOT NULL DEFAULT 0,
  total_minor INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (company_id, number)
);

CREATE INDEX idx_invoice_company ON invoice (company_id);
CREATE INDEX idx_invoice_customer ON invoice (customer_id);
CREATE INDEX idx_invoice_status ON invoice (company_id, status);

CREATE TABLE invoice_line (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  invoice_id INTEGER NOT NULL REFERENCES invoice (id) ON DELETE CASCADE,
  line_number INTEGER NOT NULL,
  item_id INTEGER REFERENCES item (id),
  description TEXT NOT NULL,
  quantity REAL NOT NULL DEFAULT 1 CHECK (quantity > 0),
  unit_price_minor INTEGER NOT NULL,
  line_total_minor INTEGER NOT NULL,
  income_account_id INTEGER REFERENCES account (id),
  UNIQUE (invoice_id, line_number)
);

CREATE INDEX idx_invoice_line_invoice ON invoice_line (invoice_id);

-- ---------------------------------------------------------------------------
-- Purchases: bills / expenses
-- ---------------------------------------------------------------------------
CREATE TABLE bill (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  vendor_id INTEGER REFERENCES vendor (id),
  payee_name TEXT,
  number TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft' CHECK (
    status IN ('draft', 'open', 'paid', 'void')
  ),
  issue_date TEXT NOT NULL,
  due_date TEXT,
  memo TEXT,
  journal_id INTEGER UNIQUE REFERENCES journal (id),
  total_minor INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (company_id, number)
);

CREATE INDEX idx_bill_company ON bill (company_id);
CREATE INDEX idx_bill_vendor ON bill (vendor_id);
CREATE INDEX idx_bill_status ON bill (company_id, status);

CREATE TABLE bill_line (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  bill_id INTEGER NOT NULL REFERENCES bill (id) ON DELETE CASCADE,
  line_number INTEGER NOT NULL,
  description TEXT NOT NULL,
  amount_minor INTEGER NOT NULL,
  expense_account_id INTEGER NOT NULL REFERENCES account (id),
  UNIQUE (bill_id, line_number)
);

CREATE INDEX idx_bill_line_bill ON bill_line (bill_id);

-- ---------------------------------------------------------------------------
-- Payments (cash/bank account_id points at an asset account with is_bank_cash = 1)
-- ---------------------------------------------------------------------------
CREATE TABLE customer_payment (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  customer_id INTEGER NOT NULL REFERENCES customer (id),
  bank_account_id INTEGER NOT NULL REFERENCES account (id),
  payment_date TEXT NOT NULL,
  amount_minor INTEGER NOT NULL CHECK (amount_minor > 0),
  memo TEXT,
  journal_id INTEGER UNIQUE REFERENCES journal (id),
  invoice_id INTEGER REFERENCES invoice (id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_cpay_company ON customer_payment (company_id);
CREATE INDEX idx_cpay_customer ON customer_payment (customer_id);

CREATE TABLE vendor_payment (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  company_id INTEGER NOT NULL REFERENCES company (id) ON DELETE CASCADE,
  vendor_id INTEGER NOT NULL REFERENCES vendor (id),
  bank_account_id INTEGER NOT NULL REFERENCES account (id),
  payment_date TEXT NOT NULL,
  amount_minor INTEGER NOT NULL CHECK (amount_minor > 0),
  memo TEXT,
  journal_id INTEGER UNIQUE REFERENCES journal (id),
  bill_id INTEGER REFERENCES bill (id),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_vpay_company ON vendor_payment (company_id);
CREATE INDEX idx_vpay_vendor ON vendor_payment (vendor_id);

-- ---------------------------------------------------------------------------
-- Bank register stub (per balance-sheet bank/cash account)
-- ---------------------------------------------------------------------------
CREATE TABLE reconciliation_stub (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  account_id INTEGER NOT NULL UNIQUE REFERENCES account (id) ON DELETE CASCADE,
  last_statement_end_date TEXT,
  statement_ending_balance_minor INTEGER,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ---------------------------------------------------------------------------
-- Bootstrap single company + minimal chart of accounts (edit freely in app later)
-- ---------------------------------------------------------------------------
INSERT OR IGNORE INTO company (id, name) VALUES (1, 'My Company');

INSERT OR IGNORE INTO account (company_id, code, name, account_type, is_bank_cash, sort_order)
VALUES
  (1, '1000', 'Cash', 'asset', 1, 10),
  (1, '1100', 'Accounts Receivable', 'asset', 0, 20),
  (1, '2000', 'Accounts Payable', 'liability', 0, 30),
  (1, '2100', 'Sales Tax Payable', 'liability', 0, 35),
  (1, '3000', 'Opening Balance Equity', 'equity', 0, 40),
  (1, '4000', 'Sales', 'income', 0, 50),
  (1, '5000', 'Expenses', 'expense', 0, 60);
