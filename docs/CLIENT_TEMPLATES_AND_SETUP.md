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

Agents often invent the wrong board. Use the product’s real model.

### Khaynjo (food) — open tickets, pay when ready

When several people order **without paying yet**, and the counter only
wants **status** (not names):

1. Use **Floor → Parcel** (or any free slots), not a nameless draft cart pile.
2. Punch → **Send KOT** → leave payment alone.
3. Floor tiles show **name (e.g. P3), amount, status · minutes**
   (In kitchen → Food ready → Served → Bill printed).
4. Settle when handed over; the slot frees on pay.
5. Customer name is optional; **P-number + status + amount + time** is enough.

Pure **Counter** order type is weaker for “eight unpaid waits” — that is
a finish-this-ticket path. Many unpaid waits belong on **Floor slots**.

Kitchen (if enabled) advances tickets; Floor stays the glance board.
Templates may hide Floor/Kitchen entirely for a counter-only shop.

### Moklanjo (courier) — finish booking, then Ops for status

Rush is **not** eight open unpaid nameless carts:

1. Destination → weigh → **ADD** → Cash or Account → **Take payment** →
   **Next booking**.
2. Pay-later freight → **Account booking** + **Credit / khata** (needs a
   party), not an anonymous hold board.
3. Parcel **ops** status after commit → **Ops** lanes
   (To dispatch → With carrier → Out for delivery → Failed → Done).
4. COD is collected around delivery; it is not a Floor of unpaid desk
   drafts.

Detail: [`apps/moklanjo/docs/MOKLANJO_APP_TUTORIAL.md`](../apps/moklanjo/docs/MOKLANJO_APP_TUTORIAL.md)
§3 and §7; Ops board: [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md).

### Do not copy one product’s board into the other

```text
❌ Parcel Floor tiles inside Moklanjo for walk-in queue
❌ Khaynjo “Next booking” clearing the only open unpaid ticket board
✅ Each trade’s desk model + templates to hide what that shop skips
```

## 4. Where configuration lives (Khaynjo today; siblings follow)

| Layer | Role |
|-------|------|
| Public / authored **App Template** packs | Onboarding face: menu + `shop_defaults.json` (chrome, pay, tax, staff without PINs) |
| Menu **Item templates** | Items only from the same catalogue grammar |
| **Export template** | Capture a live shop as a pack for another device / catalogue |
| Settings + Team rights | Day-to-day show/hide and who may use money / Kitchen / Floor |
| Product code | Always implements the full capability set the trade needs |

Catalogue / publish: [`apps/khaynjo/docs/CATALOGUE_URLS.md`](../apps/khaynjo/docs/CATALOGUE_URLS.md).
Demo folders: [`apps/khaynjo/assets/templates/README.md`](../apps/khaynjo/assets/templates/README.md).

Moklanjo / Kirano / Tabiyat get the same **configure, don’t hardcode**
rule even where Export template is not shipped yet.

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
