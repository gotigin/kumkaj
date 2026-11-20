# Client templates, shop setup, and configurable chrome

> **Agents:** read this **before** implementing UI, flows, onboarding,
> Feature / Settings toggles, or “this client needs…” work in any Kumkaj
> app. Cursor rule: `.cursor/rules/templates-configure-not-hardcode.mdc`
> (always on). Sibling Gotigin note: Wadhu
> `docs/GOTIGIN_CONFIGURE_NOT_HARDCODE.md`.

## 1. Two different “concrete” ideas (do not confuse them)

| Doctrine | Meaning |
|----------|---------|
| **Concrete product** ([`generic-concrete.mdc`](../.cursor/rules/generic-concrete.mdc), vision §1) | Moklanjo is a courier OS, Khaynjo is a food-floor OS — **not** one vague “works for anyone” SKU, and **not** a shared Kumkaj product core. |
| **Configurable shop setup** (this doc) | Inside one product, the **same app** serves many shops. Templates, Settings, and feature flags **show or hide** capability so the owner’s screen matches *their* layout — without forking the app into a one-client binary. |

Shipping a concrete **trade** app does **not** mean hardcoding one restaurant’s or one courier desk’s layout as the only face of that app.

## 2. What client / app templates are for

**Templates are a power feature.** Authored packs (e.g. Khaynjo’s Rajdhani App Template) and Export template / shop defaults configure:

- menu / rates starting points
- staff roles and chrome defaults (without baking PINs)
- pay methods, tax chrome, KOT density, which sections are hidden
- Floor / Kitchen / other peers the owner actually uses

The goal of a **client implementation** is **not** to make the app
concrete to that client. It is to **configure** the shared product so
that shop sees what they need and does not see what they do not.

### Example (Khaynjo)

Counter and kitchen sit next to each other: they do not need heavy
in-app kitchen communication. A template / Settings face can **disable
or hide** Kitchen / KDS / station chatter. The owner is happy because
nothing unused clutters the screen — the features still exist in the
product for shops that need them.

### Wrong agent direction (seen often)

```text
❌ “This client never uses Floor → delete Floor / simplify the app to
   counter-only forever for everyone.”
❌ “Rajdhani always cash-only → hardcode cash-only in Order desk.”
❌ “This courier only does cash walk-ins → remove Account booking.”
❌ New SKU / fork of the app per client layout.
```

```text
✅ Implement the capability in the product once.
✅ Expose it through Settings / rights / template `shop_defaults`
   (hide, default off, or preset on).
✅ Let the owner (or an imported pack) choose the face for that shop.
✅ Keep progressive depth: unused capability stays in the product,
   out of the way until enabled.
```

Vision already says *hide unused capability inside each product;
progressive depth*. Templates and Settings are how that happens for
real client setups.

## 3. Desk rush: same “many people waiting,” different products

**Shared idea (every Kumkaj app):** mid-work can **pause** one customer,
serve another, then **resume** — unfinished desk work is a **draft** (not
books until Take payment / settle). The *shape* of that interrupt surface
is per product, and **one app may expose more than one surface** (e.g.
Khaynjo Floor *and* Counter drafts). Templates / Settings show or hide
each surface; they must not remove the ability to handle more than one
open customer.

Agents often invent the wrong board. Use the product’s real model.

### Khaynjo (food) — Floor slots **and** Counter drafts (independent)

Two interrupt faces for the same pause/resume idea. **Floor on/off does
not gate Counter drafts** — each surface is its own Settings/template
toggle. A shop may run Floor only, Drafts only, both, or (poorly) neither.

**A. Floor / Parcel (table-shaped open tickets)** — when Floor is shown:

1. Put each wait on a **Floor** slot (Parcel **P1…** is usual for walk-ins).
2. Punch → **Send KOT** → leave payment alone.
3. Tiles show **name (e.g. P3), amount, status · minutes**.
4. Settle when handed over; the slot frees on pay.

**B. Counter drafts** — always available on the Order desk when Drafts is
shown (Floor may be on or off; that does not matter):

1. Mid-order → **Park draft** (cart saved, not books) → desk clears for the
   next person.
2. **Drafts** nav shows dynamic cards of open counter drafts (kept until
   Discard or pay — not wiped by day close); tap → resume on Counter.
3. Take payment closes that draft.

Floor is for seated/parcel status; Drafts is for counter interrupt. They
can coexist in one shop. Hiding Floor must not remove Park/Drafts; hiding
Drafts must not remove Floor.

Kitchen (if enabled) advances Floor tickets; it is not a substitute for
Counter park/resume.

### Moklanjo (courier) — Park draft + Board, then Ops for status

Rush at the desk is **not** Khaynjo Parcel tiles:

1. Destination → weigh → **ADD**. Mid-work → **Park** (cart + parcel form
   saved as a **draft**, not books) → desk clears for the next customer.
2. **Board** shows **dynamic cards** only: open drafts (kept until the desk
   **Discard**s or finishes them — not cleared by day close) + **today's
   completed bookings** (business-day window). Filter All | Draft | Booked;
   column density 2/3/4; tap a draft → **resume** on Booking.
3. Finish → Cash or Account → **Take payment** → **Next booking** (closes the
   draft if you resumed one).
4. Pay-later freight → **Account booking** + **Credit / khata** (needs a
   party), not an anonymous hold board.
5. Parcel **ops** status after commit → **Ops** lanes
   (To dispatch → With carrier → Out for delivery → Failed → Done).
6. **Ops** is post-commit tracking — not a queue of unpaid desk drafts.

Detail: [`apps/moklanjo/docs/MOKLANJO_APP_TUTORIAL.md`](../apps/moklanjo/docs/MOKLANJO_APP_TUTORIAL.md)
§3 and §7; Ops board: [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md).

### Do not copy one product’s board into the other

```text
❌ Parcel Floor tiles inside Moklanjo for walk-in queue
❌ Forcing walk-in rush onto Floor because “Counter has no park”
❌ Gating Counter Park/Drafts on Floor being off (or on)
✅ Same pause/resume idea; Floor and Drafts independent in Khaynjo; Board in Moklanjo
✅ Templates show/hide each surface — capability stays in the product
```

## 4. Where configuration lives (Khaynjo today; siblings follow)

| Layer | Role |
|-------|------|
| Public / authored **App Template** packs | Onboarding face: menu + `shop_defaults.json` (chrome, pay, tax, staff without PINs) |
| Menu **Item templates** | Items only from the same catalogue grammar |
| **Export template** | Capture a live shop as a pack for another device / catalogue |
| Settings + Team rights | Day-to-day show/hide and who may use money / Kitchen / Floor |
| Day close · business day | Shop meta `business_day_mode`: default **Until Day close** (keeps going past midnight); optional **Calendar midnight**. Owner switches on Day close — not hardcoded per client. See [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) § Business day |
| Product code | Always implements the full capability set the trade needs |

Catalogue / publish: [`apps/khaynjo/docs/CATALOGUE_URLS.md`](../apps/khaynjo/docs/CATALOGUE_URLS.md).
Demo folders: [`apps/khaynjo/assets/templates/README.md`](../apps/khaynjo/assets/templates/README.md).

Moklanjo / Kirano / Tabiyat get the same **configure, don’t hardcode**
rule even where Export template is not shipped yet.

## 4a. App Template import — when, from where, wipe prompt (every app)

Same rule in every Kumkaj product (Khaynjo, Moklanjo, Kirano, Tabiyat).

### Before Server creation (first-time DeviceSetup offer)

No books yet — **do not** show a wipe Continue/Cancel prompt.

The owner may:

1. **Public** — pick a published App Template and import (or Skip).
2. **This device** — import a pack folder they already have
   (`template.json` + optional `shop_defaults.json`).
3. **Cloud (paid) — future** — sign in and download a pack from Gotigin’s
   server. **Not implemented yet**; do not build login here until the paid
   catalogue path is specified. Leave UI/docs hooks only.

### After the shop exists (anytime)

Importing an **App Template** (public, this-device pack, or later cloud)
**always** shows **Continue / Cancel** first:

- **Books trading data will be cleared** (sales / bookings / ledgers that
  count as books entries — not a silent merge).
- **Shop settings / config from the pack will be applied** (chrome, pay,
  tax face, staff without PINs, etc. as that product’s pack defines).

**Continue** runs wipe then apply. **Cancel** leaves the shop untouched.

Item-only Menu imports (add dishes / rates without replacing shop face)
keep their own lighter confirm; they are **not** a substitute for App
Template re-import.

```text
❌ Wipe prompt on first-time empty Server offer
❌ Re-applying an App Template after create with no Continue/Cancel
❌ Cloud login UI pretending to work before the paid path ships
✅ First-time: public + this-device (+ cloud later) with no wipe prompt
✅ Anytime later: same sources + mandatory wipe/settings warning
```

## 5. Agent checklist (before coding)

1. Is this a **new trade capability** (belongs in the product), or a
   **shop preference** (belongs in Settings / template / rights)?
2. Would hardcoding this client’s layout break another shop of the same
   product?
3. Can an owner who needs the opposite setup turn it back on without a
   rebuild?
4. For rush / status UX — which product model (§3) applies?
5. Did you read this doc and the product’s `apps/<app>/docs/` brief for
   that screen?

If the request is “make it like Client X,” default answer is **template +
defaults + hide unused chrome**, not a permanent one-client app shape.
