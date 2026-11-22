# Kumkaj Cloud plans (yearly)

**Status:** product commercial decision (2026-11-21). Same five plans for
all four Kumkaj products (Khaynjo / Moklanjo / Kirano / Tabiyat). Each
product has its own Auth, API, DB, and billing sandbox; list prices match.

Doctrine that still wins on conflict: free local forever; cloud optional;
billing must never hostage the till —
[`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
§20–22. Sync/Auth engineering notes:
[`CLOUD_AND_SYNC_DISCUSSION.md`](CLOUD_AND_SYNC_DISCUSSION.md).

---

## 1. Five plans

Every paid plan **includes** plan 1 (Free forever). Paid plans add live
cloud on top; they do not remove or shrink local / LAN capability.

| # | Plan | What the customer gets |
| --- | --- | --- |
| 1 | **Free forever** | Full local product + LAN Server/Client. No cloud fee. |
| 2 | **Steady** | Free forever **+** live cloud, **10 GB**, lower bandwidth cap |
| 3 | **Swift** | Free forever **+** live cloud, **25 GB**, mid bandwidth cap |
| 4 | **Priority** | Free forever **+** live cloud, **50 GB**, highest standard bandwidth cap |
| 5 | **Custom** | Free forever **+** cloud terms agreed with that client |

### What plans differ on

- **Data amount (quota):** 10 / 25 / 50 GB (Custom as agreed).
- **Bandwidth (sync speed):** lower / mid / highest standard (Custom as agreed).

### What plans do **not** differ on

- **Live cloud** — Steady / Swift / Priority are all **live** sync. Do not
  sell “nightly only” or restrict **connection interval** by plan.
- **Device / till count** — never a plan ceiling. Extra LAN tills stay free.
- **Local app features** — never gated by plan.

Anyone may **register / log in** (product Auth). Cloud features follow
the shop’s **subscribed** plan only.

---

## 2. Yearly price (INR list, **tax exclusive**)

List prices are the **raw** amount Kumkaj charges for the plan. They do
**not** include GST, VAT, sales tax, or any other local levy.

| Plan | Raw price / year (ex-tax) |
| --- | --- |
| Free forever | ₹0 |
| Steady | **₹6,000** |
| Swift | **₹12,000** |
| Priority | **₹24,000** |
| Custom | Quote — may be below Steady or above Priority (agreed terms) |

- **Custom** is sales-led: yearly amount, quota, and bandwidth are whatever
  Gotigin and the client agree. The quote **may be less than Steady** or
  **more than Priority** (or anywhere between). Still **tax-exclusive** raw
  list + tax at the gateway; still includes Free forever local. Custom is
  never a self-serve checkout SKU.

### Tax at the billing gateway (unified worldwide)

- Kumkaj / Khaynjo send the gateway the **raw** plan amount (+ customer
  location / tax settings as the gateway requires).
- The **gateway** (e.g. Stripe Tax or equivalent) computes and collects
  the correct tax for that buyer’s country. Do **not** hard-code
  country tax tables or “always 18% GST” into the product.
- Same five plans everywhere; only the invoice tax line changes by
  place of supply / registration. No India-only cloud edition.

**India example (18% GST), for clarity only — not a second price list:**

| Plan | Raw | Customer pays if GST 18% applies |
| --- | --- | --- |
| Steady | ₹6,000 | ₹7,080 |
| Swift | ₹12,000 | ₹14,160 |
| Priority | ₹24,000 | ₹28,320 |

Other currencies / display FX later; keep the **same plan shape** and the
same **ex-tax** principle. Do not invent India-only product scope — only
this INR raw list is locked for now.

Planning note (not customer copy): host COGS at full-quota use was
estimated in discussion; list prices assume typical mix plus scale. Revisit
from real AWS/GCP bills after metering ships. Firm tax treatment for
exports / foreign buyers is a Gotigin CA + gateway-config matter, not a
separate Khaynjo plan.

---

## 3. Cancellation and refund (commercial)

Prepaid cloud terms are not unwound mid-term as a normal path; next renewal is
always the customer’s choice; exceptions go to
[support@gotigin.com](mailto:support@gotigin.com). Full policy stack:

- Company: https://gotigin.com/cancellation  
- Brand: https://kumkaj.com/cancellation  
- Product: `/cancellation` on each product site (e.g. https://khaynjo.com/cancellation)

## 4. Subscription life, grace, deletion

Clock starts from **subscription end** (renewal moves the end date).

| Phase | Timing | Behaviour |
| --- | --- | --- |
| Paid year | **12 months** | Live cloud within plan quota + bandwidth |
| Grace | **1 month after** subscription ends | Internally still treat as covered cloud (**13 months** total run for a 12‑month sale) |
| Retrieve window | Through **2 months after** subscription ends | Local app unchanged; cloud may **reject new uploads**; existing cloud data remains **retrievable** |
| Delete eligible | **2 months after** subscription ends | Cloud data may be deleted |

### Notifications

- In the **2 months after** subscription ends (before delete): **weekly**
  automatic notices that data will be removed unless they renew / export.
- Those notices are **cancellable** in-product if they annoy the user.
- Still send a clear final warning before delete even if weeklies were muted
  (implementation detail; do not strand data with zero notice).

### Quota while subscribed

- User **cannot** raise the plan ceiling except by changing plan.
- User **can** clear cloud history for a chosen period to free **usage**
  inside the quota (destructive for that slice — say so in UI).

### Expiry and offline (non‑negotiable)

- Local / LAN continue **as if nothing happened**.
- Unpaid / expired / capped cloud never locks the till, never deletes
  local SQLite, never forces a worse “edition.”
- See vision §22.

---

## 5. Gate for other paid services

Future paid services (support packs, messaging add-ons, etc. — defined
separately) require the shop to be a member of **Steady, Swift, Priority,
or Custom**. Free forever alone does not unlock that catalog.

WhatsApp: if the shop uses **their own** WABA via Wadhu Connect, **Meta’s
message bill is theirs**, not Kumkaj Cloud plan COGS. Connect relay
hosting is separate platform cost.

---

## 6. Rollout

1. Documented here (2026-11-21).
2. Implement entitlements + metering on **Khaynjo** first.
3. Copy plan shape to other Kumkaj apps when their cloud ships.

**Billing (Khaynjo, 2026-11-22):** sandbox-first Stripe + Paddle + local
mock — see [`services/khaynjo-cloud/docs/BILLING_SANDBOX.md`](../services/khaynjo-cloud/docs/BILLING_SANDBOX.md).
Ops dashboards use Chrome profile **MEARAJ (Gotigin)**. Do not advertise
plans as live in store copy until Auth → connect → sync → **live** billing
can enforce them.
