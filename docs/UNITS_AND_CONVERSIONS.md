# Kumkaj — units and conversions (all apps)

Canonical Kumkaj-wide doctrine for measurement units. This applies to
**every** Kumkaj app (Courier, Khaynjo, future products) because every
business operating system measures something — parcels, dishes,
ingredients, stock, quantities. It is a shared *idea*, not shared code:
each app implements this doc in its own code (vision §1 — no Kumkaj
core), so an app may diverge in code but must not silently diverge in
meaning.

Companions: [`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
§23 (India-first, without locking the architecture); checklist row in
[`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) (“Catalogue + units”).

## Rule in one line

Shops measure in the units their trade and country already use. Never
force one market's units: ship **standard units with exact standard
conversions**, and let the shop define **custom units based on a
standard unit directly or indirectly** (through other custom units).

## Standard units and families

Units form **families**. Each family has exactly one canonical
**anchor** unit; books store and compare quantities only in the anchor.

| Family | Anchor (books record this) | Standard units (exact factors to anchor) |
|--------|----------------------------|--------------------------------------------|
| Weight | `kg` | `kg` = 1 · `g` = 0.001 · `t` = 1000 · `lb` = 0.45359237 · `oz` = 0.028349523125 |
| Length | `cm` | `cm` = 1 · `mm` = 0.1 · `m` = 100 · `in` = 2.54 · `ft` = 30.48 |
| Volume | `ml` | `ml` = 1 · `l` = 1000 |
| Count | `pc` | `pc` = 1 · `dozen` = 12 |

Physical conversion factors are fixed constants of nature, so applying
them to old entries is always safe.

## Custom units: direct or indirect

- A shop may define custom units and custom conversions anywhere a
  standard unit can be used (e.g. `1 bag = 25 kg`, `1 crate = 40 cm`).
- A custom unit is defined relative to **any unit of the same family**:
  a standard unit directly, or another custom unit indirectly. Its
  chain of conversions must ultimately land on the family anchor
  (`crate → box → cm` means 1 crate = 40 cm via `1 crate = 2 box`,
  `1 box = 20 cm`).
- **Loops are refused** (a chain that revisits a unit), and
  **cross-family links are refused** (a weight unit cannot convert to
  a length unit).
- A unit whose chain never reaches the anchor is shown as
  **“not linked”** and cannot be used until it is linked.
- A standard unit's own conversions are fixed and automatic — a
  conversion may never be authored *from* a standard unit
  (`1 kg = …`); define the custom unit instead (`1 bag = 25 kg`).

## Recording, entry, and display

- **Books store anchor units only** (kg / cm / ml / pc, with milli-qty
  precision), so ledgers, rates, reports and P&L aggregate exactly one
  unit per family and stay comparable.
- Conversion happens **at entry and at display**, never by rewriting
  recorded history.
- The user **chooses entry/display units** (length, weight, quantity);
  reports display in whatever unit the reader picks.
- Trade constants around measurement (volumetric divisor, weight-slab
  rounding, and similar) are **shop settings with sensible local
  defaults**, not constants baked into code (vision §2, granular
  control).

## Currency is not a unit

An exchange rate is a price at a moment in time, not a fixed
conversion, so currencies never enter the unit tables. Books are kept
in the shop's currency; when foreign-currency money is recorded, the
entry must carry the amount, the currency, the rate on that date, and
the resulting shop-currency amount — history is never re-converted at
a later rate (only an explicit “value today” view may do that).

## Per-app implementation (own code, same doctrine)

| App | Where its own implementation lives | App-specific defaults (that app's doc owns these) |
|-----|-------------------------------------|---------------------------------------------------|
| Kumkaj Courier | `apps/courier/src/domain.rs`, `src/store.rs` (Settings → Units / Booking measures) | Booking desk defaults cm + kg, volumetric ÷ 5000, round up to 0.5 kg — see [`apps/courier/docs/KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md) |
| Khaynjo | `apps/khaynjo/src/domain.rs`, `src/store.rs` (Settings → Units) | Restaurant quantities/menu and stock units — see [`apps/khaynjo/docs/KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md) |
| Future apps | Their own `apps/<app>/src/` | Copy the best-fitting app's code, then diverge; never extract a shared units crate |

## Agent rule

1. Change the doctrine here, once — then update each app's own code
   and app-specific doc separately, app by app. Never edit only one
   app's copy of this idea and call it Kumkaj doctrine.
2. When an app doc needs to mention units, state only that app's
   defaults and link here. Do not restate this file.
