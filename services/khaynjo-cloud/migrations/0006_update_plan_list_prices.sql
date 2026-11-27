-- List prices (ex-tax INR paise): Swift ₹14,000 · Priority ₹28,000.
-- Steady stays ₹6,000. Applies to already-migrated DBs; 0001_init seeds match.
UPDATE plans SET price_inr_paise = 1400000 WHERE plan_id = 'swift';
UPDATE plans SET price_inr_paise = 2800000 WHERE plan_id = 'priority';
