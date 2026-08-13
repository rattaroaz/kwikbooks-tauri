-- Sales tax collected from customers is a liability, not revenue.
INSERT OR IGNORE INTO account (company_id, code, name, account_type, is_bank_cash, sort_order)
VALUES (1, '2100', 'Sales Tax Payable', 'liability', 0, 35);
