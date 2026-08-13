-- Prevent duplicate printed check numbers within a company.
-- If legacy duplicates exist, keep the lowest id and clear later check_numbers
-- so the unique index can be created (those payments remain posted).
UPDATE vendor_payment
SET check_number = NULL
WHERE payment_method = 'check'
  AND check_number IS NOT NULL
  AND id NOT IN (
    SELECT MIN(id)
    FROM vendor_payment
    WHERE payment_method = 'check' AND check_number IS NOT NULL
    GROUP BY company_id, check_number
  );

CREATE UNIQUE INDEX IF NOT EXISTS idx_vendor_payment_check_number
  ON vendor_payment (company_id, check_number)
  WHERE payment_method = 'check' AND check_number IS NOT NULL;
