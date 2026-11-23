-- Per-product Paddle sandbox prices.
-- Receipt naming: Company (seller) → Brand Kumkaj → Product → Plan (price name).
-- Product entity name in Paddle: Kumkaj · Tabiyat
UPDATE plans SET paddle_price_id = 'pri_01m844kgzn2xat61034y7f9cep' WHERE plan_id = 'steady';
UPDATE plans SET paddle_price_id = 'pri_01m844kha45fvvgkemwhb57dkv' WHERE plan_id = 'swift';
UPDATE plans SET paddle_price_id = 'pri_01m844khjyf1p8n4w2npms8r18' WHERE plan_id = 'priority';
