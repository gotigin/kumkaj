# Khaynjo — Restaurant Operating System

> **Brand / domain:** Khaynjo · khaynjo.com ("Khaynjo" = food in Memoni)
> **What ships:** the Kumkaj restaurant product — a concrete business
> operating system for restaurants and like food businesses, under the
> Kumkaj brand alongside Kumkaj Courier.
> **Platform ids:** production `com.khaynjo` · development
> `com.khaynjo.debug`
> **Naming:** Khaynjo belongs to the Kumkaj brand, but its domain is
> its own — **khaynjo.com**, not a path under kumkaj.com. Courier's id
> is `com.kumkaj.courier` (kumkaj.com + product); Khaynjo's id root is
> simply `com.khaynjo` (its own domain; the product *is* the brand),
> with `.debug` for dev exactly like Courier's split. Data dirs,
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

## 3. What was copied from Courier (do not reinvent)

Khaynjo's crate is a copy of `apps/courier`, renamed. Already working
and covered by Courier's hard-won fixes + tests:

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
without touching Courier. When a future product needs the same logic,
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
  fiscal packs, staff/roles, shell — copied from Courier and now free
  to diverge; it is not shared with Courier.
- Owner logo is in place (`assets/khaynjo-app-icon.png`); the full
  icon set and in-app marks are generated from it, and the shell
  theme is sampled from its colors (deep red `#9A0E10`, maroon
  `#69090C`, red-orange `#D64D21`, gold `#F69D2C`, warm cream
  `#F9F4F0` — `docs/DESIGN.md` § Color tokens — Khaynjo).
- Image upload follows Courier's idea, copied with the crate:
  menu-item/party photos are user-picked files re-encoded to WebP
  (`src/media.rs`), with name-initials fallback and no built-in art
  catalogue (`src/menu_images.rs`).

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
| Table floor: tap table → order, table frees when its bill is paid | **In (P1)** list-style floor; areas/shapes later | Floor screen |
| Table cards that show elapsed time and running amount at a glance | **In (P2)** — the counter reads the floor without opening a table | Floor screen |
| Table states in colour, one shared truth on every device: Free → Running → KOT sent → Bill printed → **Payment pending** → Paid/Released | **In (P2)** — payment pending is its own state; only the counter releases a table, and only once payment is recorded | Floor + server sync |
| Waiters take orders at the table on their own devices | **In (P2)** — waiters are paired Clients; they occupy tables, add items, send KOTs and request billing. Settling money and releasing tables stay counter capabilities | LAN service, role gates |
| KOT prints itself the moment an order is sent — no one carries paper to the kitchen | **In (P2)** — the server prints on send; a printer per kitchen station, copies configurable in Settings; printing can also follow the device that owns the printer | LAN service, print settings |
| KOT carries the waiter's name, table, item notes — and never prices | **In (P1/P2)** | KOT record + Kitchen |
| Kitchen display with ticket states New → Preparing → Ready → Completed, no prices (vision §9) | **In (P1)** | Kitchen screen |
| Variation / add-on choices and per-item special notes at order time (extra cheese, less spicy…) | **In (P1)** modifiers; priced variant matrix later | Order desk dialogs |
| Hold an order and come back to it | **In (P1)** via open orders on tables / counter desk state | Order desk, Floor |
| Settle with tender, amount handed over, change to return, and tip | **In (P2)** on the settle step | Order desk settle |
| Part payment and Due (credit) as normal ways to finish a bill | **In (P1)** — Due lands on the customer's khata in Parties, never a side note | Pay mix; Parties |
| Customer quick-pick at billing (name/phone) so Due and history attach to the right person | **In (P1)** | Order desk customer panel |
| Split tenders (cash + UPI + card on one bill) | **In (P1)**; split bill / merge tables later | Pay mix; Floor later |
| Devices always show whether everything is synced | **In (P2)** — one plain status line, on every device | Chrome; LAN service |
| Orders and kitchen keep working with no internet, all day | **In by design** — local-first; the shop server on LAN is the hub | §2, P2/P3 |
| KOT with station/category routing | **In (P1)** — `station_id` per menu item, tickets grouped per station | Send KOT, Kitchen |
| Day close the owner trusts: tender totals, credit given/collected, item sales, profit view | **In** via Khaynjo's own books; depth over report count | Day book, Reports |
| Recipe-level stock deduction, low-stock/expiry alerts, purchase price comparison across suppliers | **Later** (inventory depth) on Khaynjo's own stock code | Phase 4 depth |
| Floor-plan editor: areas, table shapes, seats/covers, coursing | **Later** polish on the P1 floor | Floor v2 |
| Kitchen expediting: all-day item counts, fire/hold courses | **Later** polish on the P1 ticket list | Kitchen v2 |
| Online delivery channels (aggregators) with payout reconciliation | **Later, only when pulled** — must never gate counter billing | §5 Phase “later” |
| Guests ordering from a QR at the table | **Later** — needs the LAN/web surface proven first | After P3 |
| The shop's own language, all day, on every screen | Locale engine already ships 25+; translations deepen per language over time | `locale*.rs` |

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
   category + kitchen station). Still to deepen here: veg marks,
   per-line notes/variants on the desk, additions on an occupied
   table reusing its open order.
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
   Still open here: floor areas/sections, settle change-return/tip.
   *Menu catalogue done 2026-10-22:* Item and category identity is
   the **UUID, never the name** — two dishes (or categories) may
   share a name and stay fully independent (price, photo,
   availability, category). Menu → "Pick from our items" browses
   the shipped photo shelf (`assets/items/<category>/<dish>.webp`,
   25 folders, 1459 dishes): **Use** copies name + photo into a
   brand-new item with a fresh UUID; the shelf folder is only a
   way to find dishes and is never forced as the item's category
   — the owner's category is always their own choice. Picking
   the same dish again makes another independent item (that is
   also how one dish appears under two categories: add it twice,
   once per category). "Your categories" lists the owner's
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
5. **Phase 4 — Books depth.** Purchases, expenses, stock, party
   ledgers, reports, day close on restaurant wording.
6. **Phase 5 — Control.** Permissions on discount/void/amend,
   verified restore, owner retention window, optional emergency
   wipe password (owner-only, off by default).
7. **Phase 6 — India polish, then country packs.** GST split and
   India completeness first; further packs on identical screens.
8. **Later, only when pulled.** Kumkaj Cloud (WAN sync, remote
   dashboard, hosted backup), aggregators, cash drawer/scanner.

Platforms: Windows / Android / Linux ship and are tested now.
macOS / iOS stay scaffold-only until a Mac host exists to build,
sign and test — same product when they land.

## 5b. Layout & interaction law for Khaynjo screens (learned 2026-10-22)

Generic layout/interaction law for all Kumkaj apps is canonical in
[`docs/DESIGN.md`](../../../docs/DESIGN.md) § Work-surface layout &
interaction and [`docs/DEV.md`](../../../docs/DEV.md) (state-guard and
agent-verification rules) — not restated here. Below is only the
restaurant instantiation the first build broke and must not break
again:

- **Menu grid : ticket ≈ 1.6 : 1** side-by-side (content ≥ 640),
  stacked below that. The menu grid is the work surface; the ticket
  stays lean.
- **Order context lives on the ticket**: order-type chips (short
  labels only), table label, **Send KOT**, totals, tenders — one
  glance tells the waiter what this bill is.
- **Order type is exactly four choices → 2×2 segmented grid** of Fill
  chips on the ticket (all four visible at any width; a `ChipStrip`
  half-hid chips at 1024 px and a plain `Right` row clipped Delivery).
- Ticket header shows sale state + Clear (no books jargon mid-order);
  an empty menu pane's next action is “Add dishes & items” (→ Menu).
- The whole **table/KOT card is the tap target**; KOT summary on the
  card is capped at ≤ 140 chars (full text stays on the record).
- Floor/Kitchen peers show only when the venue runs tables & kitchen
  (`creates_kot()`); unlock lands on Day book, Order/Floor one tap away.
- Khaynjo's state-guard check: `apps/khaynjo/scripts/check-lock-guards.sh`
  (root rule: `docs/DEV.md` § Makepad 2 UI faces).

## 6. Acceptance checks per phase

- `cargo test -p kumkaj-khaynjo` green (engine tests plus
  Khaynjo-specific ones, e.g. `dine_in_sale_assigns_bill_no_table_and_receipt`).
- **No courier vocabulary in Khaynjo UI or strings** — no AWB,
  consignee/consignor, carrier, manifest, docket, zones, booking,
  riders, **gift wrap, fragile** or any other parcel-trade word —
  including `empty_text` placeholders and test fixtures (a dish
  form once suggested “Gift wrap=10”; grep `apps/khaynjo/src`
  incl. placeholders to prove it; the only tolerated hits are
  legacy DB columns in pre-existing dev databases and the
  `parcel` → Takeaway parse alias for old rows).
- Lunch-service acceptance: fresh install → add menu items → Floor
  tap table → add dishes → Send KOT → Kitchen Start/Ready/Done →
  Take payment split cash/UPI → table frees — without the owner
  ever seeing a courier word.
- Manual UI smoke on real hardware from Phase 3 onward
  (Windows server + Android client at minimum).
- Sell/KOT path stays immediate — operational speed over decoration.
