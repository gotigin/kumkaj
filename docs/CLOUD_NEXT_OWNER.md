# Cloud next session (owner)

After solo shop/account/billing scaffolding: pick up here.

## Pending API + DB (all four products)

- ~~HTTPS + DNS: `api.<product-domain>` (ALB)~~ **Done 2026-11-23** — `https://api.khaynjo.com` / `api.moklanjo.com` / `api.kirano.shop` / `api.tabiyat.shop` (shared ALB `kumkaj-cloud-apis`, ACM ISSUED). Redeploy product Hosting for `/account` JS when ready.
- ~~Stable base URL~~ — sites + app `DEFAULT_CLOUD_API_BASE` point at those HTTPS hosts (commit + Firebase deploy still needed for live sites).
- Deploy billing migration `0002` + `BILLING_ENABLED=1` on each AWS API (sandbox first).
- Confirm plans seed + Custom quote path (no self-serve checkout).

## Stripe + Paddle full test (sandbox)

Chrome profile **MEARAJ (Gotigin)**.

1. Create yearly INR Prices (ex-tax ₹6k / ₹12k / ₹24k) per product (or shared catalog with separate webhook endpoints).
2. Put test keys in each `services/*-cloud/.env` (gitignored).
3. `./scripts/apply-sandbox-price-ids.sh` per product.
4. `stripe listen` / Paddle sandbox notifications → `/v1/billing/webhooks/{stripe|paddle}`.
5. Prove: site `/shop` → cart → `/account?checkout=` → gateway → `/v1/me/subscription`.
6. Prove: app lock → Cloud plans/shop → same account → subscription refresh after return.

Mock path already: `./scripts/smoke-all-clouds.sh`.

## Production (Paddle first)

Only after sandbox E2E green and owner approval:

- Live Paddle prices + keys; `BILLING_MODE=live`; mock disabled.
- Firebase Hosting redeploy for `/shop` + `/account` (requires `--i-approve-firebase-deploy`).
- Flip AWS secrets; monitor webhooks.

Do **not** ship live Stripe/Paddle without that approval.
