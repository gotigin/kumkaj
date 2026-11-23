-- Per-product Paddle sandbox prices.
-- Receipt naming: Company (seller) → Brand Kumkaj → Product → Plan (price name).
-- Product entity name in Paddle: Kumkaj · Moklanjo
UPDATE plans SET paddle_price_id = 'pri_01m84436qktc3jdbz4edhwq7a2' WHERE plan_id = 'steady';
UPDATE plans SET paddle_price_id = 'pri_01m84451qq75znv94aah4wzyj1' WHERE plan_id = 'swift';
UPDATE plans SET paddle_price_id = 'pri_01m8446wv71x8ngq96qrykeqat' WHERE plan_id = 'priority';
