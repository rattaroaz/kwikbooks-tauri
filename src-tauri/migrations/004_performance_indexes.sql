-- Phase 9: extra indexes for date-scoped listings and FK-heavy joins.

CREATE INDEX IF NOT EXISTS idx_invoice_company_issue
  ON invoice (company_id, issue_date DESC);

CREATE INDEX IF NOT EXISTS idx_bill_company_issue
  ON bill (company_id, issue_date DESC);

CREATE INDEX IF NOT EXISTS idx_journal_line_journal_account
  ON journal_line (journal_id, account_id);
