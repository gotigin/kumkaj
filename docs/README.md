# Kumkaj

Kumkaj (kumkaj.com) — brand for a **family of concrete business operating system apps**. Local-first on Makepad 2. **Current focus: [Kumkaj Courier](KUMKAJ_COURIER.md)** — full courier-agency BOS (books + booking desk, AWB/barcode, tracking as the app can own them). Other concrete products come later; shared code is extracted when a second product needs it (vision §1–2). **Software should adapt to users — not the other way around.**

Offline use is free forever on the device. Extra tills are optional LAN joins. Cloud / ESC/POS / statutory e-invoicing are optional later.

**Courier shell:** Day book · Booking · Pay out · More ([`KUMKAJ_COURIER.md`](KUMKAJ_COURIER.md), [`DESIGN.md`](DESIGN.md), [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)).

**UI:** Stitch *Tactile Countertop* (cool surface, navy chrome, terracotta accents). See [`DESIGN.md`](DESIGN.md); full pack at `../../stitch_tactile_countertop_design_system/`. Makepad 2 button selection + label rules for agents: [`DEV.md`](DEV.md) § Makepad 2 UI faces (agent guide).

**Auth:** Owner hosts the shop; Staff unlock with their own PIN (or optional login) and owner-defined role title. Sale authorship is immutable. Extra tills pair on the outlet Wi‑Fi when the shop allows them — see [`DEV.md`](DEV.md).

**Package ids:** production `com.kumkaj.courier` · development `com.kumkaj.courier.debug` (labels Kumkaj Courier / Kumkaj Courier Dev). Same reverse-DNS id on Android, desktop data dirs, and (when signed) Apple bundles. Details and per-OS paths: [`DEV.md`](DEV.md) § Development vs production env.

**Scale**

- Every launch: language (platform default) + Server/Client on one screen ([`DESIGN.md`](DESIGN.md))
- One device can hold **multiple local servers** (separate businesses/products); Server Mode creates the first or lets you create/open another
- Shop device by default (extra tills off). Allow extra tills in Settings when another phone should sell.
- Tables & kitchen (tables, KOT split by kitchen station) is optional for food businesses
- Categories, named tax rates, modifiers on Items; Inventory & purchases
- Organization + branches with snapshot export/import and consolidated reports
- A write-only sync outbox records each sale for a future Kumkaj Cloud drain; nothing reads that queue yet. Cloud syncs the shop books — extra tills are not a different cloud product.
- Books rows keep three clocks: user/business time, device write time, and cloud accept time (empty until synced) — see [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md).

**Country packs**

Apply a country in Settings to set the tax name and maths (VAT · GST · sales tax · IVA · BTW · SST · consumption tax), its usual rates, the instant-pay rail (UPI · Pix · PayNow · FPS · Interac · …), and the business-id label (GSTIN · VAT number · ABN · EIN · TRN · …). Prices can include tax or have it added at the till. Kumkaj does the arithmetic and wording only — **no statutory e-invoicing, fiscal printers, or tax filing** anywhere. See [`fiscal.rs`](../apps/courier/src/fiscal.rs).

**Sell cart totals**

- Line: **Actual** (`@ × qty`) and **Applied** (editable; defaults to Actual)
- Bill: **Actual Total**, **Applied Total**, **Rounded Total** (editable charge; defaults to Applied)
- Tax is extracted from the total (inclusive pricing) or added on top of it (exclusive pricing) at finalize

**Ship:** see [`DEV.md`](DEV.md) packaging + publish checklist. Website: **https://kumkaj.com** (`../web/`). Apple App Store deferred.

See [`KUMKAJ_COURIER.md`](KUMKAJ_COURIER.md) (current product), [`USE_CASES.md`](USE_CASES.md), [`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md), [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md), and [`DEV.md`](DEV.md).
