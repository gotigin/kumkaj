# SMS messaging — delivery approaches (planned)

Kumkaj apps (Khaynjo, Moklanjo, future products) already **queue** SMS
intents locally (`notify_outbox` / CRM campaigns / bill share). Delivery
is planned across several complementary approaches below. None of them
gates the counter or books: the shop keeps working with messages waiting
in the outbox.

**Status (2026-11-05):** queue + CRM segments + one-tap WhatsApp
(`wa.me`) / Copy SMS are **Have** in the apps. Every send path below is
**Planned** — not shipped yet.

Group by whether the shop needs the **public internet / a hosted SMS
API** to actually place the message on the mobile network.

------------------------------------------------------------------------

## Offline / local (shop SIM or attached radio — no internet to send)

These paths use a **physical SIM (or multi-SIM radio)** the business
controls. The shop pays its **mobile operator**. The Kumkaj Server (or
paired desktop) can hand the outbox over **LAN / USB / Bluetooth** —
internet to Kumkaj Cloud is **not** required to drain SMS.

| Approach | Idea | Notes |
|----------|------|-------|
| **Android phone with a SIM** | A **sibling Android app** turns the business’s own handset into a small Local SMS Gateway for the shop (and later other Kumkaj products / willing third-party tools). | Connect over **LAN or Bluetooth**. Subject to Android SMS permissions, background limits, and rate / consent rules. Reuses hardware many shops already keep on the counter. |
| **USB GSM/LTE modem with a SIM** | A USB stick / dongle modem on the shop PC or Server, with one SIM, drains the same outbox. | Useful when no dedicated Android phone should stay plugged in. Needs a supported modem, drivers, and OS permission to send SMS. Still operator-billed on that SIM. |
| **Supported multi-SIM modem or SMS device** | A dedicated multi-SIM GSM/LTE modem, SMS appliance, or similar supported device attached locally. | Higher volume / failover across SIMs without a handset UI. Same local-first rule: outbox stays on the shop; the device only forwards what was already queued. Only **supported** models — not every USB gadget. |

### Why local paths first

- Matches local-first doctrine (vision §20–21): messaging must not force
  Cloud.
- Keeps data and send decisions on the owner’s devices.
- Works in shops that already pay for prepaid/postpaid SIMs and may have
  weak or no broadband at send time.

### Non-goals for local paths

- Not a hosted blast / marketing SaaS.
- Not a substitute for WhatsApp Business API automation.
- Do not require internet to Kumkaj Cloud; LAN / USB / Bluetooth to the
  shop Server (or paired desktop) is enough for drain.

------------------------------------------------------------------------

## Online / internet (hosted gateway)

These paths need **network reach to a provider API** (and usually
Kumkaj Cloud for billing / routing). No shop SIM needs to stay connected
for send.

| Approach | Idea | Notes |
|----------|------|-------|
| **Third-party SMS gateway provider** | Optional **Kumkaj Cloud SMS Service**: send through Kumkaj Cloud and a contracted SMS provider (or a shop-chosen supported gateway where that product shape exists). | **No Android phone / modem** needs to stay connected. Desktop and other clients that can reach Cloud can send. Commercial shape: **per SMS**, **prepaid credits**, or **plan bundle** — same usage-based spirit as other Cloud metering (vision §21). Requires internet and compliance with messaging regulations (opt-in, TRAI/DND where relevant, content rules). |

Cloud SMS enhances delivery; it must never hold local books or the
outbox hostage if Cloud billing stops (vision §22). Unsent intents can
still sit locally or switch to a local SIM / modem path when one is
paired.

------------------------------------------------------------------------

## How apps use this today

| Layer | Today | Later |
|---|---|---|
| Compose / queue | **Have** — bill SMS, CRM segment campaigns, order alerts → `notify_outbox` | Same |
| One-tap WhatsApp | **Have** — `wa.me` on the device (Net optional on that phone) | Same |
| Local SIM / modem drain | Stub / Copy SMS | **Planned** — Android phone gateway, USB GSM/LTE modem, or supported multi-SIM / SMS device |
| Hosted send | Not built | **Planned** — third-party SMS gateway via Cloud SMS Service |

App-specific UI:

- Khaynjo: Parties → Guests & messages; Sales share — see [`KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md).
- Moklanjo: notify outbox drain — see [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md), [`COURIER_RESEARCH.md`](../apps/moklanjo/docs/COURIER_RESEARCH.md).

------------------------------------------------------------------------

## Product principle

Prefer a **local SIM / modem** path for cloudless shops. Offer **Cloud /
third-party gateway SMS** as a paid convenience. Always keep queue-first
behaviour so a missing gateway, a denied permission, an unsupported
modem, or an unpaid Cloud account never stops selling, cooking, or
booking.
