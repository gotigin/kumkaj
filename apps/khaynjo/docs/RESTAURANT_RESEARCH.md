# Restaurant management software — market research and Khaynjo comparison

> **Collected:** October 2026 (web research: vendor help centres, G2 /
> Capterra / GetApp / Software Advice listings, editorial reviews), plus the
> earlier Petpooja deep-dive from 450 screenshots and the Rajdhani client video
> ([`RESTAURANT_MARKET_RESEARCH.md`](RESTAURANT_MARKET_RESEARCH.md) —
> not repeated here).
> **Our app:** **Khaynjo** — the Kumkaj restaurant operating system
> ([`KHAYNJO.md`](KHAYNJO.md)).
> **Rule we compare against:** Khaynjo is **offline-first and free to run
> locally** (vision §6, §21). A competitor feature is worth building when it
> works **without the cloud** and is genuinely good for the restaurant;
> internet features are optional connectors; hosted features belong to opt-in
> Kumkaj Cloud and never gate service.
>
> **Catch-up status (2026-10-27):** every Local/LAN item from the earlier
> behind list is **Have** — see §8. **Still left** there is only Cloud/Net
> connectors (optional stubs) and owner actions (real-hardware pilot).

---

## 1. How to read this

**Caveats**

- Many comparison pages are written by vendors or affiliates. We take
  **mechanics** from them, never rankings.
- Ratings are quoted with their source and sample size; small samples are
  directional only.
- Prices are October 2026 listing figures, kept only to show the pricing
  *model* (subscription per terminal, add-on modules, payment-processing
  lock-in).

**Offline tags**

| Tag | Meaning |
|---|---|
| **Local** | Works on one device with no network |
| **LAN** | Needs only the restaurant's own Wi‑Fi between devices (Khaynjo Server + captain tablets, kitchen screens) |
| **Net** | Needs internet at that moment (aggregators, WhatsApp, card authorisation) but no Kumkaj cloud; queue offline where possible |
| **Cloud** | Needs a hosted service (vendor cloud, or opt-in Kumkaj Cloud, usage-billed per vision §21) |

**Status tags for Khaynjo:** **Have** · **Partial** · **Gap**.

---

## 2. The market at a glance

| Group | Products studied | Typical shape |
|---|---|---|
| **India, full restaurant suites** | Petpooja, Restroworks (formerly Posist), GoFrugal ServeEasy, eZee Optimus | Windows/Android billing + captain app + KDS + inventory; cloud dashboard; desktop billing that survives an outage |
| **India, ordering-first** | DotPe, LimeTray | QR ordering, own website/app ordering, CRM; POS secondary; weak offline |
| **Global cloud POS (US/EU/AU)** | Toast, Square for Restaurants, Lightspeed Restaurant (K-Series), TouchBistro, Clover, SpotOn, Revel, Lavu | Tablets/handhelds, card-payment-led business model, add-on modules, contracts |
| **Enterprise / hotel** | Oracle Simphony | Powerful, complex back office, strong offline |
| **Free / open source** | Loyverse, Odoo POS Restaurant, Floreant POS | Low cost; varying depth; Odoo and Floreant can run on-premise |

**The common pattern:** every global product is **cloud-first** and offers
an **"offline mode" for outages** with restrictions (§4). Indian suites keep a
desktop billing core that works offline but put the owner's dashboard, menu
push and reports in the cloud. Nobody in this list is designed to run a whole
restaurant — several devices, kitchen, books — **permanently without
internet and for free**. That is Khaynjo's position.

---

## 3. Product profiles

### India

#### 3.1 Petpooja
- **Features** (full module map in the earlier research §4): Dine-in /
  Delivery / Pick-up billing with categories + item grid + short codes;
  variations and add-ons; split, complimentary, BOGO; table view with
  colour states, minutes and amount; move/merge; KOT per station with cancel
  reasons; KDS with all-day counts; captain app with "find server IP";
  settle with tendered/return/tip; day end and shift handover; user rights
  with "with password" level; inventory with recipes, low-stock and expiry
  alerts; **table reservations**; **Scan & Order QR**; CRM, loyalty,
  virtual wallet; schemes and offers; 40–100+ reports (sources disagree);
  aggregator integration; app marketplace add-ons.
- **GUI / UX:** GetApp ease of use **4.6/5** (37 reviews); interface and
  captain app praised. Complaints: **mobile app lacks features the desktop
  has**, few open APIs, **menu management slow and manual** (older review),
  service quality dropped for one 6-outlet owner (2024).
- **Offline:** desktop billing continues offline; dashboard, sync codes,
  menu push and campaigns are cloud.

#### 3.2 Restroworks (formerly Posist)
- **For:** chains and multi-outlet brands.
- **Features:** billing, **central kitchen management**, recipe management,
  inventory, reporting and analytics, QR ordering; listing says it works
  **online and offline**.
- **GUI / UX:** aggregator ratings 4.8–4.9; support praised by multi-outlet
  users; complaints about **sync issues** and **printing problems**; one user
  must **switch to offline billing manually** when internet is slow; wants OCR
  for inventory bills.
- **Pricing:** from about US$200, per user.

#### 3.3 GoFrugal ServeEasy
- **Features:** billing, inventory, recipe and menu management, contactless
  dining, delivery, accounting, loyalty; sold as **on-premise, standalone,
  mobile and cloud**.
- **GUI / UX:** GoFrugal POS overall Capterra 4.4/5 (198 reviews), support
  3.9; easy to use; ServeEasy-specific reviews are scarce.
- **Offline:** on-premise options; data sync for offline access.

#### 3.4 eZee Optimus (Gujarat, India)
- **Features:** restaurant POS, inventory, **store requisition** (kitchen
  requests stock from store), reports.
- **GUI / UX:** Capterra **4.7/5** (53 reviews); interface easy; **slow
  deployment** turnaround. Per-user pricing.

#### 3.5 DotPe
- **Features:** in-store billing, **QR-based ordering**, delivery
  integrations, customer engagement; pitched as reducing dependence on
  aggregators (NRAI-backed).
- **Offline:** listed as limited offline mode (generic listing). Few reviews.

#### 3.6 LimeTray
- **Features:** restaurant's **own ordering website and app** (builder),
  online ordering with payment gateway, POS linked to aggregators, **CRM,
  loyalty and feedback**, inventory, cloud kitchen tools.
- **GUI / UX:** Capterra 4.1 (9 reviews), ease of use 4.4; old reviews
  mention lag "from the server end".

### Global

#### 3.7 Toast
- **Features:** order screen with menu groups; **item routing to prep
  stations, printers and KDS** (with routing preview); handhelds (Toast Go)
  for order and **pay at table**; menu and price management across
  locations; inventory deduction with low-stock alerts; loyalty, gift cards,
  online ordering; payroll and back office.
- **Offline mode (best documented in the market):** switches to offline
  after about **40 seconds**, with a yellow banner on each device; staff keep
  **taking orders and card payments**, and **tickets still reach printers and
  KDS**. Until reconnection they **cannot close checks, declare cash tips,
  reconcile tips or clock out**; reports exclude offline activity; online
  ordering, mobile order & pay, gift cards and loyalty stop.
- **GUI / UX:** praised as an integrated front-and-back-of-house platform
  that "runs smoothly once configured"; complaints: **contracts, payment
  processor lock-in, hardware cost, support wait times, limited offline**.

#### 3.8 Square for Restaurants
- **Features:** **coursing** (drag items between courses, **hold / fire**
  with one tap); checks assigned to **seats or tables**; **split by fraction**
  in one tap; customisable floor plans; KDS as a paid add-on (about $20 per
  device/month).
- **Offline:** offline card payments limited (sessions up to an hour,
  declined if not reconnected within 24 h; no contactless). A reviewer reports
  that when the internet goes down **tickets are lost and printers won't
  print from the KDS**.
- **GUI / UX:** GetApp 4.4 (53 reviews); "simple to get the hang of";
  complaints: **actions take three, four or more touches**, held items still
  send a kitchen ticket, **editing coursing on an open ticket is painful**,
  splitting by seat is hard to find.

#### 3.9 Lightspeed Restaurant (K-Series)
- **Features:** **multiple adjustable floor plans**, item transfers between
  tables, customisable menus, 50+ reports on iPad/phone/desktop, online
  ordering, integrations.
- **Offline:** keeps processing and **sending orders to kitchen/bar** during
  an outage, syncs later (vendor; one third-party source disagrees).
- **GUI / UX:** intuitive, efficient order management; complaints: **iPad
  only**, annual commitment, **Lightspeed Payments required**, pushy sales,
  complex pricing.

#### 3.10 TouchBistro
- **Features:** floor plans that consider **table spacing, time seated and
  spend per table**; **colour-coded server sections**; reservations (add-on);
  KDS; **local network mode** — devices keep talking to each other when the
  internet drops; hardware includes a mini server.
- **GUI / UX:** the most consistently praised for ease: Sonary ease 4.5/5,
  "intuitive, minimal training", "less cluttered than rivals". Complaints:
  **iPad only**, per-iPad yearly commitment, auto-renewing contracts,
  reservations cost extra, some screens slow.

#### 3.11 Clover
- **Features:** handhelds with **built-in cellular**; table service plan;
  automatic report emails; app marketplace.
- **Offline:** offline mode auto-queues transactions.
- **GUI / UX:** Capterra **3.8** (566 reviews), ease 4.0, **support 3.2**;
  complaints: glitchy, hard to navigate, **paid add-on apps**, confusing
  multi-device reporting, complicated pricing via resellers.

#### 3.12 SpotOn
- **Features:** order entry, **handheld POS** with tableside payment, QR
  ordering, online ordering, loyalty, email marketing, payroll and staff
  management, accounting integrations.
- **GUI / UX:** Capterra ease **4.2** (370 reviews), G2 4.4 (240): staff learn
  quickly ("night and day vs Micros"); complaints about recurring technical
  problems. Offline behaviour undocumented in our sources.

#### 3.13 Revel Systems
- **Features:** iPad POS, KDS, deep menu modifiers ("modifiers for
  modifiers").
- **GUI / UX:** Capterra **3.6** (323 reviews); flexible and feature-packed;
  complaints: **offline mode "never worked — no internet meant 100% down"**,
  no card payments offline, **iPads wear out (~2 years)**, fees for the phone
  app, slow support.

#### 3.14 Oracle Simphony
- **For:** hotels and large chains.
- **GUI / UX:** ease 3.8 (26 reviews); highly customisable, reliable
  offline mode praised, but **complex back end needing substantial training**.

#### 3.15 Lavu
- iPad restaurant POS (from about $59/month); too little review data in
  our sources to judge UX or offline behaviour.

### Free / open source

#### 3.16 Loyverse
- **Features (free):** POS, **KDS app**, kitchen printers, dining options,
  offline sales recording; paid add-ons for advanced inventory and staff.
- **GUI / UX:** cheap and kitchen-capable; complaints: **no table management**
  (workaround: predefined tickets per table), no table map, Android app more
  limited, occasionally buggy, live support paid.

#### 3.17 Odoo POS — Restaurant
- **Features:** **graphical floor-plan editor** (add, move, remove tables
  and seats), reserved tables shown on the floor, table transfer, preparation
  display for kitchen/bar, notes and allergies to printers, **QR self-ordering
  and kiosk**; part of a full ERP.
- **Offline:** POS session can **run and close offline**, payments sync later
  (vendor claim).

#### 3.18 Floreant POS
- **Features:** on-premise Java POS; split and merge tickets, table
  allocation, **kitchen printer routing per terminal**, cash control.
- **GUI / UX:** praised as the simplest open-source POS for small
  restaurants; reports and ticket templates hard to customise.

---

## 4. Offline behaviour compared

| Product | What keeps working without internet | What stops |
|---|---|---|
| Toast | Orders, card payments (offline), tickets to printers/KDS | Closing checks, tips, clock-out, reports, online ordering, loyalty, gift cards |
| Square | Limited offline card payments (≤ 1 h session) | Reviewer: kitchen tickets lost, KDS printing stops; contactless; gift cards |
| Lightspeed K | Orders to kitchen/bar | Sync and reporting until back online |
| TouchBistro | Local network between iPads, orders to KDS, offline payments | Some features depend on internet |
| Clover | Queued transactions | — (undocumented) |
| Revel | Reviewers say effectively nothing | "100% down" |
| Restroworks | Billing after manual switch to offline billing | Sync; dashboard |
| Petpooja | Desktop billing, KOT on LAN | Cloud dashboard, menu push, aggregators, campaigns |
| Odoo | Session including closing (vendor) | Online payments |
| Loyverse | Sales recording | Unclear for KDS |
| **Khaynjo** | **Everything**: tables, captain devices, KOT printing, kitchen screen, billing, settlement, shifts, day close, stock, books, reports, backups — on the restaurant's LAN | Only things that are online by nature: aggregators, online payments, messages |

**Lesson:** "offline mode" in the market is an emergency mode with a list of
things you cannot do. Khaynjo has no such list for daily service — that is
the headline difference to keep and to say in sales material.

---

## 5. What makes these GUIs convenient (and what annoys users)

**Praised**

1. **Learn in a shift** (TouchBistro, SpotOn, Square): big buttons, few
   screens, staff productive on day one.
2. **One integrated system** front and back of house (Toast).
3. **Handhelds / any phone at the table** (Toast Go, Clover Flex, SpotOn),
   including pay-at-table.
4. **Floor that tells the story at a glance**: time seated, spend, server
   sections (TouchBistro), colour states (Petpooja).
5. **Kitchen routing set once, then automatic** (Toast routing preview,
   Floreant per-terminal printers).
6. **Coursing with one-tap fire** (Square).

**Complained about**

1. **Too many touches** for common actions (Square), **cluttered or complex
   back office** (Simphony, Clover).
2. **Offline mode that isn't** (Revel, Square KDS) and features that vanish
   offline (Toast).
3. **Hardware lock-in**: iPad only (TouchBistro, Lightspeed, Revel), iPads
   wearing out (Revel).
4. **Money lock-in**: contracts, mandatory payment processing, add-on fees
   (Toast, Lightspeed, Clover, Revel).
5. **Mobile app missing desktop features** (Petpooja).
6. **Slow menu management** (Petpooja) and editing open tickets (Square).
7. **Support quality** across almost all vendors.

---

## 6. Feature matrix — market vs Khaynjo

"Market" lists where the feature is typical (I = Indian suites, G = global
POS, F = free/open source).

### 6.1 Ordering and billing

| Feature | Market | Khaynjo | Offline | Note / recommendation |
|---|---|---|---|---|
| Order types (dine-in, takeaway, delivery, counter) | I, G | **Have** | Local | — |
| Item search, short codes / aliases | I, G | **Have** | Local | — |
| Modifiers / add-ons, item notes | I, G, F | **Have** | Local | Priced variant matrix later |
| Sizes (Half/Full) | I, G | **Have** (each size its own item + dish group) | Local | — |
| Combos / BOGO / schemes | I, G | **Have** (BOGO + percent-off + fixed combo) | Local | Settings → Schemes; fixed combo expands components on Send KOT (held until fired); combo price only on bill |
| Menu timings / happy hour / time-based price | I, G | **Have** | Local | `schedule_json` on item; `price_now` / `available_now`; menu form human windows |
| Area-wise prices (AC dearer) | I | **Have** | Local | — |
| Discounts with reason and right | I, G | **Have** | Local | — |
| Complimentary (NC) items | I | **Have** | Local | — |
| Running table order, more KOTs, print bill, settle | I, G | **Have** | LAN | — |
| Settle: tendered, change, tip, part payment, due on khata | I, G | **Have** | Local | — |
| Split tenders | I, G | **Have** | Local | — |
| Split bill by items | I, G | **Have** (to a companion table) | LAN | — |
| **Split by seat / evenly by N** | G (Square, Toast) | **Have** | Local | `seat_no` on lines; Split evenly / Split by seat → companion bills |
| Move / merge tables, move items | I, G | **Have** | LAN | — |
| **Coursing: hold / fire** | G | **Have** | LAN | Held lines never print; Fire course sends only that course; fixed-combo components stay held until combo KOT |
| Customer-facing display (order + total + UPI QR) | G, some I | **Have** | Local / LAN | Counter UPI QR modal + LAN `GET /display` (lines, total, UPI data-URI when paying) |
| Integrated card terminals / payment links | G (processor-led), I | **Partial** (manual tender default; optional outbox stub) | Net | `enqueue_card_terminal` / payment-link outbox; never required for counter |
| E-bill by SMS / WhatsApp | I | **Have** (queue / WhatsApp); SMS drain **Planned** | Local SIM gateway / Net for WhatsApp / optional Cloud SMS | Sales → Share WhatsApp (`wa.me`); SMS → `notify_outbox`. Delivery plan: [`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md) |

### 6.2 Floor, reservations, front of house

| Feature | Market | Khaynjo | Offline | Note / recommendation |
|---|---|---|---|---|
| Areas, tables, colour states, minutes and amount | I, G | **Have** | LAN | — |
| Parcel as a table area | I (Petpooja) | **Have** | LAN | — |
| **Drawn floor-plan editor** (shapes, positions, seats, multiple floors) | G, F (Odoo, Square, Lightspeed) | **Have** (Grid ↔ Layout; X/Y/W/H editor + nudge; auto-pack defaults) | Local | `set_table_layout` / `ensure_table_layouts_packed`; `floor_view` meta |
| Server sections, colour by waiter | G (TouchBistro) | **Have** | LAN | `set_table_waiter`; Floor filter by waiter name |
| Reservations | I, G (often add-on) | **Have** | LAN | — |
| **Waitlist** with quoted wait time | G | **Have** | LAN | Floor waitlist: name, party, quoted mins → seat/cancel |
| Captain / waiter devices | I, G | **Have** (any Android or Windows device, LAN) | LAN | Competitors often require iPads or branded handhelds |
| Pay at table on handheld | G | **Have** | LAN | Rights-gated Settle (Allowed/WithApproval); waiter preset Denied; no hard counter-only gate |
| QR table ordering by guests | I (DotPe, Petpooja add-on), F (Odoo) | **Have** (LAN) / **Skipped** (Cloud mobile-data) | LAN (guest phone on restaurant Wi‑Fi) / Cloud (guest on mobile data) | `GET /t/{table}` + POST add on Server Wi‑Fi; mobile-data guests need Cloud later |
| Waiter call / service request | I (Petpooja add-on) | **Have** | LAN | Table button queues call; Floor ops badge; clear on attend |
| Token / order-ready display for takeaway | I, G | **Have** | LAN | `GET /tokens` HTML — ready takeaway tokens |
| Self-order kiosk | F (Odoo), G | **Have** | LAN | `GET /kiosk` + `POST /kiosk_order` on LAN |

### 6.3 Kitchen

| Feature | Market | Khaynjo | Offline | Note / recommendation |
|---|---|---|---|---|
| KOT per station, never prices, waiter + table | I, G | **Have** | LAN | — |
| Printer routes with fallback, test page | I, G | **Have** | LAN | Each station its own route |
| Kitchen display: states New → Preparing → Ready → Done | I, G, F | **Have** | LAN | — |
| All-day counts ("To cook now") | I, G | **Have** | LAN | — |
| Cancel after KOT with reason, wastage, approver, cancel slip | I | **Have** | LAN | — |
| **Ticket timers / ageing colours, target prep times** | G | **Have** | LAN | Minutes since send; amber/red from station `target_prep_mins` (default 15 → 10/20) |
| Recall / undo bump | G | **Have** | LAN | `recall_kot` Completed→Ready; Kitchen "Recently done"; LAN `POST /kot_recall` |
| Allergen flags on items and KOT | F (Odoo notes), G | **Have** | Local | `allergens` on product; printed on KOT text |
| Expo / pass screen (all stations, ready for service) | G | **Have** | LAN | `GET /expo` HTML — Ready across stations |

### 6.4 Menu, inventory, purchasing

| Feature | Market | Khaynjo | Offline | Note / recommendation |
|---|---|---|---|---|
| Menu with categories, photos, sold-out ("86") on every device | I, G | **Have** | LAN | — |
| Menu template (1,452 dishes with photos) | — (unusual) | **Have** | Local | A differentiator for fast setup |
| Recipes, stock deduction, reorder levels, running-low alerts | I, G | **Have** | Local | — |
| Raw material vs resold vs dish item kinds | I | **Have** | Local | — |
| **Expiry / batch tracking** | I (Petpooja) | **Have** | Local | Batch/expiry on purchases; alert list on Inventory ops panel |
| **Purchase orders to suppliers**, receive against PO | I, G | **Have** | Local | PO → receive → bill; supplier price history text |
| Supplier price comparison | I | **Have** | Local | From purchase / PO history |
| **Stock count / daily closing with variance** | I | **Have** | Local | Count session → expected vs counted → variance report + tip pooling report |
| Store requisition / central kitchen transfers | I (eZee, Restroworks) | **Have** | Local (file between branches) | Export/import stock transfer JSON like day snapshots |
| Bulk menu edit (price change for a category, CSV import) | I, G | **Have** | Local | Settings bulk % / set amount per category; CSV round-trip remains |

### 6.5 Staff, money, reports

| Feature | Market | Khaynjo | Offline | Note |
|---|---|---|---|---|
| Rights per person, "with approval" PIN | I | **Have** | LAN | — |
| Opening float, shift handover, day close with variance | I, G | **Have** | Local | Toast cannot do tips/clock-out offline; we can |
| Reports by staff, order type, hour, item, discounts, NC, cancellations | I, G | **Have** | Local | — |
| P&L, balance sheet, AR/AP aging, bank book, expenses | I (partial), G (via accounting apps) | **Have** | Local | Most POS products push this to Tally/QuickBooks |
| Tip pooling / tip report per staff | G | **Have** | Local | Tip pooling report by staff/shift from recorded tips |
| **Time clock / attendance** | G | **Have** | Local | Clock-in/out with PIN; hours report |
| Loyalty points / customer wallet | I, G | **Have** | Local | Earn on pay; redeem; wallet top-up/spend as tender — local only |
| Gift cards / vouchers | G | **Have** | Local | Numbered vouchers issue/redeem at billing |
| Feedback after the meal | I (Petpooja app), LimeTray | **Have** | LAN (tablet at counter) | `GET /feedback` stores scores locally |
| SMS / WhatsApp campaigns | I, G | **Have** (local segments + outbox); delivery **Planned** | Local gateway / Net / optional Cloud SMS | Parties CRM queues intents; WhatsApp via `wa.me`. SMS send: Local SMS Gateway (Android sibling, cloudless) or paid Cloud SMS — [`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md) |
| Multi-outlet dashboard, central menu push | I, G | **Partial** (snapshot export/import) / **Skipped** (live Cloud) | File / Cloud | Live roll-up = Kumkaj Cloud |
| Online orders from aggregators (Zomato, Swiggy) | I, G | **Partial** (optional file import + Net poll stub off by default) | Net | Outbox/inbox stub; never gates counter (KHAYNJO.md §4b) |
| Own online ordering website | I (LimeTray, DotPe), G | **Skipped** (Cloud) | Cloud | Kumkaj Cloud later; LAN kiosk/QR remain Have |

### 6.6 Platform

| Feature | Market | Khaynjo | Note |
|---|---|---|---|
| Runs the whole day without internet | Nobody fully | **Have** | Headline advantage (§4) |
| Hardware freedom | iPad-only for TouchBistro, Lightspeed, Revel; branded handhelds for Toast/Clover | **Have**: Windows, Linux, Android, any mix | Cheap Android tablets as captain devices |
| Same features on every device | Petpooja mobile lacks desktop features | **Have** (one codebase) | — |
| No contract, no processor lock-in, no per-terminal fee | Rare (Loyverse free tier) | **Have** (free local) | — |
| Languages | Mostly English; Petpooja has Indian languages for items | **Have**: 29 languages incl. Gujarati and RTL | — |
| Owner holds the data, daily backups | Vendor cloud | **Have** | — |

---

## 7. Where Khaynjo already wins

1. **No outage mode** — service, kitchen, settlement, shifts and books keep
   working without internet all day (§4). Toast, Square and Revel users list
   exactly these losses.
2. **No lock-in** — free local use, any hardware mix, no payment-processor
   requirement, no contracts, no paid add-ons for the KDS or reservations.
3. **Books included** — P&L, balance sheet, khata, suppliers and bank in the
   same app as billing; competitors export to accounting software.
4. **Indian restaurant mechanics done properly** — running tables, parcel as
   a table, NC, cancel-after-KOT with wastage, rights with approval, area
   prices, FSSAI on bills, opening float and shift slip.
5. **Fast setup** — menu template with photos, type-or-pick everywhere.

## 8. Local / LAN catch-up — status 2026-10-27

Every item that could ship **without Kumkaj Cloud** from the earlier
“behind” list is now **Have** (see §6 matrix). Ordered by original value:

| # | Build | Status | Offline |
|---|---|---|---|
| 1 | **Combos / BOGO / schemes** (incl. fixed combo KOT expand) | **Have** | Local |
| 2 | **Menu timings and happy-hour prices** | **Have** | Local |
| 3 | **Customer-facing display with UPI QR** (counter + LAN `/display`) | **Have** | Local / LAN |
| 4 | **KDS timers and ageing colours**, target prep times, recall | **Have** | LAN |
| 5 | **Coursing (hold / fire)** — held never print until fired | **Have** | LAN |
| 6 | **Split evenly / by seat** | **Have** | Local |
| 7 | **Loyalty points and prepaid wallet** | **Have** | Local |
| 8 | **PO, expiry, stock variance, supplier price history** | **Have** | Local |
| 9 | **Waitlist** and **server sections** | **Have** | LAN |
| 10 | **Token / order-ready display** | **Have** | LAN |
| 11 | **QR table ordering over restaurant Wi‑Fi** | **Have** (LAN); Cloud mobile-data **Skipped** | LAN |
| 12 | **Pay at table** (rights-gated) | **Have** | LAN |
| 13 | **Time clock and tip pooling** | **Have** | Local |
| 14 | **Drawn floor layout** (Grid ↔ Layout; editor + auto-pack) | **Have** | Local |
| 15 | **One-tap WhatsApp bill, SMS outbox** | **Have** (queue); SMS gateway / Cloud SMS **Planned** | Local / Net / Cloud SMS |
| 16 | Allergen tags; gift vouchers; bulk menu price edit | **Have** | Local |

Also Have outside this numbered list (same pass): waitlist siblings
(self-order kiosk `/kiosk`, expo `/expo`, waiter call, feedback
`/feedback`, staff visibility, Seated-without-order, shop pick-list
On/Off for units/areas/stations/tax). Floor colour no longer sticks on
**In kitchen** after pay/free (open KOTs closed; next seating filtered
by `occupied_at`).

### Still left (not Local-complete)

| Item | Status | Why it waits |
|---|---|---|
| Live aggregator SaaS (Zomato/Swiggy accept/reject in their cloud) | **Partial** stub (file import + Net poll off by default) | Needs vendor APIs + internet; never gates counter |
| Card terminal / payment-link confirmation | **Partial** (manual tender default; optional outbox) | Needs PSP / terminal vendor + Net |
| Guests ordering on **mobile data** / own public website | **Skipped** | Needs Kumkaj Cloud reachability |
| SMS delivery (Local SMS Gateway + paid Cloud SMS) | Queue **Have**; gateway & Cloud SMS **Planned** | See [`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md) — Android sibling (LAN/Bluetooth, shop SIM) or usage-billed Cloud SMS; hosted third-party blast SaaS still out of scope |
| Live multi-outlet dashboard / central menu push | **Skipped** (live) / **Have** (day snapshot file) | Live roll-up = Kumkaj Cloud |
| Priced variation **matrix** UI polish | Progressive depth | Half/Full as own items already Have |
| Real-hardware pilot, Store/Play listing | Owner | See `PRODUCTION_STATE_FOR_OFFLINE_USAGE.md` |

## 9. Needs the internet or a cloud (optional, never a gate)

| Feature | Why | Kumkaj approach (2026-10-27) |
|---|---|---|
| Zomato / Swiggy / aggregator orders | Their systems are online | **Partial** optional file import + Net poll stub (off by default); counter never depends on it |
| Card terminal integration, online payment links, payment confirmation from banks | Card networks and PSPs are online | Manual tender default; **Partial** outbox stub when internet is up |
| Guests ordering on mobile data, own ordering website | Must be reachable from the internet | **Skipped** until Kumkaj Cloud; LAN `/t/{table}` + `/kiosk` already Have |
| SMS/WhatsApp campaigns, review collection | Online / SIM | CRM outbox **Have**; Local SMS Gateway + Cloud SMS **Planned** ([`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md)); one-tap WhatsApp Have; third-party hosted blast SaaS still out of scope |
| Live multi-outlet dashboard, central menu push | Several sites must meet somewhere | Live **Skipped** (Cloud); today: per-branch day snapshots export/import |

---

## 10. UX lessons to apply to Khaynjo

1. **Count taps for the top five actions** (punch item, send KOT, print bill,
   settle cash, settle UPI) and keep each at three or fewer — Square's main
   complaint is "three, four or more touches".
2. **Never let "offline" appear as a mode** to staff; only show what is
   waiting to sync (already Khaynjo's design).
3. **Editing an open ticket must be easy**: move items between courses,
   seats and tables without re-entering (Square complaint).
4. **Menu changes in bulk**: price changes for a whole category in one step,
   CSV round-trip (Petpooja complaint).
5. **Cheap hardware must stay fast**: test on low-end Android tablets, the
   devices Indian restaurants buy (Revel's worn-out iPads show the cost of
   premium-only hardware).
6. **Back office stays simple**: Simphony and Clover lose points for complex
   back ends; keep settings in plain language with sensible defaults.

---

## 11. Sources

Indian suites and ordering
- Earlier deep-dive: [`RESTAURANT_MARKET_RESEARCH.md`](RESTAURANT_MARKET_RESEARCH.md) (Petpooja module map, Rajdhani video)
- [Petpooja on GetApp](https://www.getapp.ca/software/2050914/petpooja) · [Software Advice](https://www.softwareadvice.com/retail/petpooja-profile/) · [G2 features](https://www.g2.com/products/petpooja/features) · [Petpooja app marketplace](https://www.petpooja.com/restaurant-app-marketplace) · [SoftwareSuggest](https://www.softwaresuggest.com/petpooja)
- [Restroworks on Capterra](https://www.capterra.com/p/142877/Restroworks/reviews/) · [Software Advice](https://www.softwareadvice.com.au/software/77505/Restroworks) · [softwarefinder reviews](https://softwarefinder.com/retail/restroworks/reviews?page=2)
- [GoFrugal POS on Capterra](https://www.capterra.in/reviews/71693/gofrugal-pos-software) · [GoFrugal on G2](https://www.g2.com/sellers/gofrugal)
- [eZee Optimus on Capterra](https://www.capterra.in/reviews/166571/ezee-optimus)
- [DotPe on Slashdot](https://slashdot.org/software/p/DotPe/) · [DotPe on Fitgap](https://us.fitgap.com/products/006851/dotpe) · [Outlook Business on DotPe](https://www.outlookbusiness.com/magazine/business/story/with-nrais-blessing-this-fintech-start-up-is-dreaming-big-should-restaurant-aggregators-be-worried-5964)
- [LimeTray on Software Advice](https://www.softwareadvice.com.au/software/146263/limetray) · [Techjockey](https://www.techjockey.com/detail/limetray)

Global POS
- Toast: [Offline mode (support)](https://support.toasttab.com/article/Using-Toast-in-Offline-Mode) · [Offline during ISP outage](https://support.toasttab.com/article/Offline-Mode-Using-Toast-During-a-ISP-Outage) · [Menu item routing preview](https://support.toasttab.com/en/article/Preview-Menu-Item-Routing-From-Your-Toast-POS) · [Platform overview](https://doc.toasttab.com/doc/platformguide/platformToastPlatformOverview.html) · [POSUSA review](https://www.posusa.com/toast-pos-review/) · [Airframe KDS analysis](https://www.airframe.ai/product/pos-toasttab-com-products-kitchen-display-system/analysis)
- Square: [GetApp](https://www.getapp.com/retail-consumer-services-software/a/square-for-restaurants/) · [Square pricing / offline (IE)](https://squareup.com/ie/en/point-of-sale/restaurants/pricing) · [Seller community feedback](https://sellercommunity.com/t5/Square-for-Restaurants/We-want-to-hear-from-you-about-the-strengths-and-weaknesses-of/m-p/387373) · [App Store](https://apps.apple.com/app/id1346862885)
- Lightspeed: [MobileTransaction review](https://www.mobiletransaction.org/lightspeed-restaurant-pos-review/) · [GetApp reviews](https://www.getapp.com/retail-consumer-services-software/a/lightspeed-restaurant/reviews/) · [SelectHub](https://www.selecthub.com/p/pos-software/lightspeed/)
- TouchBistro: [NerdWallet review](https://www.nerdwallet.com/article/small-business/touchbistro-review) · [Sonary](https://sonary.com/reviews/touchbistro/) · [Top Ten Reviews](https://www.toptenreviews.com/touchbistro-review) · [Product guide (PDF)](https://www.touchbistro.com/wp-content/uploads/2023/03/US_TB-Product-Guide_Mar-2023.pdf)
- Clover: [Capterra reviews](https://www.capterra.com/p/226864/Clover-/reviews/) · [StartupOwl](https://startupowl.com/reviews/clover) · [BestGuide](https://bestguide.com/review/clover-pos/)
- SpotOn: [Capterra reviews](https://capterra.com/p/197473/SpotOn-Restaurant/reviews/) · [GetApp](https://www.getapp.com/bar-pos-software/a/spoton-restaurant)
- Revel: [TechRadar review](https://www.techradar.com/reviews/revel-point-of-sale-pos-review) · [Capterra reviews](https://www.capterra.com/p/135040/Revel-iPad-POS/reviews/?page=5)
- Oracle Simphony / Lavu: [GetApp Simphony reviews](https://www.getapp.ca/reviews/133023/simphony) · [Capterra Simphony](https://www.capterra.in/reviews/202814/oracle-micros-simphony) · [TrustRadius Lavu vs Simphony](https://www.trustradius.com/compare-products/lavu-vs-oracle-simphony-vs-wi-q)

Free / open source
- Loyverse: [Merchant Maverick](https://www.merchantmaverick.com/reviews/loyverse-review/) · [FitSmallBusiness](https://fitsmallbusiness.com/loyverse-review/) · [MobileTransaction](https://www.mobiletransaction.org/loyverse-review/)
- Odoo: [Restaurant features](https://odoo.com/app/point-of-sale-restaurant-features) · [POS features](https://ODOO.com/app/point-of-sale-features) · [Cybrosys Odoo 18 guide](https://cybrosys.com/odoo/odoo-books/v18-ce/point-of-sale-and-restaurant/restaurant)
- Floreant: [Capterra](https://www.capterra.ca/software/159984/floreant-pos) · [LinuxLinks](https://www.linuxlinks.com/floreantpos/)

Our own docs
- [`KHAYNJO.md`](KHAYNJO.md) §4b, §4d · [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](../../../docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md) §6, §21
