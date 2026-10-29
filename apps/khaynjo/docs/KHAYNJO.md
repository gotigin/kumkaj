# Khaynjo — Restaurant Operating System

> **Brand / domain:** Khaynjo · khaynjo.com ("Khaynjo" = food in Memoni)
> **What ships:** the Kumkaj restaurant product — a concrete business
> operating system for restaurants and like food businesses, under the
> Kumkaj brand alongside Moklanjo.
> **Platform ids:** production `com.khaynjo` · development
> `com.khaynjo.debug`
> **Naming:** Khaynjo belongs to the Kumkaj brand, but its domain is
> its own — **khaynjo.com**, not a path under kumkaj.com. Moklanjo
> likewise uses **moklanjo.com** / `com.moklanjo`. Khaynjo's id root is
> simply `com.khaynjo` (its own domain; the product *is* the brand),
> with `.debug` for dev exactly like Moklanjo's split. Data dirs,
> Android application id, iOS/macOS bundle id (when signed), and the
> desktop packager identifier all use this root. Do not re-tie
> Khaynjo ids to `com.kumkaj.*`.
> **Doctrine:** [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](../../../docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
> (concrete products in a common workspace — same Makepad 2, no Kumkaj
> core / shared product code; local-first; free local,
> usage-based cloud). This brief only says what is restaurant-specific.

## 1. What Khaynjo is

A complete restaurant operating system adaptable to most restaurants
and like businesses: dine-in restaurants, QSRs, cafés, counter shops,
cloud kitchens, bakeries and food courts. One app, progressive depth:

```text
Sell + credit + day totals
  → menu/categories, stock, expenses, reports, printing
  → tables & kitchen (floor, KOT/KDS, stations)
  → team, permissions, audit, backup
  → extra tills / clients on the shop LAN
  → optional Kumkaj Cloud (later, usage-billed)
```

A counter with no tables never sees the floor. A waiter never has to
cross the books to send a KOT.

## 2. Principles

Khaynjo follows the Kumkaj-wide principles — they are canonical in
the root docs and are **not restated here** (edit them there, once):

- Local-first / offline-first / free forever locally; optional cloud
  can never block billing — vision §§6, 20–21.
- One Server (sole writer) with paired LAN Clients — vision §20;
  books concepts in [`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md).
- The service day is the owner's; `user_at` drives the day book —
  `BUSINESS_BOOKS.md` § Triple timestamps.
- Money as it actually moved (Actual / Applied / Rounded, split
  tenders, party credit as a core book) — vision §18,
  `BUSINESS_BOOKS.md` § Ground-truth settlement.
- India-first, global by country packs — vision §23; units and
  chained custom conversions — [`UNITS_AND_CONVERSIONS.md`](../../../docs/UNITS_AND_CONVERSIONS.md).
- Auditable history; owner-controlled retention/backup — vision
  §§18–19.

Khaynjo-specific delta on top of those: India-first also means
**GST, UPI, and veg/non-veg marking** are first-class in the
restaurant face from day one.

## 3. What was copied from Moklanjo (do not reinvent)

Khaynjo's crate is a copy of `apps/moklanjo`, renamed. Already working
and covered by Moklanjo's hard-won fixes + tests:

| Area | Where |
|---|---|
| Local store, schema, backup/restore | `src/store.rs` |
| LAN server/client (sole writer, pairing, snapshot sync, manual refresh) | `src/net.rs`, `src/local_servers.rs` |
| Money, triple timestamps, Actual/Applied/Rounded | `src/domain.rs` |
| Country/fiscal packs, tax wording | `src/fiscal.rs` |
| Tables, kitchen stations, KOT (`New → Preparing → Ready → Completed`) | `src/domain.rs`, `src/store.rs` |
| Staff PIN / login, roles, permissions, audit | `src/store.rs`, `src/main.rs` |
| Makepad 2 shell, nav faces, locales | `src/main.rs`, `src/locale*.rs` |
| Tables/KOT already in schema before Khaynjo existed | `tables_dining`, `kot_tickets`, `kitchen_stations` |

This copy is Khaynjo's own code now — change it freely for restaurants
without touching Moklanjo. When a future product needs the same logic,
it copies from whichever app fits best and diverges the same way:
never extract into shared code, and never create a Kumkaj core that
would make one app's change impact another.

## 4. Khaynjo-specific state today

- Locked product face: `khaynjo_restaurant` (`src/entity_template.rs`,
  `VenueMode::Restaurant`, Tax invoice / Cash memo lanes, no seed
  catalogue — the owner adds their own menu and parties).
- Fresh shops default to the Restaurant venue: KOT creation on,
  tables T1..T6 seeded, stations Hot/Cold seeded.
- **The copied courier product has been removed, not hidden**
  (2026-10-22): no booking desk, AWB, zones/slabs, carriers,
  manifests, COD register, riders, or public tracking anywhere in
  Khaynjo code, UI, locale strings, or party roles. Parties are
  **Customer** (khata/AR) and **Supplier** (AP) — role labels on the
  one party UUID (identity by UUID, names as aliases:
  [`IDENTITY_AND_NAMES.md`](../../../docs/IDENTITY_AND_NAMES.md)).
  What remains from
  the copy is Khaynjo's own books/billing code from §3 — store, sync, money,
  fiscal packs, staff/roles, shell — copied from Moklanjo and now free
  to diverge; it is not shared with Moklanjo.
- Owner logo is in place (`assets/khaynjo-app-icon.png`); the full
  icon set and in-app marks are generated from it, and the shell
  theme is sampled from its colors (deep red `#9A0E10`, maroon
  `#69090C`, red-orange `#D64D21`, gold `#F69D2C`, warm cream
  `#F9F4F0` — `docs/DESIGN.md` § Color tokens — Khaynjo).
- The header shows only names the owner saved (2026-10-26): the
  restaurant's name and business name (both under Settings → Your brand;
  place name shows **Khaynjo** until one is saved; business name shows
  nothing until one is saved — the old "My business" placeholder is never
  shown) and who is signed in, name and role
  ("Rajdhani Foods · Owner (Owner)"). Nothing is written for the owner
  (`Store::saved_org_name`). A local server made without a name is labelled
  "Server af76697b" in the server list only — that label is never taken as
  the restaurant's name, and books that took it earlier read as unnamed
  (`is_generated_server_label`).
- Image upload follows Moklanjo's idea, copied with the crate:
  menu-item/party photos are user-picked (or shelf) files re-encoded
  to WebP (`src/media.rs`), item/category photos capped at 768 px
  (`ITEM_PHOTO_EDGE`), with name-initials fallback
  (`src/menu_images.rs`). Lists never decode a full photo: they show
  a 192 px thumb made off the UI thread (`src/thumbs.rs`) and show
  initials until it lands.

## 4b. What restaurant people need — Khaynjo feature map (Oct 2026)

Learned from the best restaurant business systems in India and
globally, and from watching a real restaurant run its lunch and
dinner service. The vision doc (§§1–4, 6–18, 23–24, 30–33) is the
law; the field only supplies proven mechanics that serve the
people doing the work. Everything below is stated as what the
user needs — In / Later, in Khaynjo's own terms:

| User need | Khaynjo call | Where |
|---|---|---|
| 3-tap billing: tap item → qty → pay | **In (P1)** — the minimal sell path of vision §3 | Order desk |
| Order types Counter / Dine-in / Takeaway / Delivery | **In (P1)** | Desk segmented grid; `OrderType` |
| Table floor: tap table → order, table frees when its bill is paid | **Have** — areas/rooms with headings (Add area e.g. AC, then tables into it), D1–D20 ranges; Grid ↔ Layout **Have 2026-10-27** | Floor screen |
| Frequent dishes as a menu group (like Bakery Cafe) | **Have 2026-10-27** — Frequent chip next to All items; config on that view (days + count; default 1 day / 24); empty state when no completed sales in the window | Menu window; `frequent_product_ids` |
| ₹0 menu prices and ₹0 bills (complimentary / owner wish) | **Have 2026-10-27** — allowed by default; Settings → Zero-value sales can require a price above ₹0 | `allow_zero_sales` meta; finalize / Take payment |
| Table cards that show elapsed time and running amount at a glance | **In (P2)** — the counter reads the floor without opening a table | Floor screen |
| Table states in colour, one shared truth on every device: Free → **Seated** → In kitchen → Food ready → Served → Bill printed → Free on pay | **Have** — Seated without order (2026-10-27); only payment (or Free table) releases; ghost In-kitchen after pay fixed same day | Floor + server sync |
| Waiters take orders at the table on their own devices | **In (P2)** — waiters are paired Clients; they occupy tables, add items, send KOTs and request billing. Settling money is rights-gated (Take payment Allowed/WithApproval on the staff record — waiter preset Denied) | LAN service, role gates |
| KOT prints itself the moment an order is sent — no one carries paper to the kitchen | **In (P2)** — the server prints on send; Settings → Printers: pool + ordered fallback route per kitchen station (empty = don't print); hardware stays on the Server device | LAN service, print settings |
| KOT carries the waiter's name, table, item notes — and never prices | **In (P1/P2)** | KOT record + Kitchen |
| Kitchen display with ticket states New → Preparing → Ready → Completed, no prices (vision §9) | **In (P1)** | Kitchen screen |
| Variation / add-on choices and per-item special notes at order time (extra cheese, less spicy…) | **Have 2026-10-27** — Petpooja-style groups: `Size*: Small=199, Large=399 \| Toppings+3: Cheese=40` (`*` = pick one required; `+` / `+N` = add-ons, optional max N); flat list still works; free-text note; Send KOT / pay blocked until required variations picked; day report **Options sold (sizes & toppings)**; choices frozen on `sale_lines.modifiers_json`. Separate-item sizes (`Half=120, Full=200`) remain | Menu Options; `ModifierGroup`; cart chips; day report |
| Hold an order and come back to it | **In (2026-10-23)** — the table's **running order** is the ticket on the Server (pending + sent lines). Punch before Send KOT writes **pending** `running_lines` (empty `kot_id`); Floor ↔ Order bounce and other devices reopen the same Server order. **Change table** (Order button) reseats pending onto the picked table. Floor can also mark **Seated** with no order yet (2026-10-27); a live order still starts on first punch / KOT | `store_running.rs` (`bump_pending_product` / `reseat_pending` / `send_kot` promote), Floor → Order / `seat_table` |
| Settle with tender, amount handed over, change to return, and tip | **In (2026-10-23)** — Customer paid → Return to customer; tip recorded apart from sales | Order desk settle; `Store::record_settlement` |
| Part payment and Due (credit) as normal ways to finish a bill | **In (P1)** — Due lands on the customer's khata in Parties, never a side note | Pay mix; Parties |
| Customer quick-pick at billing (name/phone) so Due and history attach to the right person | **In (P1)** | Order desk customer panel |
| Split tenders (cash + UPI + card on one bill) | **In (P1)**; merge / move tables **In (2026-10-23)**; split one bill into two **In (2026-10-25)** | Pay mix; ticket Move / merge / Split bill |
| Devices always show whether everything is synced | **In (P2)** — one plain status line, on every device | Chrome; LAN service |
| Orders and kitchen keep working with no internet, all day | **In by design** — local-first; the shop server on LAN is the hub | §2, P2/P3 |
| KOT with station/category routing | **In (P1)** — `station_id` per menu item, tickets grouped per station | Send KOT, Kitchen |
| Day close the owner trusts: tender totals, credit given/collected, item sales, profit view | **In** via Khaynjo's own books; depth over report count | Day book, Reports |
| Recipe-level stock deduction, low-stock alerts | **In (2026-10-25)** — recipe; reorder; running-low. Expiry/batch alerts + PO + stock variance **In (2026-10-27)** on Inventory ops | `store_recipes.rs`, `store_ops.rs` |
| Floor-plan editor: areas, table shapes, seats/covers, coursing | **Have (2026-10-27)** — Grid ↔ Layout (X/Y/W/H + nudge + auto-pack); waitlist, sections, waiter call, hold/fire | Floor; `store_ops` / `set_table_layout` |
| Kitchen expediting: all-day item counts, fire/hold courses | **Have** — all-day counts; hold/fire; expo `/expo`; allergens on KOT; ageing + recall | Kitchen; `send_kot_filtered`, `fire_course` |
| Online delivery channels (aggregators) with payout reconciliation | **Partial** — **Have 2026-10-29** Local drop-file UX (`backups/import_aggregator_orders.json` → Settings → Bill format → Import); Net poll stub off by default; live SaaS **Skipped** (Cloud). Never gates counter | `store_ops` connectors |
| Guests ordering from a QR at the table | **Have** on Server Wi‑Fi (`GET /t/{table}`); guests on mobile data **Skipped** until Cloud | `net.rs` public LAN |
| The shop's own language, all day, on every screen | **In** — static UI fully translated in all 29 Kumkaj languages incl. Arabic/Urdu RTL alignment (vision §23) | `locale*.rs` |
| Owner controls what non-owners **see** (screens, money, reports), not only what they may **do** | **Done 2026-10-27** — Team Floor/Kitchen chips; nav gates `can_access_*` (Books/Buy/DayClose/Reports for money) | Team; [`KHAYNJO_CLIENTS_REQUIREMENTS.md`](KHAYNJO_CLIENTS_REQUIREMENTS.md) §A |
| Shop-narrowed pick-lists (units first; same pattern for areas, stations, tax rates, …) | **Done 2026-10-27** — units + areas/stations/tax On/Off in Settings; suggest lists filter | Settings; [`KHAYNJO_CLIENTS_REQUIREMENTS.md`](KHAYNJO_CLIENTS_REQUIREMENTS.md) §B |
| Mark a table **Occupied** when guests sit, before any dish is punched | **Done 2026-10-27** — `FloorState::Seated` + seat/free without order | Floor; [`KHAYNJO_CLIENTS_REQUIREMENTS.md`](KHAYNJO_CLIENTS_REQUIREMENTS.md) §C |
| Emergency unlock PIN if forced to open the till | **Done 2026-10-27** — Owner sets optional PIN on Team; unlock with it wipes sales/history, keeps menu/prices/tables/staff, opens as Owner looking normal | Team; `try_duress_unlock` |
| Order desk: Counter payment docked under scrolling cart lines (no Counter/Items chips) | **In (2026-10-27)** | Order desk `pos_ticket` / `counter_dock` |

Sanitized client tracker (must remain covered):
[`KHAYNJO_CLIENTS_REQUIREMENTS.md`](KHAYNJO_CLIENTS_REQUIREMENTS.md).
Competitor comparison (offline tags): [`RESTAURANT_RESEARCH.md`](RESTAURANT_RESEARCH.md).
LAN pilot readiness (both apps): [`PRODUCTION_STATE_FOR_OFFLINE_USAGE.md`](../../../docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md).

## 4d. Petpooja parity — no cloud, more control (client, 2026-10-23)

A restaurant client runs Petpooja and wants its useful daily mechanics
with **no cloud at all** — learned from, done better, designed for
Khaynjo's users (vision §24), not cloned. Evidence: Petpooja deep-dive
[`RESTAURANT_MARKET_RESEARCH.md`](RESTAURANT_MARKET_RESEARCH.md); wider
market comparison [`RESTAURANT_RESEARCH.md`](RESTAURANT_RESEARCH.md).
Owner-control asks beyond Petpooja parity (staff visibility, units
pick-lists, occupy-without-order):
[`KHAYNJO_CLIENTS_REQUIREMENTS.md`](KHAYNJO_CLIENTS_REQUIREMENTS.md).
Kumkaj-wide rules this rests on (not restated): branch = separate
restaurant (vision §5), rights with approval + reason (vision §17),
cancel-not-delete and explicit zero-charge (vision §18), open orders
and shift cash (`BUSINESS_BOOKS.md`), finding the server without the
internet (vision §20).

| Petpooja mechanic | Khaynjo | Where |
|---|---|---|
| KOT → more KOTs → Print bill → Settle & Save | **Done** — running order per table; only new lines go on the next KOT; finalize never re-fires sent lines | `store_running.rs` (`send_kot`, `print_running_bill`), `write_sale` |
| KOT per station / category printer | **Done** — one ticket per station, numbered per day (`KOT No`), with who sent it | `send_kot`, `format_kot_text` |
| Table colours Blank / Running / Printed / Paid | **Done, redesigned 2026-10-23; clarified 2026-10-27** — grid of coloured tiles by area (Layout view optional). Visit colour: Free → **Seated** (no order) → In kitchen → Food ready → **Served** → Bill printed → Free on **payment**. Accidental Print bill → **Undo bill**; Bill printed with ₹0 → **Free table**. **Bugfix 2026-10-27:** pay/free completes leftover open KOTs and the floor only counts KOTs for *this* seating — next guests no longer stick on In kitchen. Summary line and colour key above; refresh every 8 s | `FloorState`, `FloorTile`, Floor; `complete_open_kots_for_table` |
| Kitchen display on a tablet | **Done** — ticket Start/Ready/Done from a paired tablet reaches the Server (it used to change only the tablet's cache); **2026-10-27** ageing colours + Recall | Kitchen; `/kot_advance`, `/kot_recall` |
| Bill format + UPI QR | **Done** — header / FSSAI / footer; **2026-10-27** UPI VPA + counter QR for the bill amount; **Have 2026-10-29** print brand kit (stamp + static payment QR images on bill faces / LAN `/display`; dynamic amount UPI QR still used when no payment-QR image) | Settings → Bill format; Order desk Show UPI QR |
| Areas (AC / Garden / Parcel) | **Done** — area per table, headings on the floor, range add `D1-D20` | Floor, `add_tables_range` |
| Move KOT / items, move table, merge tables | **Done** — same order id moves; minutes travel with the guests | ticket Move / merge; `move_*`, `merge_tables` |
| KOT cancellation with reason, prep status, approver | **Done** — line kept as cancelled, partial qty, cooked = wastage; kitchen gets a ✕ CANCEL slip | ticket Cancel; `cancel_running_line` |
| NC (complimentary) items | **Done** — ₹0 with value kept, reason + approver, on the bill as NC | ticket NC; `set_running_line_complimentary` |
| User rights matrix with "with password" | **Done** — per person: Take payment, Cancel sent item, NC, Move/merge, Print again → Allowed / With approval (manager PIN) / Not allowed; enforced on the Server | Team → Rights; `Store::check_right` |
| Captain can't settle | **Done** — Waiter preset: Take payment = Not allowed | `StaffRights::for_role` |
| Captain app on a tablet, LAN | **Done** — Send KOT from a paired tablet reaches the Server (it used to stay in the tablet's cache); table actions go over `/running`; refusals come back in words | `net.rs` |
| Find Server IP | **Done** — UDP broadcast + /24 scan fallback, lists restaurant names | Connect → Find the shop's server |
| What a restaurant buys | **Done 2026-10-24** — every item has a kind: **dish** (cooked and sold; never bought), **bought & resold** (bottled drinks; on the menu and in stock), **raw material** (rice, paneer, oil, gas; bought and used up; never on the menu or order desk). Buy and Inventory offer raw materials and resold goods only; a new name on Buy (or + Stock) becomes a raw material with its unit. Menu form: "Dish we make / Bought & resold" | `Product::kind`, `stock_items`, `stock_item_or_new` |
| Expense paid several ways | **Done** — one row per way (Cash / UPI / Card / Bank + amount), "+ Add another way", one blank amount takes the rest, live "Entered ₹300 of ₹500 · ₹200 left"; expenses are logged on Buy only (Reports only reports) | `PayPart`, `settle_pay_parts`, `PartsList` |
| Type or pick, everywhere | **Done** — every field that names something the shop has (table, area, station, unit, tax rate, dish group, stock item, supplier, party, staff, branch, country) opens a list on focus and narrows as you type; filled fields keep a small label | `suggest.rs`, `SuggestList`, Makepad patch `text_input_float_caption` |
| Reports on screen | **Done** — day close, books, settings summaries render as headings + label/value rows in a card, never one run-on paragraph. **2026-10-26:** every list that was still one paragraph is a card too — Units (by family, smallest first, then conversions), printers and where things print, printers found, audit, a sale's history, open bills, bank reconciliation, notes & attachments, a party's recent ledger, shift handover, the country pack being browsed, a tablet's waiting queue and servers found; run-on figure lines ("Cash ₹120 · UPI ₹0") split into rows. Right-to-left languages mirror the card (label right, value left) | `ReportText` (`set_text` → `parse_report`; lists built in code → `set_rows`) |
| Import the complete menu template | **Done 2026-10-24** — every catalogue folder becomes a category (with its photo) and every dish a menu item in it (photo referenced as `shelf:<folder>/<file>`, not copied; price ₹0 for the owner to set). Offered at first-run setup ("Start with our menu template") and on Menu ("Import complete template", with a confirm). Dishes the shop already has (same name) are kept; importing twice adds nothing. Big menus stay fast: the menu window's dish grid and the Menu list draw only the rows on screen; the Menu list has "Find in your menu" | `Store::import_catalogue`, `menu_images::shelf_key`, `MENU_LIST_MAX_HEIGHT` |
| Favorites screen | **Removed 2026-10-24** — duplicated the order desk and confused staff | — |
| Order screen: the menu in its own window (owner's request, 2026-10-25) | **Done** — one **+ Add dishes** button opens a window over the order: search (name, alias / short code, dish group; a short code typed exactly comes first), the category tabs (All items + the shop's categories, ‹ › when they overflow) and the dishes as cards, as many per row as fit and **never fewer than two**. **Card (redesigned 2026-10-27):** the dish's photo fills the whole card (cut to the card's shape off the UI thread, `thumbs::ThumbKind::Card`); with no photo, a red card with the initials on a frosted plate. A dark wash at the foot carries the name (up to 2 lines) and the price, always on the same line on every card. A tap anywhere on the card adds one; once the dish is on the order a **− n +** pill shows at the top and the card gets a gold edge (− never takes back what the kitchen already has — that stays a reasoned cancel on the ticket). Sold out: dimmed, "Unavailable" in place of the price, taps add nothing. Rows sit 14 px apart. The window stays open for the next dish and says "On this order: 3 dishes · ₹180" under its title. Typing looks through every category (a tab tapped afterwards narrows the results); Done, Escape, Back or a tap outside closes it and the next opening starts fresh. What is on the order shows only on the ticket (the separate "On this order" tiles were removed: they repeated the ticket) | `menu_modal`, `ProductList`, `visible_products`, `open_menu_window` |
| Billing screen: Dine In / Delivery / Take Away tabs, table button, category list, item tiles | **Done, redesigned 2026-10-23/24** (menu grid moved into the menu window above, 2026-10-25) — one page with one scroll (2026-10-24: two separately scrolling panes were rejected) — menu on top (search, category tabs with ‹ › chevrons, the first 3 rows of tiles + "Show all N dishes"), the order under it (lines + Send KOT left, bill + one-tap payment right). One segmented row Dine-in · Takeaway · Delivery · Counter. **Dine-in with no table opens the floor in "pick a table for this order · N items" mode**; a tap seats the order there with what was already punched and comes back (Back to order leaves it unseated). The ticket's table is a button: "Choose a table" / "D3 · Change table". Category chips (All + the shop's categories in use) above the grid; tiles are the button (tap = +1; since 2026-10-27 photo cards with a − n + pill, see the row above); as many across as fit (min 124 px). The ticket gets 1 : 1.25 of the width (a real bill pane). An empty ticket shows only its hint — no ₹0 totals, pay fields or Clear | `ProductCard`, `ProductList`, `pick_table_for_order`, Floor `floor_pick_bar` |
| Order ticket that stays readable on a busy table | **Done, redesigned 2026-10-23 after looking at it** — one compact row per dish: name · amount, then "2 × ₹250 · Sent to kitchen · note" with − qty + on lines not yet sent. Tap a row to open its price, applied total, add-ons, note and Cancel / NC / Move (one row open at a time). Send KOT sits right under the lines; Print bill, Move / merge and Next order are light outlined buttons, not competing with it | `CartRow`, `CartList`, Order desk |
| Settle: customer paid, return, tip | **Done** | Order desk |
| Search by name / short code | **Done** — aliases double as short codes | Order desk search |
| Bill format: header, GSTIN, FSSAI, footer | **Done** | Settings → Bill format |
| Day end: cancellations, NC, tips | **Done** on the day close text | `day_close_summary` |
| Branches | **Done (doctrine)** — one restaurant per local server; in-book branch switching hidden | vision §5 |
| Sizes (Half/Full) | **Done — each size is an item of its own** (decided 2026-10-23): own UUID, price, photo, station, tax, stock, sold-out, area prices and short code, because each is sold, cooked, cancelled and reported on its own. Quick entry "Half=120, Full=200" on the item form creates "Paneer Tikka (Half)" / "(Full)" with the form's shared settings (not its area prices or short codes, which belong to one size); an optional **dish group** label ("Paneer Tikka") finds and totals them together — a label, never identity. Combos / BOGO / fixed combo **Done 2026-10-27** (components expand on Send KOT; combo price only) | Menu form; `Store::add_variant_items`, `Product::group_name`; day report "By dish group"; Settings → Schemes; `seat_fixed_combo` |
| Area-wise prices (AC dearer) | **Done** — "AC=220, Parcel=210" on the item; a table in that area bills that price, others the item price; the grid shows the open table's price | Menu form; `Product::price_in_area` |
| Menu timings / happy hour | **Done 2026-10-27** — windows on the item (`Mon-Fri 07:00-11:00`, `HappyHour 17:00-19:00=150`); desk uses `price_now` / hides outside visibility windows | Menu form; `Product::schedule_json` |
| Customer UPI QR | **Done 2026-10-27** — counter modal + LAN `GET /display` (order lines, total, UPI QR data-URI when paying) | Order desk; `upi_qr`; `display_order_html` |
| KDS timers + recall | **Done 2026-10-27** — age colours from station target mins; Recall on Recently done; LAN `/kot_recall` | Kitchen; `kot_age_band`, `Store::recall_kot` |
| Discount with reason / right | **Done** — bill discount 10 % or ₹50, reason required, its own right (cashier: with approval; waiter: not allowed), PIN checked on the Server at payment; spread over lines so tax is on what is charged; on the receipt and the day report. Coupons later | Ticket → Discount; `BillDiscount` |
| Opening float + shift handover slip | **Done** — opening cash per day; shift close counts the drawer against float + cash taken + cash tips − cash paid out, names short/over, prints the slip, starts the next shift; day close expects the float too | Day close; `store_service.rs` |
| Item-wise / staff-wise / hourly reports | **Done** on the day close: by staff, by order type, by hour, top items, discounts (plus cancellations, NC, tips) | `restaurant_day_report_text` |
| Split one bill into two | **Done 2026-10-25** — item pick split; **2026-10-27** Split evenly / by seat → companion bills | `split_running_lines`, `split_evenly`, `split_by_seat` |
| Delivery boy assignment, reservations, KDS all-day counts | **Done 2026-10-25** — Delivery by; Floor **Reservations**; Kitchen all-day counts. **2026-10-27** waitlist, token `/tokens`, expo `/expo`, kiosk `/kiosk`, QR `/t/{table}`, Share WhatsApp | `sale_delivery`, `reservations`, `store_ops`, `net` public LAN |
| Aggregators, SMS/WhatsApp campaigns, live multi-outlet Cloud | Aggregators / live multi-outlet **Skipped** (true Cloud). CRM queue + WhatsApp share **Have**. SMS **delivery** Planned: Local SMS Gateway (Android sibling) + paid Cloud SMS — [`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md); never gate counter | `notify`, connectors |

## 4c. Languages & RTL

Khaynjo’s static UI — every label, heading, button, placeholder,
hint, and the setup / lock / onboarding screens — ships fully
translated in the 29 Kumkaj languages (list and doctrine: vision
§23). The dictionary is Khaynjo’s own copy (`src/locale.rs`,
`src/locale_chrome.rs`), diverged from Moklanjo’s; the
`shipped_languages_translate_chrome` test fails if any key would fall
back to English in any shipped language, so new labels land with all
translations. First launch follows the device language; afterwards
the saved per-device choice wins
(`local_servers::{load,save}_app_language`). Guest- and owner-typed
names are never translated. Arabic and Urdu run right-to-left:
Makepad shapes and bidi-orders the script, and
`apply_text_alignment` in `main.rs` right-aligns static text and
input fields (vendor patch: `TextFlow::paragraph_align_x`,
`TextInput::label_align`); the window layout itself is not mirrored.

## 5. Build plan

1. **Phase 0 — Foundations (done).** Copied crate, Khaynjo identity
   (ids, labels, env), locked restaurant template, Restaurant venue,
   `khaynjo.sqlite3`, `cargo khaynjo-dev`, workspace wiring.
   Verified: `cargo test -p kumkaj-khaynjo` green.
2. **Phase 1 — Order, Floor, Kitchen, Bill (done, 2026-10-22).**
   The copied courier face is deleted and the restaurant face
   stands in its place: Order desk (order-type chips, menu grid,
   Actual/Applied/Rounded, pay / credit / split tenders, bill
   number on receipt), Floor (tap table → Order, occupied/free,
   table frees on bill), Kitchen (station-grouped KOTs,
   New → Preparing → Ready → Done, no prices), Menu (items with
   category + kitchen station). Later deepened: allergens/veg marks,
   per-line notes/variants, running-order additions on an occupied
   table (§4d / research §8).
3. **Phase 2 — Floor & kitchen, the service day.** Table map
   (areas, colour states incl. **Payment pending**, elapsed time,
   running amount), tap table → order → Send KOT → additions →
   bill → payment recorded at the counter → table released;
   station routing; **KOT auto-prints at the server on send**
   (printer per station; printing can follow the device that owns
   the printer); kitchen (KDS) list without prices; settle with
   change-return and tip.
   *First cut done 2026-10-22:* `TableStatus::PaymentPending`,
   running total + elapsed time on floor cards, **Request bill**
   on the card (waiter clients reach the server via
   `POST /table_billing`), release gated on payment in
   `Store::free_table`, KOT auto-print on send (host counter +
   server-side for client KOTs; Settings → Kitchen printing
   toggle). Test: `table_payment_pending_releases_only_after_payment`.
   *Running orders + Petpooja parity done 2026-10-23* (§4d): the
   table keeps its order between KOTs, print bill → 🟢, settle with
   change and tip, cancel/NC with reason and approver, move/merge,
   areas and ranges, rights matrix, tablet KOTs reach the Server,
   find-server. Tests: `store_running.rs` (9), `net.rs` (2),
   `floor_tests` (2).
   *Menu catalogue done 2026-10-22:* Item and category identity is
   the **UUID, never the name** — two dishes (or categories) may
   share a name and stay fully independent (price, photo,
   availability, category). *Item picker (2026-10-23):* the item
   form's name field is a search-as-you-type picker over the
   shipped photo shelf (`assets/items/<folder>/<dish>.webp`, 25
   folders, 1459 dishes) and the owner's own menu. It opens the
   moment the field is focused — empty, it browses the whole shelf,
   so an owner who would rather not type (or not type in English)
   just picks; "Pick from our items" toggles it, Escape closes it.
   **Add item / Save sits at the end of the form**, after every
   field and the photo. Rows, in order: the
   typed text itself ("Use this as your own item name" — a custom
   name is always one tap), items already on the menu (name or
   alias match → opens that item to edit, so nothing is duplicated
   by accident), then shelf dishes ranked exact → prefix → word →
   substring → all-words, plus whole shelf folders whose name
   matches (`menu_images::{match_rank, search_shelf}`). Tapping a
   dish copies name + photo **into the form** (nothing is written
   yet) and never sets a price. *Category (2026-10-23):* the dish's
   shelf folder is **suggested, editable** in the Category field — the
   owner's own category of that name when one exists (linked by
   UUID), else the folder's name, created on save with one of the
   folder's dish photos as its photo (only if the category has none).
   A category the owner typed is never replaced by a later pick;
   opening a menu item fills its own category. The Category field has
   the same picker as the name: opens on focus (blank = all), search
   as you type; rows = the typed text as a new category, the owner's
   categories (photo or initials), then shelf folders as suggestions
   (`CategoryPick`, `menu_images::shelf_folders`,
   `Store::set_category_photo_if_missing`).
   Keeping your own name drops a photo that came from a previously
   picked dish, but keeps one the owner picked from their files.
   **₹0 is a valid price** (blank = ₹0): complimentary items and
   dishes priced at the counter by their `@`. Menu form, onboarding
   and till agree — only a negative price is refused
   (`domain::menu_price_error`); a ₹0 line bills, and a bill still
   needs a positive total. The shelf itself is **bundled,
   read-only art**: the app never writes, moves or deletes it, so
   a dish can always be found and added again.
   **Remove is an archive, not a delete** — the Menu screen's
   "Removed items" section lists what was taken off the menu with
   **Restore**, which puts the item back exactly as it was (price,
   photo, category, station) whether it came from the app's shelf
   or was typed by the owner (`Store::restore_product`, audit
   `menu.restore`). "Your categories" lists the owner's
   categories by UUID with a per-category photo; **Use** carries
   the category UUID into the item form (retyping the field
   falls back to find-or-create by name as a typing
   convenience). No photo → initials placeholder (one word:
   first two letters; two+ words: first letters of the first
   two words) for items and categories alike.
4. **Phase 3 — LAN service.** Role setup/pairing on restaurant
   screens (waiter Clients occupy/order/request-billing; counter
   settles and releases), plain sync status on every device,
   offline-then-resync; proven on one server + two clients with
   no duplicate tickets or bills.
   *Done in code 2026-10-24* (design and rules: `docs/DEV.md` § LAN
   Client / Server): tablets keep taking orders through a Wi‑Fi drop
   and send each KOT / payment once ("Offline · N waiting" in the
   header), never freeze waiting on the network, sign in with PINs the
   Server checks, can be forgotten by the owner; the Server stays
   awake, can start with the computer (`--serve`, Windows/Linux), backs
   up daily and opens its firewall from Settings (Windows, Linux
   firewalld/ufw). Any supported device is either role — Windows, Linux
   or Android, in any mix (`docs/LAN_QUICK_START.md` §5 lists what
   differs). Packages: Windows MSI (`apps/khaynjo/scripts/msi.sh`), Linux
   (`package-linux.sh`), Android APK (`package-android.sh`). Two devices
   + the counter writing at the same moment lose and double nothing
   (`two_tablets_and_the_counter_writing_at_once_lose_and_double_nothing`).
   *2026-10-26:* per-device printers (Settings → Printers on this device:
   system printer by name or a network ESC/POS printer by address, test
   page), print failures shown on screen, device clock checked against
   the Server, Android saves backup copies through the system dialog and
   restores from Downloads/USB; two devices of different supported platforms
   served a third device end to end on the emulator (pair, PIN on the Server,
   names synced).
   *2026-10-27:* research Local/LAN catch-up **done** — see
   [`RESTAURANT_RESEARCH.md`](RESTAURANT_RESEARCH.md) §8 (Cloud/Net/owner
   items only in “Still left”).
   *Still to do:* the same run on real hardware, on the devices the shop
   actually has — see §6.
   *Unpriced dishes (2026-10-24):* the template imports dishes at no
   price; Menu says "N dishes on your menu have no price yet" with
   "Show only dishes without a price", and the order desk says so when
   such a dish is tapped.
5. **Phase 4 — Books depth.** Purchases, expenses, stock, party
   ledgers, reports, day close on restaurant wording.
6. **Phase 5 — Control.** Permissions on discount/void/amend,
   verified restore, owner retention window. **Emergency unlock PIN
   done 2026-10-27** (Owner-only, off by default): Team → set a PIN
   that is not anyone's unlock PIN; if typed at unlock, sales/history
   wipe and the till opens as Owner looking normal — menu, prices,
   tables and staff stay.
7. **Phase 6 — India polish, then country packs.** GST split and
   India completeness first; further packs on identical screens.
8. **Research Local/LAN catch-up — done 2026-10-27.** Everything from
   market/research that ships without Kumkaj Cloud is **Have**
   ([`RESTAURANT_RESEARCH.md`](RESTAURANT_RESEARCH.md) §8). Remaining:
   live aggregator SaaS, card-terminal confirmation, mobile-data guest
   order, live multi-outlet Cloud — optional Net stubs never gate the
   counter; real-hardware pilot is owner-only.
9. **Later, only when pulled.** Kumkaj Cloud (WAN sync, remote
   dashboard, hosted backup), live aggregators, cash drawer/scanner.

Platforms: Windows / Linux / Android build, package and pass the tests
now, and each can be the Server or a client.
macOS / iOS stay scaffold-only until a Mac host exists to build,
sign and test — same product when they land.

## 5b. Layout & interaction law for Khaynjo screens (learned 2026-10-22)

Generic layout/interaction law for all Kumkaj apps is canonical in
[`docs/DESIGN.md`](../../../docs/DESIGN.md) § **Always responsive**,
§ **Text selection & copy**, § Work-surface layout & interaction, and
[`docs/DEV.md`](../../../docs/DEV.md) (state-guard and agent-verification
rules) — not restated here. Below is only the restaurant instantiation
the first build broke and must not break again:

- **Order desk stack:** order-type chips and **+ Add dishes** stay
  pinned above; cart lines scroll in the middle; **Counter** (payment)
  is fixed at the foot of the ticket column (part of the same ticket,
  not a separate tab/chip). Always one column.

- **Order context lives on the ticket**: order-type chips (short
  labels only), table label, **Send KOT**, totals, tenders — one
  glance tells the waiter what this bill is.
- **Order type is exactly four choices → 2×2 segmented grid** of Fill
  chips on the ticket (all four visible at any width; a `ChipStrip`
  half-hid chips at 1024 px and a plain `Right` row clipped Delivery).
- Ticket header shows sale state + Clear (no books jargon mid-order);
  the state line is the ticket's own error strip, so it **wraps inside
  the ticket** (`width: Fill`; a `Fit` label pushed Clear past the
  right edge and clipped it to "Clea…" at 1024 px, 2026-10-22).
  An empty menu pane's next action is “Add dishes & items” (→ Menu).
- **A `PortalList` that sits inside a page `ScrollYView` gets a
  `Size::Fixed(list_content_height(...))` walk, never `Fit`/measured** —
  with `Fit` it collapses to 0 height and the whole list disappears:
  "nothing to scroll" (2026-10-22). Menu screen lists: CategoryRow 82 /
  MenuCard 160, plus the list's own `spacing`.
- **…but a content-height list draws every row.** Fine for the menu's
  own items; fatal for anything long or photo-heavy. The old shelf drew a
  whole folder (~100 dishes) and decoded every full-size photo
  synchronously in one frame — ~250 ms each in a dev build, so the Menu
  screen, the Order desk and anything showing item photos froze
  (2026-10-23). Long lists get their **own capped viewport** that scrolls
  inside the page (`DishPickList`, max 360 px, claims the wheel like
  Settings' lists), and every row photo goes through `thumbs` (192 px,
  made on worker threads, initials meanwhile) — never
  `load_image_file_by_path` on the original from a `draw_walk`.
- Item form errors and confirmations paint on the Menu screen's own
  status line (`menu_status`) — it had none, so "Name required" /
  "Set a price …" failed silently.
- The whole **table/KOT card is the tap target**; KOT summary on the
  card is capped at ≤ 140 chars (full text stays on the record).
- **A sent line has no ± on the ticket** — it changes only by Cancel
  (reasoned), NC or Move, in the ticket's action panel; a tap on the
  same dish starts a new line for the next KOT (2026-10-23).
- **LAN surface (Server, port 7847 + UDP 7848):** `/kot` (send KOT,
  lines join the table's running order), `/running` (cancel / NC /
  move / merge / print bill — rights checked on the Server, refusals
  return `{"error": …}`), `/hello` (open: app + restaurant name, for
  the scan), UDP `KHAYNJO_DISCOVER?` broadcast → `{app, venue, port}`.
  Snapshots carry `running_orders`.
- Floor/Kitchen peers show only when the venue runs tables & kitchen
  (`creates_kot()`); unlock lands on Day book, Order/Floor one tap away.
- **Floor grid columns from the list’s turtle width** — never from
  shell `content_width` alone (that ignores Floor screen padding and
  crops the last tile). Same pattern as the dish grid.
- **Display text is selectable** (`WrapLabel` / `ReportText`); only
  tap targets keep `selectable: false` / plain `Label` (DESIGN § Text
  selection).
- Side rail (≥720): **‹** collapses to zero width, **›** in the header
  restores (`rail_collapsed`); bottom tabs unchanged.
- Khaynjo's state-guard check: `apps/khaynjo/scripts/check-lock-guards.sh`
  (root rule: `docs/DEV.md` § Makepad 2 UI faces).

## 6. Acceptance checks per phase

- `apps/khaynjo/scripts/check-tests-never-print.sh` clean — the suite
  never reaches the OS printer (`docs/DEV.md` § Smoke tests).
- `cargo test -p kumkaj-khaynjo` green (engine tests plus
  Khaynjo-specific ones, e.g. `dine_in_sale_assigns_bill_no_table_and_receipt`).
- **No courier vocabulary in Khaynjo UI or strings** — no AWB,
  consignee/consignor, carrier, manifest, docket, zones, booking,
  riders, **gift wrap, fragile** or any other parcel-trade word —
  including `empty_text` placeholders and test fixtures (a dish
  form once suggested “Gift wrap=10”; grep `apps/khaynjo/src`
  incl. placeholders to prove it; the only tolerated hits are
  legacy DB columns in pre-existing dev databases, the
  `parcel` → Takeaway parse alias for old rows, and **"Parcel" as a
  restaurant's own floor-area name** — Indian restaurants say parcel
  for takeaway, and the owner names their areas (research §3)).
## Petpooja-shaped gaps (local / LAN / optional Net — no Cloud required)

Compared with Petpooja POSS marketing (menu variations/add-ons, KOT,
tables, inventory, CRM, reports). **Have** means Khaynjo already covers
the job locally; **Next** is worth building without Cloud; **Skip** needs
true multi-outlet Cloud or a third-party marketplace.

| Capability | Khaynjo | Note |
|---|---|---|
| Item variations + priced add-ons | **Have 2026-10-27** | Groups on the item; also separate-item sizes |
| Combos / BOGO | **Have** | Settings → Schemes |
| Station KOT / KDS / tables / areas | **Have** | |
| Multi-terminal LAN | **Have** | Server + paired Clients |
| Recipe stock, low-stock, purchases | **Have** (depth varies) | |
| Day close + item/staff/hour reports | **Have** | Fewer named reports than Petpooja’s 94 |
| Loyalty points on customer | **Have 2026-10-27** | Earn on pay; Order till redeem points / wallet / voucher; top-up & issue voucher |
| QR / kiosk / token / waitlist | **Have** (LAN pages) | |
| Aggregator sync / head-office menu push | **Skip / Partial** | **Have 2026-10-29** Local file import UX; live Zomato/Swiggy + HO push = Cloud |
| CRM segments + birthday SMS | **Have** queue 2026-10-27; **Planned** delivery | Parties → Guests & messages queues SMS/WhatsApp intents. Delivery: Local SMS Gateway (cloudless Android sibling) or paid Cloud SMS — [`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md) |
| Variation / add-on sales reports | **Have 2026-10-27** | Day report section from `sale_lines.modifiers_json` |
| Modifier max (e.g. 3 toppings) in UI | **Have 2026-10-27** | `Toppings+3:` in Options field |
| Captain upsell AI / Purchase Manager AI | **Skip** | Cloud / vendor marketplace |

- Lunch-service acceptance: fresh install → add menu items → Floor
  `D1-D20` (Dine In) + `P1-P20` (Parcel) → tap D3 → add dishes →
  Send KOT → leave the table, come back: the dishes are still on it →
  add one more → Send KOT (only the new dish prints) → Cancel one
  sent dish as a cashier (asks a manager PIN; kitchen gets ✕ CANCEL)
  → NC the chaas → Print bill (D3 turns 🟢) → Customer paid 500 →
  Return to customer shown → Take payment split cash/UPI → table
  frees — without the owner ever seeing a courier word.
- Extra-device acceptance (any client: Android, Windows or Linux) →
  Client Mode → Find the shop's server → pair → as Waiter, open a
  table, Send KOT → it prints at the Server; Take payment is refused
  for the waiter.
- Manual UI smoke on real hardware from Phase 3 onward, with the Server
  and clients on whichever supported devices the shop uses; before a
  release, at least one Server of each kind (Windows, Linux, Android)
  with a client of another kind.
- Sell/KOT path stays immediate — operational speed over decoration.
