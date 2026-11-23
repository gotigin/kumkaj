# Kirano Cloud billing — Paddle + local mock

**Chrome profile for Paddle:** **MEARAJ (Gotigin)**  
**Doctrine:** free local forever; unpaid cloud never hostages the till
([`docs/CLOUD_PLANS.md`](../../../docs/CLOUD_PLANS.md)).

**Stripe is not used.** India is not a supported Stripe Tax *business location*
(`stripe_tax_inactive`). Tax-at-checkout is via **Paddle**.

### Status

| Piece | State |
| --- | --- |
| Mock local smoke (`./scripts/local-smoke.sh`) | Green (mock-only) |
| `/account` plan checkout UI | Done — prefers Paddle when configured |
| Paddle sandbox Prices + keys + webhooks | Done (sandbox; Chrome MEARAJ/Gotigin) |
| Live mode | Only after owner approval |

## Modes — do not mix

| Mode | Auth | Cloud API | Paddle | Webhook destination |
| --- | --- | --- | --- | --- |
| **Local mock** | `DEV_AUTH_BYPASS` (API smoke only) | local + `BILLING_ALLOW_MOCK=1` | none | n/a (`sandbox/complete`) |
| **Local isolated E2E** | **Firebase Auth Emulator** | local + sandbox `PADDLE_*` | **sandbox** | **local tunnel** (`PADDLE_NTF_LOCAL_ID`) |
| **Deployed sandbox** | **Real** Firebase (kirano.shop) | AWS `api.kirano.shop` + sandbox keys | **sandbox** | `https://api.kirano.shop/...` (`PADDLE_NTF_DEPLOYED_ID`) |
| **Live** | Real Firebase | AWS + live keys | **live** | live notification URLs |

Never open the production account site against the Auth Emulator, and never point local UI at production Auth for isolated E2E (authorized-domain / mixing hell).

| `BILLING_MODE` | Meaning |
| --- | --- |
| `sandbox` (default) | Test money. Mock only if `BILLING_ALLOW_MOCK=1`. Paddle sandbox API when keys set. |
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

## Local mock (API only)

```bash
cp .env.example .env.development
# BILLING_ENABLED=1, BILLING_ALLOW_MOCK=1
./scripts/local-smoke.sh
```

## Local isolated E2E (site + Auth Emulator + Paddle sandbox)

```bash
./scripts/local-paddle-e2e.sh
```

Starts Auth Emulator (`:9099`), local API, local site with
`__KUMKAJ_AUTH_EMULATOR__` + `__KUMKAJ_CLOUD_API__`, ngrok on the **local**
notification destination only, and an HTTPS return URL for Paddle.
Open `http://127.0.0.1:8012/account`, use **email/password** (emulator), buy with
sandbox test cards below.

## Sandbox test cards

Paddle sandbox **rejects real cards**. Use only these
([Paddle sandbox docs](https://developer.paddle.com/sdks/sandbox#test-cards)).
Any cardholder name; expiry any **future** date; CVC any 3 digits.

| Card number | Result |
| --- | --- |
| `4242 4242 4242 4242` | Success (no 3DS) — default happy path |
| `4000 0038 0000 0446` | Success with 3DS challenge |
| `4000 0566 5566 5556` | Valid Visa debit |
| `4000 0000 0000 0002` | Declined |
| `4000 0027 6000 3184` | First payment succeeds; later renewals decline |

## Sandbox receipt / confirmation email

In **sandbox**, Paddle does **not** reliably deliver customer receipts to an
arbitrary personal address entered at checkout. Mail for non-account domains is
**forwarded to the Paddle account owner’s main email** (you). Live mode sends to
the customer email normally.

So seeing the receipt on the **owner** inbox while the buyer’s personal email
stays empty is expected sandbox behavior — not a broken webhook.

## Deployed sandbox

Real Firebase + AWS API + Paddle sandbox keys on ECS + notification
`https://api.kirano.shop/v1/billing/webhooks/paddle`. Not the local tunnel.

## Product confirmation email (SES)

Each product API sends **its own** plan-activation mail via Amazon SES after a
successful Paddle (or mock) activation. This is **not** a shared Kumkaj mailer.

| Env | Purpose |
| --- | --- |
| `MAIL_ENABLED=1` | Turn SES send on (AWS task defs have this) |
| `MAIL_FROM` | Product From (e.g. `noreply@khaynjo.com`) |
| `MAIL_PRODUCT_NAME` | Brand in subject/body |
| `MAIL_ACCOUNT_URL` | Link to `/account` |

Mail is **best-effort**: activation still succeeds if SES fails (logged).

SES identities are created in `ap-south-1`. Domains sit on **GoDaddy**
(`domaincontrol.com` NS) — add the three DKIM CNAMEs per domain (from
`aws sesv2 get-email-identity`), then wait for `VerifiedForSendingStatus=true`.
Until then, and while the account is in the SES sandbox, sends may fail until
(1) domain/email verified and (2) recipient verified **or** production access.

Click the SES verification mail for `mearaj@gotigin.com` to use that inbox as a
sandbox recipient during testing.

## Deployed sandbox AWS wiring

ECS task defs inject sandbox `PADDLE_API_KEY`, `PADDLE_CLIENT_TOKEN`, and the
**deployed** notification secret (`PADDLE_WEBHOOK_SECRET` for `api.*`) from SSM
SecureString params under `/{product}-cloud/…`. Task role
`kumkaj-cloud-ecs-task` may `ses:SendEmail`. `BILLING_ALLOW_MOCK=0` on AWS.

Sandbox Price ids ship in migration `0003_paddle_sandbox_prices.sql` (applied on
API boot). Live Price ids only after owner approval for live mode.

## Failure / cancel checks (overlay)

Paddle overlay events:

| Event | UI |
| --- | --- |
| `checkout.completed` | Refresh plan; success message |
| `checkout.error` / payment failed | Error; **plan unchanged** |
| `checkout.closed` without completed | “Checkout closed — no payment…”; **plan unchanged** |

Decline test card: `4000 0000 0000 0002`.

## Receipt / catalog naming (Company → Brand → Product → Plan)

Paddle customer receipts show the **seller company** (Gotigin Software & Hardware
Private Limited) plus the catalog **product name** and **price name**.

Sandbox catalog (active):

| Field | Value |
| --- | --- |
| Company | Gotigin Software & Hardware Private Limited (Paddle seller) |
| Brand | Kumkaj (first half of product name) |
| Product | Khaynjo / Moklanjo / Kirano / Tabiyat |
| Plan | Steady / Swift / Priority (price name) |

Product entity name format: `Kumkaj · {Product}`. Price name: `{Plan}`.
Do **not** use `Kumkaj Cloud — Steady` (that treated the brand as the product).

Each cloud API uses **its own** Price ids (migration `0005_paddle_product_prices.sql`).
