# Moklanjo — Courier / logistics-agency Operating System

> **Brand / domain:** Moklanjo · moklanjo.com ("Moklanjo" = to send / act of sending in Memoni)
> **What ships:** the Kumkaj courier-agency product — a concrete business
> operating system for courier and logistics agencies, under the
> Kumkaj brand alongside Khaynjo.
> **Platform ids:** production `com.moklanjo` · development
> `com.moklanjo.debug`
> **Naming:** Moklanjo belongs to the Kumkaj brand, but its domain is
> its own — **moklanjo.com**, not a path under kumkaj.com. Its id root
> is simply `com.moklanjo` (the product *is* the brand), with `.debug`
> for dev exactly like Khaynjo's split. Data dirs, Android application
> id, iOS/macOS bundle id (when signed), and the desktop packager
> identifier all use this root. Do not re-tie Moklanjo ids to
> `com.kumkaj.*`.
> **Role:** books **and** booking-desk ops, including **handover to
> named carriers** *and* **own-fleet rider dispatch**.
> **Doctrine:** [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](../../../docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
> (concrete products in a common workspace — same Makepad 2, no Kumkaj
> core / shared product code; local-first; free local, usage-based
> cloud). This brief only says what is courier-agency-specific.

Companions: [`MOKLANJO_APP_TUTORIAL.md`](MOKLANJO_APP_TUTORIAL.md) (learn business + app + rush-desk demo), [`USE_CASES.md`](USE_CASES.md) (industry + client research), [`grokbot/MOKLANJO_BUSINESS_TUTORIAL.md`](grokbot/MOKLANJO_BUSINESS_TUTORIAL.md) (ops + tax + Part 10 mapping), [`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md) (business-books concepts checklist — concepts, not shared code), vision §1–2. Screen titles also have in-app info tips.

**Build rule:** implement this product concretely, in Moklanjo's own code only. Never promote logic into shared crates — there is no Kumkaj core. When another Kumkaj product needs the same thing, it copies Moklanjo's code and diverges (as Khaynjo did); a Moklanjo change must never impact another app.

------------------------------------------------------------------------

## Product shape (agency + own fleet)

A **complete** courier BOS for this audience covers both operating modes on one spine:

| Mode | What the owner does | In Kumkaj |
|------|---------------------|-----------|
| **Agency / franchise counter** | Book parcels, hand bags to Delhivery / DTDC / … | Rates, booking, carrier manifest, period bills, COD |
| **Own last-mile** | Same booking desk, assign AWBs to **own riders / vans** | Riders master, own-fleet dispatch, out-for-delivery → POD |

Live GPS maps and AI route optimisation are **progressive depth** on the same dispatch document — not a separate product. Building a second *national carrier network* that replaces Delhivery stays out of scope; running *your* vans is in scope.

```text
Booking desk → AWB → (carrier bag | rider run) → track / POD → bill / COD
```

------------------------------------------------------------------------

## Why the account manager was confused

He was shown a **generic till + books** shell with courier *labels*. He expected a **courier desk**:

| He asked about | What it is | Product response |
|----------------|------------|------------------|
| Many walk-ins, each with their own parcels | Fast **new booking** loop (next person → weigh → charge → receipt → next) | Core desk UX — not “pick shop SKUs” |
| Barcoding / labels | AWB/CN on booking + print/scan barcode | Ops capability on the same booking document |
| Tracking details | Status on that AWB (booked → handed over / out for delivery → …) | Ops capability; deep carrier-network track may stay with carrier portal until we integrate |

**One product covers both:** money books **and** desk ops that an app can own.

------------------------------------------------------------------------

## Industry baseline (top agency apps)

Sources: Bellboy, GoInvoicing, ESE, CourierDesk, Saap, BillMyCourier, aggregators (Shiprocket-class), plus client WhatsApp (`USE_CASES.md`).

### A. Books & money (must-have — client already asked)

Canonical books concepts for all Kumkaj apps live in
[`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md); this table is
only Moklanjo's Ship/Rule instantiation of them. Party identity follows
the Kumkaj-wide rule — uniqueness by UUID, names as aliases —
canonical in [`IDENTITY_AND_NAMES.md`](../../../docs/IDENTITY_AND_NAMES.md);
Moklanjo's role labels on that one UUID are Consignor / Carrier /
Supplier.

| Feature | Notes | Moklanjo target |
|---------|--------|------------------------|
| Consignor + carrier accounts | One party; roles Consignor and/or Carrier (same firm may be both); phone, address, optional logo, optional custom fields (e.g. WhatsApp) | **Ship** — Parties multi-role + address + logo/photo + custom key/value fields; AR+AP ledgers; net position in reports |
| Debtors & creditors / aging | AR + AP (separate); net who-owes-whom | **Ship** — Parties + Reports |
| Day book | Today money in/out | **Ship** — Day book tab / Reports |
| Bank book | Till ↔ bank | **Ship** — Bank book & P&L |
| Expense voucher | Running costs + bill photo/PDF | **Ship** — Pay out / Reports + attachments |
| Entry notes / attachments | Docket photos, party docs, expense bills | **Ship** — notes + WebP images + PDF on sale / party / expense |
| Invoice | Account booking path | **Ship** — Booking + Account booking |
| Cash booking vs Account booking | Pay now vs party ledger | **Ship** — bill-type tag |
| Mixed settlement, cash-settled parts | One bill / expense settled part bank, part cash; a supporting bill may cover only part of it | **Rule** — books record the **actual total**, split by tender; document optional, never a gate (vision §18 “Ground truth”; `BUSINESS_BOOKS.md` mechanics) |
| Party ledger + outstanding | Running balance | **Ship** / deepen |
| P&L / monthly | Period books | **Ship** — Reports month |
| Staff rights | PINs / roles | **Ship** (Team) |

### B. Booking desk (must-have for “real courier app”)

| Feature | Notes | Target |
|---------|--------|--------|
| Consignment / docket booking | Party, destination, weight, service + receipt | **Ship** — Booking desk |
| Walk-in flood | Clear cart / “new booking” every customer | **Ship** — Next booking |
| Rate master (zone × service × slab) | ₹/kg, first/extra slab | **Ship** — zone master + first/extra kg on Rates |
| Chargeable weight | Actual / volumetric / slab | **Ship** — Domestic default L×B×H ÷ 5000 (shop setting); Booking desk **Domestic / International** chip switches ÷6000 air for that booking without rewriting Settings; round up 0.5 kg |
| Measurement units | User-chosen entry units, standard + custom conversions | **Ship** — Moklanjo defaults: booking desk cm + kg, volumetric ÷ 5000, round up to 0.5 kg (Settings → Booking measures / Units). Canonical units doctrine (standard units, chained custom units, anchor recording) for all apps: [`UNITS_AND_CONVERSIONS.md`](../../../docs/UNITS_AND_CONVERSIONS.md) |
| Extra charges | Fuel, docket, ODA, packing, COD fee | **Ship** — fuel/docket on desk; catalog seeded |
| AWB / CN + **label / barcode** | Unique id; print; scan | **Ship** — Code 128 SVG label + Ops Find/Scan (wedge) → desk card → advance/COD |
| Carrier AWB / barcode **number pools** | Start–end series per courier; next from pool | **Ship** (2026-10-29) — Settings pools + allocate on book |
| Cargo / multi-piece booking | Pieces + heavy kg on same spine | **Ship** (2026-10-29) — desk `Pieces` + receipt |
| City / zone / pincode | Destination masters | **Ship** — zone master + booking zone + pincode CSV serviceability |
| Party-specific rates | Consignor override cards per zone | **Ship** — `products.party_id` + booking resolve |
| Carrier rate compare | Sell vs buy cards + margin at desk | **Ship** — `carrier_party_id` cards + compare line |
| Carrier bill reconcile | CSV match AWB / weight / charge | **Ship** — import + report + apply costs |
| Bulk booking | Many dockets | **Ship** — CSV parse (`notify::parse_bulk_booking_csv`) |
| Credit limit on account | Block over-limit | **Ship** — `credit_limit_paise` + gate on finalize |

### C. Ops (carrier handover **and** own fleet)

| Feature | Target |
|---------|--------|
| Tracking statuses on AWB | **Ship** — timeline events + status advance |
| Bulk status update | Multi-select AWBs → one status | **Ship** (2026-10-29) — Ops bulk paste |
| Barcode scan-to-find / act | **Ship** — Ops AWB field + Find (scanner Enter); card + OFD / Delivered / label / COD |
| POD note / photo | **Ship** — note + WebP POD on sale (`pod_ref` = attachment id); ops desk picker |
| Manifest / handover list | **Ship** — carrier bag **or** own-fleet rider run |
| Printable DRS / run sheet | **Ship** — receipt-route print with COD total + sign-off |
| NDR / failed delivery / RTO | **Ship** — reasons, attempts, reattempt, auto-RTO after N |
| Docket compliance fields | **Ship** — declared value, insurance, contents, e-way (print only) |
| Rider COD + pay settlement | **Ship** — expected vs handed-in + per-delivery/kg commission slip |
| Riders / vans | **Ship** — riders master + assign on manifest |
| Rider app mode (same binary) | **Ship** — download run, offline POD/signature/COD/NDR → LAN outbox |
| Signature capture (POD) | **Ship** — strokes → WebP attachment (desk + rider) |
| Incoming manifest | **Ship** — scan/type hub AWBs → delivery dockets |
| Pickup requests / runs | **Ship** — multi-pickup → one drop, assign rider |
| Auto-assign + editable stops | **Ship** — area → rider round-robin; sort/reorder anytime |
| Vehicle inspection / fuel | **Ship** — checklist per run + fuel note |
| A4/A5 docket template | **Ship** — consignor/consignee blocks (`format_docket_text`) |
| Franchise commission statement | **Ship** — period % of margin/revenue per carrier |
| Tally voucher export | **Ship** — beside existing CSV |
| RTO charges | **Ship** — fee line on auto-RTO |
| LAN corporate self-booking | **Ship** — `GET/POST /book` HTML on office Wi‑Fi: the company names itself, picks a service; priced by weight with that rate's slab (as at the desk) and **billed to its account** — never cash the drawer does not have (fixed 2026-10-31; it booked as Owner, cash, at the first rate) |
| Phone-camera barcode | **Ship** — pick image decode; wedge primary |
| Pickup / dispatch lists | **Ship** — same manifest + status filters |
| COD register + remittance | **Ship** — COD entries + remit batch |
| Carrier cost / margin on docket | **Ship** — `carrier_cost_paise` + margin helper + bill CSV reconcile |
| Period party bill | **Ship** — unbilled account dockets → one invoice |
| Generate bill vs Generate & print | Dual CTA on period bill | **Gap** — UX (C Trunk) |
| Bill-reminder / unpaid invoices list | Chase view beyond aging | **Ship** (2026-10-29) — Reports + Parties |
| Settlement TDS / discount / cheque on party pay | Record cut + instrument | **Ship** (2026-10-29) — Pay selected fields |
| Print stamp + payment QR (with logo) | Franchise bill face | **Ship** (2026-10-29) — Settings + receipt face |
| Month GST summary CSV | CA handoff (not GSTR filing) | **Ship** (2026-10-29) — Reports CSV |
| Month mix report (courier / service / top clients) | Reports facet; Day book stays home | **Ship** (2026-10-29) — Reports action |
| Consignment search/filters | **Ship** — AWB / party / date / billed / status |
| WhatsApp / SMS notify drain | **Ship** — outbox + Open WhatsApp / Copy SMS / mark sent. SMS **delivery** Planned: Local SMS Gateway (Android sibling) + paid Cloud SMS — [`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md) |
| One-tap WhatsApp receipt | **Ship** — `wa.me` with receipt text |
| Optional GPS on POD | **Ship** — lat/lon on tracking event when OS provides; no map tiles |
| Delivery feedback (local) | **Ship** — score on delivered AWB; no public Net survey |
| Destination autocomplete | **Ship** — pincode service + past destinations beyond exact 6-digit |
| Public track-by-AWB | **Ship** — `GET /track/{awb}` on LAN server (no pairing code) |
| Aggregator rate compare | **Ship** — local carrier cards (live API later) |
| Live GPS map / AI routes / Cloud ETA | Progressive depth / **Cloud skipped** — optional Net GPS points only |

### D. Good features competitors have that we should not skip

Prioritize in this order for Moklanjo:

1. ~~**Booking desk** (walk-in loop + weight + cash/account)~~ **done**
2. ~~**AWB + barcode/label print**~~ **done** (SVG label + text AWB)
3. ~~**Rate/zone/slab** deepen~~ **done**
4. ~~**Period party bill**~~ **done** (store API + Reports)
5. ~~**Tracking + manifest** (carrier **and** own fleet)~~ **done**
6. ~~**COD register**~~ **done**
7. ~~**Carrier cost / margin**~~ **done** (incl. CSV reconcile)
8. ~~**Research Local/LAN catch-up**~~ **done 2026-10-27** — rider mode, DRS, NDR/RTO, pincode, rate compare, bill reconcile, party rates, pickups, incoming manifest, barcode image, WhatsApp/SMS drain, Tally/franchise, … Full matrix: [`COURIER_RESEARCH.md`](COURIER_RESEARCH.md) §5–§7.
9. ~~**C Trunk / ASP.NET Local gaps (2026-10-29)**~~ **done** — AWB pools, bulk status, settlement TDS/cheque, bill reminder, stamp/QR, GST month CSV, cargo/multi-piece, month mix report — [`COURIER_RESEARCH.md`](COURIER_RESEARCH.md) §3.4b–c / §7.

------------------------------------------------------------------------

## Shell (courier names)

| Tab | Moklanjo meaning |
|-----|-----------------|
| Day book | Today’s position (Owner/Sales home; limited roles unlock onto their first permitted peer) |
| Booking | Money in — book parcels |
| Pay out | Money out — carrier bills & expenses |
| Rates · Ops · COD · Riders · Bookings · Day close · Reports · Parties · Team · Settings · Lock | Bottom/rail peers (scrollable, role-gated) |

**Rights match chrome.** Day book hub jumps and money sections use the same
gates as nav — never show Expense / Day close / Bank / Parties / TODAY money
to someone whose rail hid those peers (`docs/DEV.md` § Staff rights vs chrome).

**Rearrangeable tabs.** The tab order is the shop's, not ours: press a tab and drag it onto another to swap them (bottom bar and rail alike). The arrangement is saved in the books (`ui_nav_order`), so it survives reinstall and follows the shop, not the device. Scrolling stays one gesture away: the bars keep whitespace lanes around the tabs — swipe the lane to scroll, grab a tab to drag. Holding a dragged tab against the left/right edge of the bar (top/bottom of the rail) auto-scrolls it, so tabs that start off-screen can still be reached and swapped with. Implementation rules that keep this working are in `docs/DEV.md` (“Nav drag-to-swap”).

**Booking desk: Parcel | Counter side by side at every width (2026-11-02).** Never one above the other, phones included. Both fit → they share the width; else each pane keeps its min (600 | 520) and the desk pans sideways — but a pane is never wider than the screen: on a phone each fills it with a 28 px peek of the other, so a swipe goes Parcel ↔ Counter (`booking_desk_pane_widths`). Mouse / trackpad sideways scroll pans it too (the old `claim_hstrip_nested_scroll` pre-claim stopped every sideways strip after one step — removed; Makepad chains nested scroll on its own).

**Settings is a hub (2026-11-02).** One row per section — Language & currency, Country & tax, Your brand, Printers, AWB pools, Booking measures, Units, Extra tills & branches, Alerts, Backup, Diagnostics, Sidebar settings — each a `HubDestBtn` with a Font Awesome mark (the Books hub's look). A row opens that section alone, from its top; the header ‹ / Back / Alt+← / Android back close it to the hub before leaving Settings; tapping the Settings tab again starts at the hub. **Alerts:** how long messages stay for the signed-in person (3–60 s, default 10; warnings 1.5×, errors 2×). Hub rows stay full-face taps; Device setup accordions keep ℹ beside the title and extend the hit with a Fill `_ext` (`docs/DEV.md` bug **7c** — most toggles, with room for exceptions like the sidebar pill).

**Collapsible side rail (≥720 css px).** **‹** at the top of the rail hides the whole nav (zero width); **›** in the header restores it. Phone bottom tabs are unchanged. Same pattern as Khaynjo (`rail_collapsed` in `apply_shell_layout`).

**Responsive + selectable text.** Layouts reflow to the window; on-screen display text is selectable/copyable by default (`WrapLabel` / `ReportText`). Exceptions only inside tap targets — see [`docs/DESIGN.md`](../../../docs/DESIGN.md) § Always responsive and § Text selection & copy.

------------------------------------------------------------------------

## Languages & RTL

Moklanjo’s static UI — every label, heading, button, placeholder,
hint, and the setup / lock / onboarding screens — ships fully
translated in the 29 Kumkaj languages (list and doctrine: vision
§23). The dictionary is Moklanjo’s own (`src/locale.rs`,
`src/locale_chrome.rs`), never a shared crate; the
`shipped_languages_translate_chrome` test fails if any key would fall
back to English in any shipped language, so new labels land with all
translations. First launch follows the device language; afterwards
the saved per-device choice wins
(`local_servers::{load,save}_app_language`). Owner-typed names are
never translated. Arabic and Urdu run right-to-left: Makepad shapes
and bidi-orders the script, and `apply_text_alignment` in `main.rs`
right-aligns static text and input fields (vendor patch:
`TextFlow::paragraph_align_x`, `TextInput::label_align`); the window
layout itself is not mirrored.

## Modules (keep `main.rs` thin)

| Module | Role |
|--------|------|
| `booking.rs` | Chargeable kg, AWB, slab apply, extras as lines |
| `rates.rs` | Zones, slabs, extra charge kinds |
| `consignments.rs` | Search filters, credit gate, period bills |
| `manifest.rs` | Carrier **or** own-fleet handover, riders, tracking statuses |
| `cod.rs` | COD register / remittance, margin |
| `labels.rs` | SVG AWB label |
| `notify.rs` | Alert outbox + bulk CSV |
| `track.rs` | Public track JSON view |
| `courier_ops_view.rs` | Reports-facet text + desk actions (advance / dispatch / COD / riders / period bill) |
| `media.rs` | JPG/PNG/WebP → always store WebP; rate photos, party logos and the app icon capped at 768 px (`SMALL_PHOTO_EDGE`); POD / bill attachments, stamp and payment QR stay full size |
| `lan_images.rs` | Paired devices pull rate photos / party logos from the Server (`GET /photo`, bare file names only) into `…/lan/` copies; refreshed every 6 h, failed fetches retried after 5 min |
| `app/settings_hub.rs` | Settings hub: one section open at a time, Back closes it to the hub; Alerts (per-user snackbar time) |

**Desk UI:** Ops · COD · Riders are first-class bottom/rail peers that open the Reports screen with the matching facet. **Barcode loop:** scan/type AWB → Find → desk card → Out for delivery / Delivered / Failed delivery (NDR reasons) / Reattempt / Print label / Collect COD. Carrier/own-fleet dispatch, Print run sheet / DRS, COD remit, rider add, Settle rider COD, and period bill run from the same desk; credit limit + period bill also sit on Parties.

**Reports on screen (2026-10-26):** day close, books, the Ops / COD / Riders panels, the scanned-AWB card and every settings summary or list (language, country pack, units by family, booking measures, printers, audit, team, inventory, branches) render as a white card — headings, label/value rows with the value at the right, plain lines — like the Day book's cards, never one run-on paragraph (`ReportText` in `main.rs`: `set_text` → `parse_report` for printable text, `set_rows` for lists built in code). A booking's history, open bills, bank reconciliation, notes & attachments and a party's recent ledger are cards too; booking rows read "AWB · destination", then when / how paid / by whom, with the amount at the right. Right-to-left languages mirror the card (label right, value left).

------------------------------------------------------------------------

## Client WhatsApp checklist (must remain covered)

Canonical sanitized tracker: [`MOKLANJO_CLIENTS_REQUIREMENTS.md`](MOKLANJO_CLIENTS_REQUIREMENTS.md).
Courier market comparison (offline tags): [`COURIER_RESEARCH.md`](COURIER_RESEARCH.md).
LAN pilot readiness (both apps): [`PRODUCTION_STATE_FOR_OFFLINE_USAGE.md`](../../../docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md).

- Per day report · kg + credit/debit · party outstanding · bank posting · monthly · P&L / BS · invoice format  
- Named carriers & consignors as **parties** (owner enters their own names)

------------------------------------------------------------------------

## Doctrine follow-ups (Moklanjo code not yet aligned)

- **Branches (vision §5) — offline single-server 2026-11-04:** a branch is
  its own business — offline that means one Server **install** (or OS user)
  per outlet, not Create/Open another server on DeviceSetup. Multi-server
  on one install is cloud-only later. In-book *Add branch* / *Switch this
  device* stay hidden; snapshot export/import/roll-up compare branches.
  Owner wipe of local data: Settings → Backup (same categories as Windows
  uninstall). Legacy `branch_id` columns and the store's branch API stay
  for existing data.
- **Known failing test — resolved 2026-10-24:**
  `store::tests::bank_posting_aging_period_and_rich_receipt` asserted a
  `Party: …` line that `format_receipt_text` never prints. The receipt
  already names the billed party on account-booking (tax invoice)
  receipts as `Bill to: <name> (<role>)`, which satisfies the intent;
  the assertion was aligned to the shipped wording rather than adding a
  duplicate line.
- **Rights with approval + reason (vision §17) — aligned 2026-10-24:**
  Team → pick the person by name → **Void or refund a booking** and
  **Amend a recorded booking**, each Allowed / With approval (an Owner or
  Manager PIN typed at that moment) / Not allowed; cashier preset: with
  approval. `Store::check_right` decides on the main device and on the
  server for paired devices; the approver is recorded and the reason
  names both. Undoing your own booking stays free. Void/refund/amend all
  take a reason (cancel-not-delete, vision §18). Test:
  `per_person_rights_gate_void_and_keep_own_undo_free`. The feature-access
  chips on Team show on/off (active face) for the person being edited.
- **Offline office + devices (2026-10-24):** extra counters pair over
  the office Wi‑Fi (Find the shop's server, pairing code, PIN checked on
  the server), keep booking through a Wi‑Fi drop ("Offline · N waiting"),
  never record a booking twice, refresh by themselves every 20 s; the
  office's Server stays awake while serving, can start with the computer
  (Windows/Linux), backs up daily and opens its firewall from Settings.
  Any supported device can be the Server or a client — Windows, Linux or
  Android, in any mix (`docs/LAN_QUICK_START.md` §5). Design and rules:
  `docs/DEV.md` § LAN Client / Server. **Person-driven Client (2026-11-04):**
  Inventory/Branches/Team/shop Settings follow the signed-in person’s
  caps on an extra till; shop admin writes use `POST /books` (`BooksOp`);
  host-only remain pairing/firewall/extra tills/autostart/local backup
  path+restore. *2026-10-26:* printer pool + ordered
  routes per device (Receipt and AWB label; fallback, empty = don't print;
  system name or network ESC/POS; test page), device
  clock checked against the Server, Android backup copies through the
  system save dialog. *2026-10-27:* research Local/LAN catch-up **done**
  ([`COURIER_RESEARCH.md`](COURIER_RESEARCH.md) §7). Not yet run on real
  hardware.

## Lessons adopted from Khaynjo (2026-11-02)

Khaynjo ran real shop trials (LAN pilot, Android tablets, Windows counter)
and learned engineering lessons Moklanjo had not met yet. They were copied as
**approaches**, written in Moklanjo's own code and shaped for a courier
office — not Khaynjo's screens or features (no Kumkaj core; copy-then-diverge).

| Lesson (Khaynjo) | In Moklanjo | Where |
|---|---|---|
| Settings as one endless page is unusable → hub, one section at a time, Back closes the section first | **Adopted**, in Moklanjo's own hub look (`HubDestBtn` + mark, as Books) | `app/settings_hub.rs`, `ui_script.rs` `settings_hub` |
| Fixed 4 s messages vanish before staff read them → per-user alert time | **Adopted, improved**: base time per user; warnings / errors still linger longer | `snack.rs`, Settings → Alerts |
| Paired devices only get photo *keys* → fetch the files from the Server | **Adopted, hardened**: bare `.webp` names only (Khaynjo's route joins the key onto a path), retry after 5 min, refresh after 6 h | `lan_images.rs`, `net.rs` `/photo` |
| Full-size photos make lists crawl → cap photos only ever shown small | **Adopted**: rates, party logos, app icon at 768 px | `media::write_image_as_webp_fit` |
| Signals (thumbs, LAN, device queue) rebuilt Settings lists every wake | **Adopted**: found-printers list repaints only when Find's state moves (and says "Cancelled" once, not on every wake) | `main_impl.rs`, `found_printers_painted` |
| Long imports need progress + Cancel on a worker | **Adopted, improved**: CSV imports run in 250-row units (one disk sync per chunk, the write lock let go between chunks), progress on the buttons, Cancel keeps whole chunks; running the file again finishes it | `Store::import_rows`, `ImportWatch` |
| Look at the screen with realistic data | **Adopted**: opt-in courier seed (bookings in every desk state, logos, riders, carriers) | `tests/visual_seed.rs` |
| Tests for UI rules / generated server labels | **Adopted** | `shell_tests/settings_hub_tests.rs`, `local_servers` |
| Input labels visible once a field is filled (floating caption) | **Adopted**: the caption had no font (`text_style +:` on an unstyled `draw_caption`), so every filled field showed a blank band; now `theme.font_regular`. Cart / totals fields beside their own label: no caption; the line note keeps one; Rates price placeholder "Price", not "0" | `FieldInput` / `CartField` in `ui_script.rs` |
| Static UI translated in all 29 languages | **Restored**: 13 keys fell back to English (`shipped_languages_translate_chrome` was red); new keys land translated | `locale_chrome.rs` |

Considered and **not** adopted: Khaynjo's `remember_content_row` without a
redraw (its fix for a cart whose rows open and close; Moklanjo's lists have no
such rows) and `page_window` (fixed-pitch heavy grids only; Moklanjo's lists
are variable-height cards — DEV § Look at the screen 13).

## Emergency unlock PIN (offline cloudless LAN only, 2026-10-27)

Owner-only, off by default (Team). For **local Server/Client shops**, not
cloud accounts. A PIN that is **not** anyone's unlock PIN: if typed at
unlock, `try_duress_unlock` → `wipe_trading_data_keep_structure` clears
booking/sales history and the desk opens as Owner looking normal — rates,
parties, zones, riders and staff stay. Same behaviour in Khaynjo
(menu/prices/tables kept). Does **not** remove shop setup or the server
books file. Distinct from Settings → Backup → Delete local data (Owner can
also wipe shop setup there). Use only under coercion; never advertise the
wipe to the person forcing the unlock.

## Out of scope (for now)

- Replacing Delhivery/DTDC as a **national network** (hub TMS competing with carriers)  
- Claiming statutory e-invoicing / GSTR filing before implemented and reviewed  
- Pre-loading sample rates or parties  
- **Live** GPS fleet map / public customer ETA page / road-network routing (optional lat/lon on POD already ships; map tiles / Cloud ETA stay later)  
- Live carrier API book/track connectors (desk stays manual AWB; local rate compare Have)  
- Remote self-booking / public track from outside the office LAN (LAN `/book` + `/track` Have; Cloud later)

Research “still left” detail: [`COURIER_RESEARCH.md`](COURIER_RESEARCH.md) §7.
