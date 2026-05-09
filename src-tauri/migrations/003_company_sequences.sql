-- Document numbering defaults (UI can advance; optional automation later).
ALTER TABLE company ADD COLUMN next_invoice_number INTEGER NOT NULL DEFAULT 1000;

ALTER TABLE company ADD COLUMN next_bill_number INTEGER NOT NULL DEFAULT 1000;
