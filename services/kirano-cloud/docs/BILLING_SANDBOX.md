# Kirano Cloud billing — Paddle + local mock

**Chrome profile for Paddle:** **MEARAJ (Gotigin)**  
**Doctrine:** free local forever; unpaid cloud never hostages the till
([`docs/CLOUD_PLANS.md`](../../../docs/CLOUD_PLANS.md)).

**Stripe is not used.** India is not a supported Stripe Tax *business location*
(`stripe_tax_inactive`). Tax-at-checkout is via **Paddle**.

Until Paddle sandbox keys exist, local **mock** (`BILLING_ALLOW_MOCK=1`) proves
checkout → activation → subscription without a gateway.

### Status

| Piece | State |
| --- | --- |
| Mock local smoke (`./scripts/local-smoke.sh`) | Green (mock-only) |
| `/account` plan checkout UI | Done — prefers Paddle when configured |
| Paddle sandbox Prices + keys + webhooks | **Next** — owner + agent |
| Live mode | Only after owner approval |

## Modes

| `BILLING_MODE` | Meaning |
| --- | --- |
| `sandbox` (default) | Test only. Mock only if `BILLING_ALLOW_MOCK=1`. Paddle sandbox API when keys set. |
| `live` | Real money. Mock disabled. Paddle live keys + live Price ids. |

| `BILLING_ENABLED` | Meaning |
| --- | --- |
| `0` / unset | Checkout refuses; Auth + plans still work. |
| `1` | Checkout + Paddle webhook on. |

## API

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| GET | `/v1/billing/config` | Bearer | mode, providers, Paddle client token |
| POST | `/v1/billing/checkout` | Bearer | `{ "plan_id", "provider": "mock"|"paddle" }` |
| POST | `/v1/billing/sandbox/complete` | Bearer | mock only + `BILLING_ALLOW_MOCK=1` |
| POST | `/v1/billing/webhooks/paddle` | Paddle-Signature | activate paid plan |
| GET | `/v1/me/subscription` | Bearer | current plan |

Prices are **tax exclusive** INR; Paddle adds tax at checkout.

## Local

```bash
cp .env.example .env.development
# BILLING_ENABLED=1, BILLING_ALLOW_MOCK=1 for mock smoke
./scripts/local-smoke.sh
```

## Paddle sandbox (next)

1. Switch to Sandbox in Paddle vendors UI (label on Get started screen).
2. Create yearly Prices for Steady / Swift / Priority (₹6k / ₹12k / ₹24k ex-tax).
3. Set `PADDLE_*` in `.env.development`; notification URL
   `https://api.<domain>/v1/billing/webhooks/paddle`.
4. `./scripts/apply-sandbox-price-ids.sh` with `PADDLE_PRICE_*` env vars.
