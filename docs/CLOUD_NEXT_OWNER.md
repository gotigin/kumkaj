# Cloud next session (owner)

After solo shop/account/billing scaffolding: pick up here.

## Done 2026-11-23 (sandbox)

- HTTPS APIs live: `https://api.khaynjo.com` / `api.moklanjo.com` / `api.kirano.shop` / `api.tabiyat.shop`.
- Sites + app `DEFAULT_CLOUD_API_BASE` → HTTPS; Firebase Hosting redeployed (all four).
- Stripe Test: restricted `rk_test_` key, 3 yearly Prices, 4 webhook endpoints + `whsec_` in each `services/*-cloud/.env.development`.
- AWS ECS: `BILLING_ENABLED=1`, `BILLING_MODE=sandbox`, Test Stripe keys on all four APIs (healthz reports sandbox).
- Migration `0003_sandbox_stripe_prices.sql` sets Stripe Test price ids on `plans`.
- Local mock smoke: `./scripts/smoke-all-clouds.sh` **PASSED** (4/4).
- Stripe Checkout keeps `automatic_tax` + address/tax-id collection (CLOUD_PLANS intent). **Verified fact (Stripe docs + API):** for **India (`IN`)**, Stripe Tax lists **customer location ✓** but **business location ❌ Not supported**. Gotigin is India-based → API returns `stripe_tax_inactive` with either `rk_test_` or `sk_test_`. This is not a restricted-key issue and not fixed by toggling Tax UI. Tax-at-gateway for us needs a path Stripe Tax supports for our merchant country (e.g. Paddle MoR) — do not disable tax to green a smoke. See [TESTING.md](TESTING.md) and [Stripe supported countries](https://docs.stripe.com/tax/supported-countries).
- Env split: use `.env.development` (Test) vs `.env.production` placeholder; live keys stay out of local files until approved.

- Mock unpaid activate requires `BILLING_ALLOW_MOCK=1` (local smoke only); not set on AWS.

## Still open

- Browser E2E: `/shop` → Stripe test card → `/v1/me/subscription` (and app lock path) per product.
- Paddle sandbox Prices + API keys + 4 notification URLs (not wired yet).
- Move AWS Stripe keys from task env into Secrets Manager when that API is reliable.
- Production flip (Paddle-first live) — **blocked** until you explicitly approve. Do **not** set `BILLING_MODE=live`.
