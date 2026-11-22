-- Billing milestone 2: checkout + webhook ledger for Stripe / Paddle / mock.
-- Prices stay tax-exclusive INR (docs/CLOUD_PLANS.md). Gateway adds tax.

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS stripe_price_id TEXT,
    ADD COLUMN IF NOT EXISTS paddle_price_id TEXT;

COMMENT ON COLUMN plans.price_inr_paise IS
    'Raw yearly list price in INR paise (tax exclusive). Gateway adds tax.';
COMMENT ON COLUMN plans.stripe_price_id IS
    'Stripe Price id (price_…) for this plan; sandbox and live use different Dashboards.';
COMMENT ON COLUMN plans.paddle_price_id IS
    'Paddle Price id (pri_…) for this plan; sandbox and live use different accounts.';

ALTER TABLE subscriptions
    ADD COLUMN IF NOT EXISTS provider TEXT,
    ADD COLUMN IF NOT EXISTS external_subscription_id TEXT,
    ADD COLUMN IF NOT EXISTS last_checkout_id UUID;

CREATE TABLE billing_checkouts (
    checkout_id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts (account_id) ON DELETE CASCADE,
    plan_id TEXT NOT NULL REFERENCES plans (plan_id),
    provider TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    external_id TEXT,
    checkout_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT billing_checkouts_provider_chk
        CHECK (provider IN ('mock', 'stripe', 'paddle')),
    CONSTRAINT billing_checkouts_status_chk
        CHECK (status IN ('pending', 'completed', 'expired', 'failed'))
);

CREATE INDEX idx_billing_checkouts_account
    ON billing_checkouts (account_id, created_at DESC);
CREATE INDEX idx_billing_checkouts_external
    ON billing_checkouts (provider, external_id);

CREATE TABLE billing_events (
    provider TEXT NOT NULL,
    event_id TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (provider, event_id)
);

INSERT INTO schema_meta (version) VALUES (2)
ON CONFLICT (version) DO NOTHING;
