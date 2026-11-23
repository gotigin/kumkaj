-- Stripe Test Price ids (sandbox). Safe to re-run; live flip uses different ids later.
UPDATE plans SET stripe_price_id = 'price_1UesfVKJLz4z9U1Wk4J9JSqZ' WHERE plan_id = 'steady';
UPDATE plans SET stripe_price_id = 'price_1UesfVKJLz4z9U1WpxaA7z78' WHERE plan_id = 'swift';
UPDATE plans SET stripe_price_id = 'price_1UesfWKJLz4z9U1W2cEzPmCu' WHERE plan_id = 'priority';

INSERT INTO schema_meta (version) VALUES (3)
ON CONFLICT (version) DO NOTHING;
