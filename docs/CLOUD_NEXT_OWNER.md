# Cloud next session (owner)

## Done 2026-11-23

- HTTPS APIs: `https://api.khaynjo.com` / `api.moklanjo.com` / `api.kirano.shop` / `api.tabiyat.shop`.
- Sites + apps use those HTTPS bases; Firebase Hosting redeployed.
- **Stripe discarded** for Gotigin cloud billing: India is not a supported *business location* for Stripe Tax (`stripe_tax_inactive`). Code/routes/env no longer offer Stripe.
- Billing providers now: **mock** (local `BILLING_ALLOW_MOCK=1` only) + **Paddle** (next).
- Local mock smoke still: `./scripts/smoke-all-clouds.sh` (labeled mock-only).
- Testing rules: [TESTING.md](TESTING.md).

## Next — Paddle sandbox

On Paddle vendors UI you are on Get started (Live). Use the page’s **Switch to Sandbox** / **Integrate in sandbox** (labels on that screen) before wiring keys.

1. Sandbox catalog: Steady / Swift / Priority yearly INR ex-tax ₹6k / ₹14k / ₹28k.
2. Sandbox API key + client token + four notification URLs → `/v1/billing/webhooks/paddle`.
3. Put secrets in each `services/*-cloud/.env.development` (gitignored); apply `paddle_price_id` via `apply-sandbox-price-ids.sh`.
4. Browser E2E per product; then AWS sandbox env (no live flip until you approve).

## Blocked

- `BILLING_MODE=live` until you explicitly approve.
