# Khaynjo Cloud (API + DB)

**Own** AWS API service + **own** Postgres. Never shared with Moklanjo or other products.

Firebase Auth project: `khaynjo-2b2a2` (same as khaynjo.com `/account`).

Stripe + Paddle dashboards: Chrome profile **MEARAJ (Gotigin)**.  
Sandbox billing guide: [`docs/BILLING_SANDBOX.md`](docs/BILLING_SANDBOX.md).

## Milestone 1 — Auth + plans

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| GET | `/healthz` | no | liveness |
| GET | `/v1/me` | Bearer Firebase ID token | ensure account (`account_id` + unique `username`) |
| GET | `/v1/plans` | Bearer | available plan catalog |
| GET | `/v1/me/subscription` | Bearer | current plan (defaults to Free forever) |

## Milestone 2 — Billing (sandbox first)

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| GET | `/v1/billing/config` | Bearer | sandbox/live + enabled providers |
| POST | `/v1/billing/checkout` | Bearer | start mock / Stripe / Paddle checkout |
| POST | `/v1/billing/sandbox/complete` | Bearer | finish **mock** checkout locally |
| POST | `/v1/billing/webhooks/stripe` | Stripe-Signature | activate after test/live pay |
| POST | `/v1/billing/webhooks/paddle` | Paddle-Signature | activate after sandbox/live pay |

Default local path uses the **mock** provider (`BILLING_DEFAULT_PROVIDER=mock`) so
checkout → subscription can be proven without gateway keys. Wire Stripe test /
Paddle sandbox keys when ready (same Chrome profile).

## Local

```bash
docker compose up -d
cp .env.example .env.development
# BILLING_ENABLED=1 already in the example
./scripts/local-smoke.sh
# or: set -a && source .env.development && set +a && cargo run
```

API default: `http://127.0.0.1:8080`.

## AWS (Gotigin account)

Two services per product:

1. **Database** — RDS Postgres (private VPC; not public).
2. **API** — App Runner or ECS/EC2; only the API may open the DB.

Deploy notes: [`scripts/deploy-aws.md`](scripts/deploy-aws.md).

Account for ops: AWS IAM / console via Chrome profile **MEARAJ (Gotigin)**.
Configure CLI with that profile before deploy (`aws configure --profile gotigin`).
