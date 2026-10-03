# Kumkaj

Gotigin's business operating system — local-first selling and books on Makepad 2.

Offline use is free forever, from a single counter through extra tills, kitchens, and multi-branch groups coordinated on your own devices. Local billing is production-ready on the shop device; extra tills are optional LAN joins. Cloud usage (WAN sync, hosted dashboards, automatic offsite backup), raw ESC/POS, and statutory e-invoicing are optional / later — not required to bill.

**Who it is for:** anyone who sells or keeps a book — a clothes seller, a tea stall, a shop with staff, or several outlets. Same Sell · Items · More shell.

**UI:** Stitch *Tactile Countertop* (cool surface, navy chrome, terracotta accents). See [`DESIGN.md`](DESIGN.md); full pack at `../../stitch_tactile_countertop_design_system/`.

**Auth:** Owner hosts the shop; Staff unlock with their own PIN (or optional login) and owner-defined role title. Sale authorship is immutable. Extra tills pair on the outlet Wi‑Fi when the shop allows them — see [`USER_EXPERIENCE.md`](USER_EXPERIENCE.md) and [`DEV.md`](DEV.md).

**Scale**

- Shop device by default (extra tills off). Allow extra tills in Settings when another phone should sell.
- Tables & kitchen (tables, KOT split by kitchen station) is optional for food businesses
- Categories, named tax rates, modifiers on Items; Inventory & purchases
- Organization + branches with snapshot export/import and consolidated reports
- A write-only sync outbox records each sale for a future Kumkaj Cloud drain; nothing reads that queue yet. Cloud syncs the shop books — extra tills are not a different cloud product.

**Country packs**

Apply a country in Settings to set the tax name and maths (VAT · GST · sales tax · IVA · BTW · SST · consumption tax), its usual rates, the instant-pay rail (UPI · Pix · PayNow · FPS · Interac · …), and the business-id label (GSTIN · VAT number · ABN · EIN · TRN · …). Prices can include tax or have it added at the till. Kumkaj does the arithmetic and wording only — **no statutory e-invoicing, fiscal printers, or tax filing** anywhere. See [`fiscal.rs`](../apps/kumkaj/src/fiscal.rs).

**Sell cart totals**

- Line: **Actual** (`@ × qty`) and **Applied** (editable; defaults to Actual)
- Bill: **Actual Total**, **Applied Total**, **Rounded Total** (editable charge; defaults to Applied)
- Tax is extracted from the total (inclusive pricing) or added on top of it (exclusive pricing) at finalize

**Ship:** see [`DEV.md`](DEV.md) packaging + publish checklist. Website: **https://kumkaj.com** (`../web/`). Apple App Store deferred.

See [`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md) and [`DEV.md`](DEV.md).
