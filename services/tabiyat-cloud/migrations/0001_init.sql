-- Tabiyat cloud schema v1 (additive thereafter: 0002_…, never reuse numbers).

CREATE TABLE schema_meta (
    version BIGINT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO schema_meta (version) VALUES (1);

CREATE TABLE accounts (
    account_id UUID PRIMARY KEY,
    firebase_uid TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    email TEXT,
    phone TEXT,
    display_name TEXT,
    product_code TEXT NOT NULL DEFAULT 'tabiyat',
    auth_issuer TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT accounts_product_tabiyat CHECK (product_code = 'tabiyat')
);

-- Empty for now. Later: link to Kumkaj brand / Gotigin subjects without merging DBs.
CREATE TABLE account_external_links (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts (account_id) ON DELETE CASCADE,
    system TEXT NOT NULL,
    external_subject TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (system, external_subject)
);

CREATE INDEX idx_account_external_links_account
    ON account_external_links (account_id);

CREATE TABLE plans (
    plan_id TEXT PRIMARY KEY,
    sort_order INT NOT NULL,
    title TEXT NOT NULL,
    quota_gb INT,
    bandwidth_band TEXT NOT NULL,
    price_inr_paise BIGINT,
    includes_live_cloud BOOLEAN NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE subscriptions (
    account_id UUID PRIMARY KEY REFERENCES accounts (account_id) ON DELETE CASCADE,
    plan_id TEXT NOT NULL REFERENCES plans (plan_id),
    status TEXT NOT NULL DEFAULT 'active',
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ends_at TIMESTAMPTZ
);

-- Catalog from docs/CLOUD_PLANS.md (INR GST-inclusive list prices × 100 paise).
INSERT INTO plans (plan_id, sort_order, title, quota_gb, bandwidth_band, price_inr_paise, includes_live_cloud) VALUES
    ('free', 1, 'Free forever', NULL, 'none', 0, FALSE),
    ('steady', 2, 'Steady', 10, 'lower', 600000, TRUE),
    ('swift', 3, 'Swift', 25, 'mid', 1400000, TRUE),
    ('priority', 4, 'Priority', 50, 'highest', 2800000, TRUE),
    ('custom', 5, 'Custom', NULL, 'custom', NULL, TRUE);
