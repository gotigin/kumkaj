# Cloud, sync, Auth — discussion notes (continue here)

**Status:** discussion / working notes (2026-11-20). Not a finished
implementation plan. Captured so we do not re-argue the same points.

Canonical product doctrine still wins on conflict:
[`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
§20–22 (optional cloud, free local forever) and
[`CLOUD_PLANS.md`](CLOUD_PLANS.md) (yearly Free / Steady / Swift /
Priority / Custom).
Website Auth how-to: [`FIREBASE_AUTH_SETUP.md`](FIREBASE_AUTH_SETUP.md).

---

## 1. What is already decided (product shape)

### Auth (person)

- **Global** products. Do not narrow geography or invent regional-only
  billing/product scope.
- Auth is **per product domain**, not brand-level Gotigin/Kumkaj login.
  (Unlike Wadhu: Wadhu’s apps do **not** each have their own domain;
  Auth lives on the Wadhu site. Kumkaj products each have their own.)
- Website Auth (email, Google, Phone, GitHub, Microsoft; Apple postponed)
  is enabled per product Firebase project. App login with the **same**
  product Auth is next — not done yet.
- Auth = who the person is. Cloud books sync = a separate concern after
  login / connect.

### Offline LAN (non‑negotiable)

- Free **Server / Client** on the local network forever.
- Subscription lapse, payment failure, sync errors, or cloud outage must
  **never** stop or bug the offline till. Cloud features pause; local
  SQLite + LAN pairing keep working as if nothing happened.
- Duress / emergency unlock stays offline-LAN-only (not a cloud feature).

### One logical shop database + optional sync

- The offline Server’s **SQLite is the shop’s database**.
- Cloud does **not** invent a second independent “cloud shop” the user
  lives in. When they connect, they use the **same** books identity:
  local replica + cloud replica of that same DB.
- **Sync is one cloud service** the user can enable/disable. Not the
  existence of the database.
- **Local → cloud:** login/connect turns sync on; same DB, now also
  replicated. Seamless.
- **Unsubscribe / disable sync / “back to local”:** same SQLite stays on
  the Server. No migrate-to-a-different-product-DB. Till keeps the same
  data.
- Still two *places* bytes live (device + cloud) when sync is on; one
  *logical* shop DB.

---

## 2. Sync as a service — scenarios to handle carefully

Sync must never silently destroy cloud history with an empty new Server.

| Local | Cloud | Safe default |
| --- | --- | --- |
| Empty / new Server | Has data | **Restore cloud → local** (or explicit confirm). Never auto-upload empty local. |
| Has data | Empty | **Seed local → cloud**, then sync. |
| Both meaningful | Both meaningful | **Ask** (restore / keep local & replace cloud / cancel). No silent overwrite. |
| Sync off / plan ended | — | Local only; cloud replica frozen until they reconnect with a conflict check if both changed. |

Other cases to design for:

- Uninstall → reinstall → new Server → login: restore from cloud, don’t
  create a competing empty shop and push.
- Offline while sync was on: queue local writes; sync on reconnect.
- Partial sync / crash mid-restore: resume; don’t mark healthy until
  verified.
- User is unaware local could overwrite cloud → product must **prevent**
  that by default (orientation step on first connect), not only warn
  after the fact.

---

## 3. Cloud hosting shape (discussion preference)

Two services (roles):

1. **API / app server** — talks to clients, checks Auth tokens, runs
   sync protocol, reads/writes the cloud DB.
2. **Database server** — holds cloud-side replica / sync metadata.

**Preference:** put the **database on its own host from the beginning**
(not API+DB packed on one VM). Keeps the DB neat; API talks to it in a
controlled way.

**Obvious rule — plan assumes it:** clients (website, app, Server/Client
tills) **never** talk to the cloud database directly. Only the **API
server** opens DB connections. Users and devices use HTTPS (or equivalent)
to the API; credentials, quotas, sync rules, and metering all sit there.
DB host stays private (VPC / security groups; no public DB for end users).

Local till SQLite stays on the device — never “the only copy lives only
on cloud.”

### Provider split (working decision, 2026-11-20)

| Piece | Direction |
| --- | --- |
| **Auth** | **Firebase** for almost all products (already per product site). |
| **DB (+ sync API for now)** | **Not locked** forever to GCP or AWS. **Near term:** use **AWS** (Activate credits cover RDS + compute such as EC2/App Runner/ECS, etc.) so we can build the stack, **see real expense patterns**, and learn how to charge users fairly. |
| **GCP** | **Not ignored.** Keep using it (Firebase Auth/Hosting today; deploy apps / other workloads there as needed). |

**Per product on AWS:** each product has its **own** API service and **own**
RDS — never one shared cloud for Khaynjo + Moklanjo. Ops account:
`mearaj@gotigin.com` (Gotigin / Activate). Local scaffolds:
`services/khaynjo-cloud/`, `services/moklanjo-cloud/`.

Path of a cloud user: **app → API → database**. Some users will burn more
bandwidth / data / ops than others. Metering and isolation must reflect
that (quotas, usage lines, fair scheduling) so one heavy user does not
trample others — without inventing multi-shop product chrome.

Commercial frame (vision §21 + [`CLOUD_PLANS.md`](CLOUD_PLANS.md)): free
local forever; yearly cloud membership **Steady / Swift / Priority /
Custom** (quota + live bandwidth); list prices **tax-exclusive**, tax at
the billing gateway by country. Other paid services require one of those
paid plans. Billing gateway under watch includes Stripe (Tax); alternatives
OK if better globally. Do not lock geography.

AWS credits = **delay the bill** while we learn cost → price. After
credits, revisit AWS vs GCP (or split) for API/DB on **numbers**, not
loyalty. Auth can stay on Firebase either way.

---

## 4. Explicitly out of this discussion (do not re-inject)

- India-only or region-locked product/billing assumptions.
- “Multi-shop” UI / org trees as the default mental model for v1 connect.
- Claiming Firebase (or any one vendor) is the only possible cloud
  **database/API** backend — Auth on Firebase is fine; DB/API vendor is
  open.
- Making cloud required for normal till use.
- Two independent unrelated databases (local shop vs separate cloud shop)
  as the long-term story — superseded by **one logical DB + sync service**.
- Treating “AWS vs GCP” as migrating whole Kumkaj products; only the
  optional sync API+DB (and where we choose to deploy other apps) moves.

---

## 5. Rollout order and non‑negotiables (2026-11-20; updated 2026-11-21)

- **Never share cloud stacks across products.** Each of Khaynjo, Moklanjo,
  Kirano, and Tabiyat owns a separate **API service** + separate
  **database**. Copy-then-diverge only — no shared cloud crate, no shared
  user table, no one API more than one app calls.
- **Milestone 1 (API+DB + Auth clients 2026-11-21):** per-product AWS API
  + RDS; Lock-screen Firebase email Continue + forgot + website federated
  link; site `/account` calls `/v1/me` + subscription after sign-in;
  ensure cloud account (`account_id` + unique `username`);
  `GET /v1/plans` + `GET /v1/me/subscription`. **Not yet:** HTTPS/DNS for
  stable `api.<domain>` (site HTTPS→HTTP API is mixed-content blocked until
  then — native apps OK on HTTP), in-app OAuth/phone, shop connect,
  restore/seed, `sync_outbox` drain. **Billing sandbox (2026-11-22):**
  **All four** product clouds have checkout + webhooks for
  **mock / Stripe / Paddle** (`services/*-cloud/docs/BILLING_SANDBOX.md`);
  site `/account` plan buttons + app `cloud_api` purchase helpers.
  Chrome profile **MEARAJ (Gotigin)**. Local mock smoke proves activate →
  subscription on each product; Stripe/Paddle test keys + Price ids still
  to be pasted per product when dashboards are ready.
- Code lives under `services/{khaynjo,moklanjo,kirano,tabiyat}-cloud/`
  (standalone Cargo projects — not Makepad app members).
- **Grow and scale** without redesigning from scratch: API and DB as
  separate hosts; clear contracts; metering from day one so heavy users
  don’t define the architecture later.
- **User data never corrupted** by our silly changes: migrations must be
  additive/safe; sync must be idempotent; empty local must never wipe
  cloud; restore/seed orientation before blind push; no destructive
  schema experiments on live books.
- **API / DB compatibility:** path version (`/v1`); additive cloud schema
  migrations (`migrate_N_to_N+1`); never reuse a version; older clients keep
  working when fields are only added.
- **Future‑proof identity (hooks only):** each product account has
  `account_id` (UUID), product-scoped `username`, `firebase_uid`, and an
  empty `account_external_links` table for a later Kumkaj-brand /
  Gotigin graph. Do **not** merge Auth or DBs across products now.
- **Future‑proof product:** one logical shop DB + optional sync service;
  offline LAN always works if plan/cloud dies; Auth stays on Firebase; AWS
  for near-term API+DB learning costs; GCP remains in play for
  Auth/Hosting. Don’t invent a second unrelated “cloud shop” database.

## 6. Cloud plans (decided 2026-11-21)

Canonical commercial grid: [`CLOUD_PLANS.md`](CLOUD_PLANS.md).

Summary: **Free forever** + **Steady (10 GB)** + **Swift (25 GB)** +
**Priority (50 GB)** + **Custom**. Paid plans include free forever. All
paid tiers are **live** sync; they differ by **quota** and **bandwidth**
only (no device caps, no connection-interval tiers). Yearly INR **raw
(ex-tax)** self-serve list: ₹6,000 / ₹12,000 / ₹24,000; **Custom** is a
negotiated quote that **may be below Steady or above Priority**. Tax
(GST/VAT/etc.) added by the **billing gateway** per buyer country. After
subscription ends: 1 month grace (13 months cloud run), retrieve until
+2 months, then delete eligible; weekly cancellable notices in that window.

Plan catalog is **seeded in each product’s cloud DB** and returned by the
API — apps display what the API returns (so copy can change without an
app-store release when possible). Purchase / subscribe mutations come
after billing is wired.

## 7. Open for the next discussion

- Concrete bandwidth cap numbers (Steady / Swift / Priority).
- **AWS v1 live (2026-11-21):** all four products — private RDS + ECS
  Fargate API in `ap-south-1` (Gotigin `mearaj@gotigin.com`). App Runner
  create was blocked on the account — Fargate used instead. Local smoke
  must pass before recreate. Task public IPs change on redeploy — add
  DNS/ALB later. Ops ids: gitignored `local/aws-gotigin/ap-south-1.env`.
- **Next ops:** HTTPS + DNS (`api.<product-domain>`) so site `/account`
  cloud sync is not mixed-content blocked; ALB or equivalent. In-app
  Google/phone later. Keep local PIN / staff username unlock for offline
  till.
- Billing: finish Stripe test + Paddle sandbox catalog (Price ids on
  `plans`), then live flip per BILLING_SANDBOX.md. Both gateways stay
  supported; mock is sandbox-only.
- Other paid services that require Steady+ membership.
- Later (not now): Kumkaj brand “my products” / Gotigin brand tree via
  `account_external_links` — still separate product Auth + DBs.

When continuing, start from [`CLOUD_PLANS.md`](CLOUD_PLANS.md), this file,
and vision §20–22.
