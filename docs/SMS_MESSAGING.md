# SMS messaging — Local gateway and Cloud service

Kumkaj apps (Khaynjo, Moklanjo, future products) already **queue** SMS
intents locally (`notify_outbox` / CRM campaigns / bill share). Delivery
is planned in two complementary ways. Neither path gates the counter or
books: the shop keeps working with messages waiting in the outbox.

**Status (2026-10-28):** queue + CRM segments + one-tap WhatsApp
(`wa.me`) / Copy SMS are **Have** in the apps. The Android Local SMS
Gateway sibling and the paid Cloud SMS Service are **Planned** — not
shipped yet.

------------------------------------------------------------------------

## 1. Local SMS Gateway (cloudless) — Planned

A **sibling Android app** that turns the business’s own phone and SIM
into a small SMS gateway for the shop (and, later, for other Kumkaj
products and willing third-party tools).

| | |
|---|---|
| **Idea** | Use the business’s Android phone and SIM. |
| **Link to desktop / Server** | Connects over **LAN** or **Bluetooth** — no Kumkaj Cloud subscription. |
| **Who pays for SMS** | The business pays its **mobile operator** (prepaid/postpaid SIM usage). |
| **Sending** | Automatic drain of the shop’s outbox and manual send, subject to **Android permissions and platform restrictions** (background limits, SMS rate limits, user consent). |
| **Scope** | Useful to Khaynjo (bills, birthday/loyalty CRM), Moklanjo (booked / OFD / delivered), future Kumkaj apps, and potentially third parties that speak the same local outbox / gateway protocol. |

### Why this shape

- Matches local-first doctrine (vision §20–21): messaging must not force
  Cloud.
- Reuses hardware the shop already has (an Android handset on the
  counter or in the office).
- Keeps data and send decisions on the owner’s devices; the gateway only
  forwards what the shop already queued.

### Non-goals for the local gateway

- Not a hosted blast/marketing SaaS.
- Not a substitute for WhatsApp Business API automation.
- Does not require internet to the Kumkaj Cloud; LAN/Bluetooth to the
  shop Server (or paired desktop) is enough for drain.

------------------------------------------------------------------------

## 2. Cloud SMS Service (paid) — Planned

An optional **Kumkaj Cloud** messaging path for shops that do not want
to leave a phone plugged in, or that need reachability without a local
gateway.

| | |
|---|---|
| **Idea** | Send through Kumkaj’s cloud service and an SMS provider. |
| **Device** | **No Android phone** needs to stay connected. |
| **Clients** | Desktop installations and other supported clients that can reach Kumkaj Cloud. |
| **Commercial** | Charge **per SMS**, offer **prepaid credits**, or **bundle** usage into a paid plan — same usage-based spirit as other Cloud metering (vision §21). |
| **Requirements** | Internet access and compliance with applicable messaging regulations (opt-in, TRAI/DND where relevant, content rules). |

Cloud SMS enhances delivery; it must never hold local books or the
outbox hostage if Cloud billing stops (vision §22). Unsent intents can
still sit locally or switch to the Local SMS Gateway when one is paired.

------------------------------------------------------------------------

## How apps use this today

| Layer | Today | Later |
|---|---|---|
| Compose / queue | **Have** — bill SMS, CRM segment campaigns, order alerts → `notify_outbox` | Same |
| One-tap WhatsApp | **Have** — `wa.me` on the device (Net optional on that phone) | Same |
| Local SIM drain | Stub / Copy SMS; **Planned** Local SMS Gateway app | Pair gateway → automatic + manual send |
| Hosted send | Not built | **Planned** Cloud SMS Service |

App-specific UI:

- Khaynjo: Parties → Guests & messages; Sales share — see [`KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md).
- Moklanjo: notify outbox drain — see [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md), [`COURIER_RESEARCH.md`](../apps/moklanjo/docs/COURIER_RESEARCH.md).

------------------------------------------------------------------------

## Product principle

Prefer the **Local SMS Gateway** for cloudless shops. Offer **Cloud SMS**
as a paid convenience. Always keep queue-first behaviour so a missing
gateway, a denied permission, or an unpaid Cloud account never stops
selling, cooking, or booking.
