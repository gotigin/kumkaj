-- Per-product Paddle sandbox prices.
-- Receipt naming: Company (seller) → Brand Kumkaj → Product → Plan (price name).
-- Product entity name in Paddle: Kumkaj · Kirano
UPDATE plans SET paddle_price_id = 'pri_01m844ajvebt7fdqpv5jaxtv5w' WHERE plan_id = 'steady';
UPDATE plans SET paddle_price_id = 'pri_01m844cdvrmq1n2ht47cnyqpvv' WHERE plan_id = 'swift';
UPDATE plans SET paddle_price_id = 'pri_01m844e8wv3ws14g4gxavjrwak' WHERE plan_id = 'priority';
