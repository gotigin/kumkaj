# Deploy Moklanjo Cloud on AWS

Ops login: Gotigin AWS account (`mearaj@gotigin.com`, browser profile **MEARAJ (Gotigin)**).

```bash
aws configure --profile gotigin
# region suggestion: ap-south-1 (Mumbai) unless you already standardized elsewhere
export AWS_PROFILE=gotigin
```

## Two services (required)

| Role | AWS | Notes |
| --- | --- | --- |
| **Database** | RDS PostgreSQL 16 (`db.t4g.micro`) | **Not** publicly accessible; SG allows 5432 only from this product’s API SG |
| **API** | **ECS Fargate** (App Runner blocked on this account as of 2026-11-21) | Env: `DATABASE_URL` from Secrets Manager, `FIREBASE_PROJECT_ID=moklanjo-com`. **No** `DEV_AUTH_BYPASS`. |

Clients (apps / site) call **only** the API. Never expose RDS publicly.

Region: `ap-south-1`. Separate ECS cluster / service / task def / RDS / secrets from Khaynjo.

## Gate before any paid AWS (RDS / App Runner)

```bash
./scripts/local-smoke.sh   # Docker Postgres only — must PASS
```

Do **not** create RDS or App Runner until that smoke is green. Never set
`DEV_AUTH_BYPASS` on AWS.

## First bring-up (checklist)

1. Pass `./scripts/local-smoke.sh`.
2. VPC with private subnets for RDS; API in same VPC (or VPC connector for App Runner).
3. Create RDS instance `moklanjo-cloud` (db name `moklanjo_cloud`). Store password in Secrets Manager.
4. Build/push container from this directory’s `Dockerfile`.
5. Deploy API with secrets injected (**no** `DEV_AUTH_BYPASS`); confirm `GET /healthz`.
6. Migrations run automatically on API boot (`sqlx::migrate!`).
7. Point a hostname later (e.g. `api.moklanjo.com`) at the API — optional for first smoke.

Activate credits cover RDS + App Runner/EC2 while learning cost → price.

**Do not** put Moklanjo data in this RDS or this API.
