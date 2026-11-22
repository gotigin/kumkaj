# Khaynjo Cloud billing — sandbox first

**Product:** Khaynjo only (copy to Moklanjo / Kirano / Tabiyat later).  
**Chrome profile for Stripe + Paddle dashboards:** **MEARAJ (Gotigin)**  
**Doctrine:** free local forever; unpaid cloud never hostages the till
([`docs/CLOUD_PLANS.md`](../../../docs/CLOUD_PLANS.md)).

Assume Stripe and Paddle are (or will be) approved. Until live keys exist,
everything runs in **sandbox / test mode**, with a local **mock** provider so
we can prove checkout → webhook/activation → subscription without waiting on
either gateway UI.

### Status (2026-11-22)

| Piece | State |
| --- | --- |
| API checkout + webhooks (`mock` / `stripe` / `paddle`) | **Done** in `services/khaynjo-cloud` |
| Mock local smoke (`./scripts/local-smoke.sh`) | **Green** |
| `/account` sandbox plan buttons | **Done** (needs cloud API + `BILLING_ENABLED=1`) |
| Stripe Test Price ids + keys | **Owner** — MEARAJ (Gotigin) Dashboard |
| Paddle sandbox Price ids + keys | **Owner** — MEARAJ (Gotigin) sandbox |
| Live mode / Firebase site redeploy | Only after owner approval |

**Next:** create sandbox catalog in Stripe + Paddle, put keys in gitignored
`.env`, run `./scripts/apply-sandbox-price-ids.sh`, then
`BILLING_DEFAULT_PROVIDER=stripe` (or `paddle`) and a test checkout.

---

## 1. Modes

| `BILLING_MODE` | Meaning |
| --- | --- |
| `sandbox` (default) | Test money only. Mock provider allowed. Stripe `sk_test_` / Paddle sandbox API. |
| `live` | Real money. Mock provider disabled. Requires live keys + live Price ids. |

| `BILLING_ENABLED` | Meaning |
| --- | --- |
| `0` / unset | Checkout routes refuse; Auth + plans still work. |
| `1` | Checkout + webhooks on. |

Default provider order when keys are missing: **mock**. When Stripe keys are
set, default becomes Stripe unless `BILLING_DEFAULT_PROVIDER` says otherwise.

---

## 2. API (Khaynjo cloud)

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| GET | `/v1/billing/config` | Bearer | mode, enabled providers, publishable / client tokens |
| POST | `/v1/billing/checkout` | Bearer | body `{ "plan_id": "steady", "provider": "mock"\|"stripe"\|"paddle" }` → checkout URL |
| POST | `/v1/billing/sandbox/complete` | Bearer | mock only: `{ "checkout_id": "…" }` activates the plan for 365 days |
| POST | `/v1/billing/webhooks/stripe` | Stripe-Signature | `checkout.session.completed` → activate |
| POST | `/v1/billing/webhooks/paddle` | Paddle-Signature | `transaction.completed` / `subscription.activated` → activate |
| GET | `/v1/me/subscription` | Bearer | current plan (Free forever until paid) |

Plans sold via self-serve: **steady / swift / priority**.  
`free` and `custom` are not checkout SKUs.

List prices are **tax exclusive** INR. Stripe Tax / Paddle tax adds the
buyer-country levy at the gateway.

---

## 3. Local proof (no Stripe/Paddle keys)

```bash
cd services/khaynjo-cloud
docker compose up -d
cp .env.example .env   # BILLING_ENABLED=1, BILLING_DEFAULT_PROVIDER=mock
./scripts/local-smoke.sh
```

Smoke covers: healthz → ensure account → plans → mock checkout → sandbox
complete → subscription is Steady with `ends_at` ~1 year out.

---

## 4. Stripe sandbox (MEARAJ (Gotigin) profile)

1. Open Chrome as **MEARAJ (Gotigin)** → [Stripe Dashboard](https://dashboard.stripe.com) → **Test mode**.
2. Products → create yearly Prices (INR) for Steady / Swift / Priority matching
   raw list ₹6,000 / ₹12,000 / ₹24,000 (ex-tax). Enable **Stripe Tax** in test.
3. Developers → API keys → copy `sk_test_…`, `pk_test_…`.
4. Developers → Webhooks → endpoint  
   `https://<khaynjo-api>/v1/billing/webhooks/stripe`  
   (local: `stripe listen --forward-to localhost:8080/v1/billing/webhooks/stripe`)  
   Events: `checkout.session.completed` (add invoice/subscription events later).
5. Put keys in `.env` / AWS secrets. Set on Postgres:

```sql
UPDATE plans SET stripe_price_id = 'price_…' WHERE plan_id = 'steady';
-- same for swift / priority
```

6. `BILLING_DEFAULT_PROVIDER=stripe` and run a test card checkout
   (`4242…`). Confirm `/v1/me/subscription` flips.

---

## 5. Paddle sandbox (same Chrome profile)

1. Open Chrome as **MEARAJ (Gotigin)** → [Paddle sandbox](https://sandbox-vendors.paddle.com).
2. Create catalog Prices (`pri_…`) for the three yearly plans (tax exclusive).
3. Authentication → API key (`…_sdbx…`) + client-side token (`test_…`).
4. Notifications → destination  
   `https://<khaynjo-api>/v1/billing/webhooks/paddle`  
   Subscribe to `transaction.completed`, `subscription.activated`.
5. Set `paddle_price_id` on `plans` rows; put keys in env;  
   `BILLING_DEFAULT_PROVIDER=paddle`.

Paddle.js overlay can be wired on `/account` later using
`paddle_client_token` from `/v1/billing/config`. Until then the API returns
the transaction checkout URL from the Transactions API.

---

## 6. Production flip (after both approvals)

1. Create **live** Prices in Stripe live + Paddle live (separate ids).
2. Store live secrets in AWS (never in git).  
3. Set `BILLING_MODE=live`, `BILLING_ENABLED=1`, remove mock from any public UI.
4. Point webhooks at the HTTPS API host (`api.khaynjo.com` once DNS exists).
5. Re-run smoke against staging with test→live checklist; only then flip DNS.

---

## 7. What is intentionally not here yet

- Customer portal / cancel / change plan UI (API can extend).
- Metering bandwidth/quota enforcement.
- In-app (Makepad) pay sheet — website `/account` first.
- Moklanjo / Kirano / Tabiyat copies of this service.
