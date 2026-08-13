-- Check printing: company address / check sequence / stock style; vendor payment check fields.

ALTER TABLE company ADD COLUMN address_line1 TEXT;
ALTER TABLE company ADD COLUMN address_line2 TEXT;
ALTER TABLE company ADD COLUMN city TEXT;
ALTER TABLE company ADD COLUMN region TEXT;
ALTER TABLE company ADD COLUMN postal_code TEXT;
ALTER TABLE company ADD COLUMN next_check_number INTEGER NOT NULL DEFAULT 1000;
ALTER TABLE company ADD COLUMN default_check_style TEXT NOT NULL DEFAULT 'voucher_top'
  CHECK (default_check_style IN ('voucher_top', 'voucher_middle', 'voucher_bottom', 'generic'));

ALTER TABLE vendor_payment ADD COLUMN check_number TEXT;
ALTER TABLE vendor_payment ADD COLUMN payment_method TEXT NOT NULL DEFAULT 'other'
  CHECK (payment_method IN ('check', 'other'));
ALTER TABLE vendor_payment ADD COLUMN payee_name TEXT;
ALTER TABLE vendor_payment ADD COLUMN check_printed_at TEXT;
