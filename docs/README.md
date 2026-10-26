# Kumkaj

Kumkaj (kumkaj.com) — brand for a **family of concrete business operating system apps**. Local-first on Makepad 2. **Current focus: [Kumkaj Courier](../apps/courier/docs/KUMKAJ_COURIER.md)** — full courier-agency BOS (books + booking desk, AWB/barcode, tracking as the app can own them); Khaynjo (restaurant OS) is the second concrete product. **The only shared thing is the common workspace** — same Makepad 2 and similar libraries, because BOS apps have similar requirements. There is **no Kumkaj core / shared product code**: each app owns duplicated, independently-evolving code, so a change in one app never impacts another (vision §1–2). **Software should adapt to users — not the other way around.**

**Workspace layout:** this `docs/` folder holds only **global** doctrine (vision, business-books concepts checklist, DEV, DESIGN). Each app owns its product code, docs and launch scripts: `apps/<app>/` (e.g. [`apps/courier/docs/`](../apps/courier/docs/KUMKAJ_COURIER.md)). Root `scripts/` are build/packaging tooling only, parameterised per app (KUMKAJ_APP_DIR / app env files) — never product logic; fork a script per app rather than let one app's tooling change break another.

## Documentation system (read before adding or moving a doc)

All Kumkaj apps get their core ideas and vision from Kumkaj. Docs are
therefore organised by **scope**, with one canonical source per idea:

| Scope | Lives in | Contains | Examples |
|-------|----------|----------|----------|
| **Kumkaj-wide** | root `docs/` | Ideas, vision, principles and concepts that apply to every current/future app | Vision, [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) (books concepts), [`UNITS_AND_CONVERSIONS.md`](UNITS_AND_CONVERSIONS.md), [`IDENTITY_AND_NAMES.md`](IDENTITY_AND_NAMES.md) (UUID identity, names as aliases), [`DESIGN.md`](DESIGN.md) (work-surface law), [`DEV.md`](DEV.md) (platform/agent rules) |
| **App-specific** | `apps/<app>/docs/` | That trade's instantiation only: product brief, state, build plan, discovery evidence, tutorials, acceptance checks | [`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md), [`KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md) |
| **App code** | `apps/<app>/src/` | That app's own implementation of the root ideas (no shared code, vision §1) | `apps/courier/src/fiscal.rs`, `apps/khaynjo/src/store.rs` |

**Placement rules**

1. If an idea applies to more than one app — or would apply to a
   future app — its canonical text goes in root `docs/`, even if it
   was first discovered while building one app (e.g. units/chained
   conversions were first shipped in Courier and now live in
   `UNITS_AND_CONVERSIONS.md`).
2. An app doc states only that app's defaults, labels, state and
   status for the idea, and **links** to the root doc. Never restate
   root text in an app doc — duplicated docs drift (the opposite of
   code, where duplication is preferred to coupling).
3. Discovery evidence stays with the app that discovered it
   (checklists, tutorials, market research), with a scope note
   pointing at the root concept it evidences. Evidence is not
   doctrine for other apps.
4. Only create a new root doc when a Kumkaj-wide topic is large and
   has no canonical home in the vision / `BUSINESS_BOOKS.md` /
   `DESIGN.md` / `DEV.md`. Otherwise extend the existing home and
   link. Keep this index updated in the same change.
5. When a root idea changes, update the root doc once, then update
   each affected app's own code/doc separately, app by app.

Offline use is free forever on the device. Extra tills are optional LAN joins. Cloud / ESC/POS / statutory e-invoicing are optional later.

**Courier shell:** Day book · Booking · Pay out · More ([`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md), [`DESIGN.md`](DESIGN.md), [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)).

**UI:** Stitch *Tactile Countertop* (cool surface, navy chrome, terracotta accents). See [`DESIGN.md`](DESIGN.md); full pack at `../../stitch_tactile_countertop_design_system/`. Makepad 2 button selection + label rules for agents: [`DEV.md`](DEV.md) § Makepad 2 UI faces (agent guide).

**Auth:** Owner hosts the shop; Staff unlock with their own PIN (or optional login) and owner-defined role title. Sale authorship is immutable. Extra tills pair on the outlet Wi‑Fi when the shop allows them — see [`DEV.md`](DEV.md).

**Package ids:** production `com.kumkaj.courier` · development `com.kumkaj.courier.debug` (labels Kumkaj Courier / Kumkaj Courier Dev). Same reverse-DNS id on Android, desktop data dirs, and (when signed) Apple bundles. Details and per-OS paths: [`DEV.md`](DEV.md) § Development vs production env.

**Scale**

- Every launch: language (platform default) + Server/Client on one screen ([`DESIGN.md`](DESIGN.md))
- One device can hold **multiple local servers** (separate businesses/products); Server Mode creates the first or lets you create/open another
- Shop device by default (extra tills off). Allow extra tills in Settings when another phone should sell.
- Tables & kitchen (tables, KOT) belong to Khaynjo (restaurant OS) — not shipped in Kumkaj Courier
- Categories, named tax rates, modifiers on rates/items; Inventory & purchases when needed
- Parties: identity is UUID (names are aliases and may repeat) — [`IDENTITY_AND_NAMES.md`](IDENTITY_AND_NAMES.md); roles (Consignor / Carrier / …) may combine on one id; AR and AP stay separate with a net “who owes whom” view — [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)
- Each branch is its own business: one local server (books) per branch, compared through exported snapshots (vision §5)
- A write-only sync outbox records each sale for a future Kumkaj Cloud drain; nothing reads that queue yet. Cloud syncs the shop books — extra tills are not a different cloud product.
- Books rows keep three clocks: user/business time, device write time, and cloud accept time (empty until synced) — see [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md).

**Country packs**

Apply a country in Settings to set the tax name and maths (VAT · GST · sales tax · IVA · BTW · SST · consumption tax), its usual rates, the instant-pay rail (UPI · Pix · PayNow · FPS · Interac · …), and the business-id label (GSTIN · VAT number · ABN · EIN · TRN · …). Prices can include tax or have it added at the till. Kumkaj does the arithmetic and wording only — **no statutory e-invoicing, fiscal printers, or tax filing** anywhere. Each app implements packs in its own `src/fiscal.rs` (e.g. [`apps/courier/src/fiscal.rs`](../apps/courier/src/fiscal.rs)); canonical wording: vision §23.

**Sell cart totals**

- Line: **Actual** (`@ × qty`) and **Applied** (editable; defaults to Actual)
- Bill: **Actual Total**, **Applied Total**, **Rounded Total** (editable charge; defaults to Applied)
- Tax is extracted from the total (inclusive pricing) or added on top of it (exclusive pricing) at finalize

**Ship:** see [`DEV.md`](DEV.md) packaging + publish checklist. Website: **https://kumkaj.com** (`../web/`). Apple App Store deferred.

See [`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md) (current product), [`COURIER_CLIENTS_REQUIREMENTS.md`](../apps/courier/docs/COURIER_CLIENTS_REQUIREMENTS.md) (sanitized client checklist), [`USE_CASES.md`](../apps/courier/docs/USE_CASES.md), [`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md), [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md), and [`DEV.md`](DEV.md).
