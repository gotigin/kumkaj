-- Per-product Paddle sandbox prices.
-- Receipt naming: Company (seller) → Brand Kumkaj → Product → Plan (price name).
-- Product entity name in Paddle: Kumkaj · Khaynjo
UPDATE plans SET paddle_price_id = 'pri_01m843vtnmjy22408ny5600z0a' WHERE plan_id = 'steady';
UPDATE plans SET paddle_price_id = 'pri_01m843xnr3e9zzgskb65rgp7hz' WHERE plan_id = 'swift';
UPDATE plans SET paddle_price_id = 'pri_01m843zgqec0nsv4kd9rb33n2y' WHERE plan_id = 'priority';
