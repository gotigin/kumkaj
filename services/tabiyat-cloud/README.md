# Tabiyat Cloud (API + DB)

**Own** AWS API service + **own** Postgres. Never shared with Moklanjo or other products.

Firebase Auth project: `tabiyat-shop` (same as tabiyat.com `/account`).

## Milestone 1

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| GET | `/healthz` | no | liveness |
| GET | `/v1/me` | Bearer Firebase ID token | ensure account (`account_id` + unique `username`) |
| GET | `/v1/plans` | Bearer | available plan catalog |
| GET | `/v1/me/subscription` | Bearer | current plan (defaults to Free forever) |

## Local

```bash
docker compose up -d
cp .env.example .env.development
# export vars from .env
cargo run
```

API default: `http://127.0.0.1:8080`.

## AWS (Gotigin account)

Two services per product:

1. **Database** — RDS Postgres (private VPC; not public).
2. **API** — App Runner or ECS/EC2; only the API may open the DB.

Deploy notes: [`scripts/deploy-aws.md`](scripts/deploy-aws.md).

Account for ops: AWS IAM user / console login tied to `mearaj@gotigin.com` (browser profile **MEARAJ (Gotigin)**). Configure CLI with that profile before deploy (`aws configure --profile gotigin`).
