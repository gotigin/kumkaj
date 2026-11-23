-- Stripe billing removed (India Tax incompatible). Clear unused price ids.
UPDATE plans SET stripe_price_id = NULL
WHERE stripe_price_id IS NOT NULL;

INSERT INTO schema_meta (version) VALUES (4)
ON CONFLICT (version) DO NOTHING;
