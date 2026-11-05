# khaynjo.com — website plan

> **Status:** plan, 2026-11-05. Nothing built yet.
> **Scope:** Khaynjo's instantiation of the Kumkaj-wide website plan
> [`docs/WEBSITES.md`](../../../docs/WEBSITES.md) — architecture
> (HTML first + Makepad stage), agent pipeline, budgets, hosting and
> roadmap live there and are not restated here. This file holds only
> what is Khaynjo's: audience, pages, the 3D story, tools and the
> claim sources.
> **Code:** `apps/khaynjo/web/` (Zola) and `apps/khaynjo/web/stage/`
> (Makepad stage crate). No code shared with any other site or app.

------------------------------------------------------------------------

## 1. Who it is for, and the promise

**Visitors:** owners and managers of dine-in restaurants, QSRs, cafés,
counter shops and bakeries in India first; captains/waiters and
kitchen leads who will use it; owners already on a cloud POS who want
the same daily mechanics with no internet and more control
([`RESTAURANT_MARKET_RESEARCH.md`](RESTAURANT_MARKET_RESEARCH.md) §1).

**Promise (headline candidates, English):**

- "Lunch rush. No internet. No problem."
- "From table to kitchen to bill — on your own devices."

**Main actions:** **Download free** (Android · Windows · Linux) and
**WhatsApp for a demo** (prefilled: "Khaynjo demo — restaurant type,
tables, city").

**What these visitors check first** (from the client's own service
video, research §3): does a table show time and amount at a glance;
does the KOT print in the kitchen without anyone carrying it; can the
waiter's tablet order but not take money; does settle handle change,
tip, part payment and Due; does it keep going when the internet drops.
The home story answers them in that order.

------------------------------------------------------------------------

## 2. Pages

| Path | Content | Notes |
|---|---|---|
| `/` | The service story (§3) | Stage: live / clip / still |
| `/features` | One page, anchored: Order desk · Floor & tables · Kitchen & KOT · Menu · Billing & settle · Day end & shift · Stock & purchases · Team & rights · Reports · Devices & printers · Languages | Split when a section outgrows the page |
| `/for/dine-in` | Areas, running tables, captains on tablets, coursing, split by seat, reservations & waitlist | |
| `/for/quick-service` | Counter, takeaway, token display, kiosk on the shop Wi-Fi, combos, happy-hour prices — QSR, café, bakery | |
| `/switch` | "Moving from a cloud POS": the mechanics that stay the same (running tables, KOT per station, settle with change and tip, rights "with password"), what changes (no internet needed, no subscription), how the menu comes over | Names no other product |
| `/tools/gst-bill-calculator` | §4 | Search magnet; works with no JS |
| `/download` | Android (Play when listed, APK), Windows MSI, Linux; SHA-256; system needs; first-run steps | Reads `releases.toml` |
| `/guide` | Getting started: first menu, tables, a waiter tablet, first service, day close | Screenshots from the real app |
| `/pricing` | Free forever on your devices; Kumkaj Cloud planned, usage-based | Links kumkaj.com/principles |
| `/present` | Presenter mode, works offline after one visit | `WEBSITES.md` §6 step 8 |
| `/privacy`, `/terms`, `/contact` | Legal + WhatsApp / email | |

No `/for/cloud-kitchen` page until aggregator orders are more than a
file import (§6).

------------------------------------------------------------------------

## 3. Home story — "One table, one service"

**Stage:** a warm, isometric restaurant in Khaynjo colours — cream
walls, deep-red accents, gold light — one dining area, a parcel shelf,
a counter and the kitchen pass. The cloche from the app icon is the
recurring object. The room is illustration; **every device screen in
the scene shows a real capture of Khaynjo**, never mock UI
(`WEBSITES.md` §6 step 2).

The chapters walk a table through the app's real visit colours
(Free → Seated → In kitchen → Food ready → Served → Bill printed →
Free on payment) — the visitor learns the product's model by watching
it.

| # | Chapter | On stage | Says (draft) | Claim ids |
|---|---|---|---|---|
| 1 | **Lunch rush** (hero) | Camera glides over a full room. Each table glows in its visit colour with minutes and running amount floating above it (labels pinned to tables). **Toy:** tap a table to step it through its colours (HTML fallback: the same steps as buttons) | "Every table, its time and its bill — at a glance." | `floor-states`, `table-cards` |
| 2 | **Seated, order taken** | A captain's tablet: the table turns Seated; dishes added; a size picked, toppings added; "less spicy" typed as a note | "Take the order at the table. Sizes, add-ons and notes, right there." | `seated`, `variations`, `captain-tablet` |
| 3 | **The KOT prints itself** | Send → the order travels along the shop Wi-Fi line to the kitchen; tickets print at the right station (Hot, Cold) — waiter's name, table, notes, **no prices**; dish names printed in Gujarati | "Send, and the kitchen already has it. In your kitchen's language." | `kot-station`, `kot-autoprint`, `kot-no-prices`, `kot-script` |
| 4 | **Kitchen in step** | Kitchen screen cards move New → Preparing → Ready; older tickets warm in colour; "To cook now" totals; mains held until starters go | "The kitchen sees what to cook now — and what can wait." | `kds`, `all-day-counts`, `coursing` |
| 5 | **Service changes** | Guests move: the table's order slides to another table, two tables merge; an item cancelled after the KOT prints a cancel slip in the kitchen with a reason; a complimentary dish stays on the bill at ₹0 | "Move, merge, cancel or comp — never silently. Who, when and why is kept." | `move-merge`, `kot-cancel`, `nc` |
| 6 | **Settle** | Bill printed (table turns Bill printed). Split by seat into two bills; one pays cash + UPI (QR with the amount baked in), change to return shown, tip kept apart; the other goes on the customer's khata as Due. Only then the table turns Free | "Cash, UPI, part, Due, tip — and the table frees only when it's paid." | `split`, `settle`, `upi-qr`, `due-khata`, `free-on-pay` |
| 7 | **Close the day** | The drawer counts against float + cash taken + cash tips − cash paid out; short/over named; shift slip prints; the day report opens by staff, hour and dish | "Day end you can trust: expected, counted, and the difference named." | `shift-close`, `day-report` |
| 8 | **Your menu, your way** | Menu cards fan out with Gujarati names and English aliases; a short code typed brings a dish first; the AC room bills a dearer price; a happy-hour price switches on at five | "Your menu in your words — with short codes, area prices and timed prices." | `aliases-short-codes`, `area-prices`, `menu-timings`, `languages` |
| 9 | **Internet down? Service goes on.** | The router turns red; the counter, captains' tablets and kitchen screen keep working together on the shop Wi-Fi | "No internet needed. Your tablets and kitchen screens work on your own Wi-Fi." | `offline-lan` |
| 10 | **Close** | The room at rest, cloche on the pass | "Free forever on your devices." → Download · WhatsApp | `free-local` |

**Props built in code** (`WEBSITES.md` §6 step 2): tables and chairs,
plates and cloche, kitchen pass, thermal ticket printer + paper
ticket, bill printer, tablets, kitchen screen, cash drawer, UPI QR
stand, router with status light, menu cards. glTF only for diners,
captains and cooks (simple, faceless, stylised).

------------------------------------------------------------------------

## 4. GST bill calculator (`/tools/gst-bill-calculator`)

Mirrors the app's two tax modes (`docs/README.md` "Sell cart totals";
vision §23):

- **HTML form first:** amount, tax rate (presets 5 / 12 / 18 % and a
  free field — the page says which rate applies is for the owner's
  accountant, it does not advise), mode **Tax inside price** or **Tax
  added at till**, intra-state (CGST + SGST halves) or inter-state
  (IGST).
- Output: taxable value, each tax line, total, rounding — with one
  line of working.
- Runs entirely in the browser; nothing is sent.
- Rounding and split rules re-implemented in a few lines of JS for the
  page (copy, not import) with tests that pin the app's own worked
  examples from `src/fiscal.rs`.
- Ends with "Khaynjo does this on every bill" → Download.

------------------------------------------------------------------------

## 5. Claim sources (seed for `content/claims.toml`)

Sources are rows in [`KHAYNJO.md`](KHAYNJO.md) §4b (marked **Have /
Done / In <date>**) and §4d (**Done**) unless noted. `gate` marks a
claim held back until the owner's real-hardware pilot passes
(`docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md` §3).

| Claim id | Sentence on the site | Source | Gate |
|---|---|---|---|
| `floor-states` | Visit colours every device agrees on | §4b "Table states in colour…"; §4d "Table colours" | — |
| `table-cards` | Minutes and running amount on each table | §4b "Table cards that show elapsed time…" | — |
| `seated` | Mark a table Seated before any dish | §4b "Mark a table Occupied…" | — |
| `variations` | Sizes (pick one) and add-ons (optional, with a max), notes per dish | §4b "Variation / add-on choices…" | — |
| `captain-tablet` | Captains order on tablets; payment is a right they don't have by default | §4d "Captain app on a tablet, LAN", "Captain can't settle" | `lan-pilot` |
| `kot-station` | One KOT per kitchen station, numbered per day, with who sent it | §4d "KOT per station / category printer" | — |
| `kot-autoprint` | The KOT prints in the kitchen the moment it's sent | §5 Phase 2 "KOT auto-print on send" | `lan-pilot` |
| `kot-no-prices` | KOTs carry table, waiter and notes — never prices | §4b "KOT carries the waiter's name…"; vision §9 | — |
| `kot-script` | Tickets print in Gujarati, Hindi, Arabic… | `docs/DEV.md` print table (ESC/POS raster); `src/receipt_raster.rs` | `lan-pilot` |
| `kds` | Kitchen screen New → Preparing → Ready, ageing colours, recall | §4d "Kitchen display on a tablet", "KDS timers + recall" | — |
| `all-day-counts` | "To cook now" totals | §4b "Kitchen expediting…" | — |
| `coursing` | Hold and fire courses | §4b "Kitchen expediting…"; research §6 | — |
| `move-merge` | Move items, move or merge tables | §4d "Move KOT / items, move table, merge tables" | — |
| `kot-cancel` | Cancel after KOT with reason, wastage and approver; cancel slip to kitchen | §4d "KOT cancellation…" | — |
| `nc` | Complimentary at ₹0 with reason and approver | §4d "NC (No Charge / complimentary) items" | — |
| `split` | Split a bill by item, by seat or evenly | §4d "Split one bill into two" | — |
| `settle` | Customer paid, change to return, tip apart, part payment | §4b "Settle with tender…"; §4d "Settle: customer paid, return, tip" | — |
| `upi-qr` | UPI QR with the bill amount | §4d "Customer UPI QR" | — |
| `due-khata` | Due goes on the customer's khata | §4b "Part payment and Due (credit)…" | — |
| `free-on-pay` | A table frees only when its bill is paid | §4b "Table states…" (`free_table` refuses) | — |
| `shift-close` | Float, expected vs counted, short/over named, shift slip | §4d "Opening float + shift handover slip" | — |
| `day-report` | Day report by staff, order type, hour, dish; cancels, NC, tips | §4d "Item-wise / staff-wise / hourly reports", "Day end: cancellations, NC, tips" | — |
| `aliases-short-codes` | Search by name or short code; aliases in another language | §4d "Search by name / short code" | — |
| `area-prices` | A dearer price in the AC room | §4d "Area-wise prices" | — |
| `menu-timings` | Breakfast menus and happy-hour prices | §4d "Menu timings / happy hour" | — |
| `rights` | Per-person rights: allowed / with manager PIN / not allowed | §4d "User rights matrix…" | — |
| `languages` | 29 languages, Arabic/Urdu right-to-left | §4c | — |
| `offline-lan` | Works with no internet; devices work together on the shop Wi-Fi | §4b "Orders and kitchen keep working with no internet"; vision §6, §20 | `lan-pilot` for the multi-device half |
| `free-local` | Free forever on your devices | vision §21 | — |

Until `lan-pilot` passes, chapters 3 and 9 run with the gated
sentences removed (single-device wording only); the claims check
enforces it.

------------------------------------------------------------------------

## 6. Do not say / do not show (until the source changes)

- **The client's name, menu, prices, staff names or photos** (the
  Rajdhani template and Rajdhani App Template are a client's private
  setup) — only with written consent, and never staff names or PINs.
- **Dish photos from the app's shelf** (`assets/items/`) on the site
  until their licence for web publishing is confirmed (owner).
- **A drawn floor-plan editor** — removed from Floor; the face is a
  colour grid by area. The 3D room is illustration, the screens show
  the grid.
- **Live delivery-aggregator integration** — today a file import;
  live SaaS skipped (needs Cloud). Never name aggregators.
- **Guest QR ordering over mobile data** — QR, kiosk and token pages
  work on the restaurant's Wi-Fi only.
- **SMS delivery, campaigns** — SMS delivery planned
  (`docs/SMS_MESSAGING.md`); WhatsApp share ships.
- **Live multi-outlet dashboard** — cloud, later; branches compare by
  exported snapshots today.
- **GST filing, e-invoice IRN/QR** — never; the calculator and bills do
  arithmetic and wording only.
- **Coupons, priced variation matrix** — later.
- **Hardware specifics** (printer models, "tested with …") — after the
  real-hardware pilot, and then only what was tested.
- **iOS / macOS** — deferred.
- **Competitor names** — none (`WEBSITES.md` §12 item 6); `/switch`
  describes mechanics, not brands.
