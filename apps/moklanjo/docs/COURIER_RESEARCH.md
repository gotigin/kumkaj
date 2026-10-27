# Courier business software — market research and Moklanjo comparison

> **Collected:** October 2026 (web research: vendor pages, help centres,
> G2 / Capterra / GetApp / Software Advice listings, editorial reviews).
> **Our app:** **Moklanjo** — the Kumkaj courier-agency operating system
> (formerly "Kumkaj Courier"; product brief
> [`MOKLANJO.md`](MOKLANJO.md)).
> **Rule we compare against:** Moklanjo is **offline-first and free to run
> locally** (vision §6, §21). A competitor feature is worth building when it
> works **without the cloud** and is genuinely good for the user; features
> that need the internet are optional connectors, and features that need a
> hosted service belong to opt-in Kumkaj Cloud — never a gate on daily work.
>
> **Catch-up status (2026-10-27):** every Local/LAN item from the earlier
> behind list is **Have** — see §7. **Still left** there is only Cloud/Net
> connectors and owner actions (real-hardware pilot).

---

## 1. How to read this

**Caveats about the evidence**

- Many "best courier software" round-ups are written by vendors about their
  competitors (Shipday, Upper, Circuit). We took **mechanics** from them, not
  rankings.
- Review samples are small for most courier products (often 10–100 reviews).
  Ratings are quoted with their source and should be read as directional.
- Prices change often; the figures below are what listings reported in
  October 2026, kept only to show the pricing *model* (per driver, per user,
  per order, flat).

**Offline tags used in the comparison tables**

| Tag | Meaning |
|---|---|
| **Local** | Works on one device with no network at all |
| **LAN** | Needs only the office's own Wi‑Fi between devices (no internet) — Moklanjo's Server + paired devices |
| **Net** | Needs internet on the device at that moment (carrier API, WhatsApp, maps), but **no Kumkaj cloud**; can queue offline and send later |
| **Cloud** | Needs a hosted service (someone else's cloud, or opt-in Kumkaj Cloud, usage-billed per vision §21) |

**Status tags for Moklanjo:** **Have** (ships today) · **Partial** (part of it
ships) · **Gap** (not built).

---

## 2. The market at a glance

"Courier software" is four different kinds of product. Moklanjo's users (an
agency / franchise counter that books walk-ins, hands bags to carriers and
may run its own riders) sit mostly in group A, with group C as the "own
fleet" depth.

| Group | What it is for | Products studied |
|---|---|---|
| **A. Agency / franchise counter & courier office** | Book walk-in and account consignments, rates, receipts, party billing, manifests, delivery-boy run sheets | FR Billing, Indian courier-office desktop packages (IndiaMART listings), Vyapar (general billing used by couriers), Delhivery / DTDC partner tooling, Shiprocket (aggregator panel) |
| **B. Courier company TMS (same-day / regional couriers)** | Order entry, dispatch board, rating engine, billing, driver pay settlement, client portal | CXT Software, Datatrac, Express-Pak, e-Parcel, Elite EXTRA, Shipox |
| **C. Last-mile delivery SaaS** | Dispatch, driver app, proof of delivery (POD), live tracking, customer notifications, route optimisation | Onfleet, Tookan (JungleWorks), Shipday, Detrack, Track-POD, LogiNext Mile |
| **D. Enterprise logistics orchestration** | Multi-carrier, first/middle/last mile, allocation engines | Shipsy, Bringg, DispatchTrack |

Almost every product in groups B–D is **cloud SaaS with per-driver, per-user
or per-order pricing**. Offline support, where it exists, is limited to the
**driver app caching POD until signal returns**; dispatch, billing and
reports stop when the office loses internet. Group A in India is the only
place where desktop/offline products still exist, and they are dated.

---

## 3. Product profiles

Each profile: who it is for · what it does · how convenient the GUI is
(what users praise and complain about) · offline behaviour · pricing model.

### A. Agency / franchise counter & courier office

#### 3.1 FR Billing (CodeTentacles, India)
- **For:** courier franchisees (DTDC/Delhivery-style counters).
- **Features:** customisable **rate master** with bulk rate updates;
  **walk-in booking**; bill printing; **billed vs unbilled consignments**;
  payment management; **receipts with barcodes**; expense tracking; GST
  reports. Premium tier: several franchisees on one platform, **customer
  portal** (bookings, invoices, dues), role-based access.
- **GUI / UX:** listings repeat vendor text; no independent UX evidence.
- **Offline:** cloud-based.
- **Pricing:** from about ₹799/month (listing).

#### 3.2 Indian courier-office desktop packages (e.g. Pune vendor on IndiaMART)
- **For:** traditional courier offices and franchise hubs that also deliver.
- **Features:** **incoming and outgoing manifests**; **run sheets (DRS) per
  delivery boy, area and DRS number**; **POD summary**; **C-note (docket)
  printing with or without barcode**; payment entry; **transhipment and
  co-loader billing**; **independent zone generation and unlimited rate
  slabs**; reports.
- **GUI / UX:** classic form-heavy desktop screens; no review data.
- **Offline:** sold in **online and offline versions** — one of the few
  products in the market that runs without internet.
- **Takeaway:** this is the closest *functional* match to Moklanjo's
  audience, and shows what courier offices expect: inbound manifests, DRS
  printing, co-loader billing.

#### 3.3 Vyapar (general SMB billing, used by small couriers)
- **For:** small Indian businesses; has a "courier billing" landing page.
- **Features:** GST invoices, **cash sale mode for walk-ins / COD**, credit
  tracking with payment-due reminders, sync across devices, multiple firms,
  **WhatsApp sharing of invoices**, barcode printing.
- **GUI / UX:** Capterra ease of use **4.4/5** (181 reviews); praised for
  WhatsApp sharing and the mobile + desktop versions; complaints that the app
  **hangs**, **barcode printing problems** went unresolved, and blank
  WhatsApp bills; "needs time to understand all features".
- **Offline:** works offline (desktop and mobile), sync is a paid feature.
- **Takeaway:** proves Indian SMBs value **offline + WhatsApp share**, but it
  has no courier workflow (no AWB, weights, manifests).

#### 3.4 Delhivery / DTDC franchise partner tooling
- **For:** partners running a carrier-branded retail counter or a parcel
  delivery centre.
- **Features:** the carrier supplies its own app/tracking system and
  **handheld scanners**; the partner books walk-ins on the carrier's system and
  earns commission. DTDC's public connector (DTDC EConnect for Shopify:
  bookings, AWBs, labels, cancellations, tracking links) is rated **1.3** on
  its listing.
- **Takeaway:** a franchise counter often uses the **carrier's booking
  system for the AWB** and still needs **its own books** (commission, cash,
  party credit, expenses) — that second job is Moklanjo's core. Multi-carrier
  agencies juggle several carrier portals and want one desk.

#### 3.5 Shiprocket (India, shipping aggregator)
- **For:** online sellers; also used by small agencies as a booking back-end.
- **Features:** assign **AWB**, choose courier (**courier recommendation
  engine**), **print labels**, track from one panel; **COD remittance** with an
  "early COD" option; **automated NDR panel** (non-delivery reports) for
  faster returns; insurance on lost shipments; white-label tracking pages;
  26,000+ pincodes; returns.
- **GUI / UX:** reviewers call the dashboard **intuitive and clean**;
  complaints concentrate on operations: frequent **NDRs / RTOs without
  notice**, delayed pickups, **weight-discrepancy charges**, wallet balance
  issues, slow or scripted support.
- **Offline:** cloud only.
- **Takeaway:** the features agencies copy from aggregators are **rate
  comparison across carriers, NDR/RTO handling, COD remittance tracking and
  weight-discrepancy disputes**.

### B. Courier company TMS

#### 3.6 CXT Software (US)
- **For:** courier, on-demand and route delivery companies.
- **Features:** **order entry**; **drag-and-drop dispatch board** with
  scheduling and allocation; **billing rate management**, invoicing,
  QuickBooks integration; **client portal** (place orders, track, delivery
  confirmation, invoices with card payment); driver app (job accept/reject,
  **stop list with stop consolidation**, GPS, **signature capture**,
  **barcode scanning**).
- **GUI / UX:** reviewers praise real-time courier tracking, routing and
  onboarding with training videos; complaints: driver tracking "could use an
  overhaul", price (about $329/month quoted by one reviewer) too high for
  small firms.
- **Offline:** cloud.

#### 3.7 Datatrac (US)
- **Features:** **rating and settlement engine** — mileage, zone, zip code,
  time, weight and **fuel surcharges**; **driver pay settlement**;
  QuickBooks / Sage; driver app POD with **photos, signatures and ID**.
- **GUI / UX:** too few reviews (2 on Capterra) to judge.
- **Offline:** cloud. From about $100/month.

#### 3.8 Express-Pak (US)
- **Features:** one integrated system for **order entry, dispatching, rating,
  billing, receivables, settlements and accounting**; barcode recognition;
  label printing; routing and delivery tracking.
- **GUI / UX:** Capterra **4.9/5** (12 reviews), top for likelihood to
  recommend.
- **Offline:** hosted; flat $99/month.
- **Takeaway:** small couriers value **one system from order to accounts** —
  the same "books + desk on one spine" idea as Moklanjo.

#### 3.9 e-Parcel
- **Features:** national and international services, special services,
  **branch rates**, **alliance (partner network) management**, billing,
  tracking, mobile app, reporting. Capterra **4.8/5** (12 reviews).

#### 3.10 Elite EXTRA (US)
- **Features:** service dispatch, route optimisation, GPS driver tracking,
  **e-signature and image POD**, **customer tracking link with map and ETA**,
  **customer portal**, ETA API.
- **GUI / UX:** the **driver app is called the best part** ("simplified
  driver app"); fast support; complaints: **downtime because it is
  cloud-based**, reports **not customisable**, web pages **not intuitive**.

#### 3.11 Shipox (white-label courier management)
- **Features:** orders from **customer app/web, merchant API or admin**;
  **bulk order upload**; service types (same-day, next-day) with package
  rules; **COD workflow** — COD amount assigned with the order, **driver app
  reminds the driver to collect**, finance settlement; hub/warehouse
  management; driver app with turn-by-turn navigation, POD, push alerts;
  live tracking, heat map, dashboard analytics; white-label apps.
- **Offline:** cloud. Price on request.

### C. Last-mile delivery SaaS

#### 3.12 Onfleet
- **Features:** dispatcher dashboard; driver app (assignments, navigation);
  **POD by photo, signature and barcode scan**; route optimisation;
  analytics (90-day history on entry plan).
- **GUI / UX:** strongest ease-of-use scores in the group — G2 ease of use
  **9.3** vs Tookan 8.4; Capterra 4.6; quick setup praised by most reviewers.
  Complaints: **drag-and-drop task assignment is tedious** at volume,
  glitches/downtime, limited customisation, **entry price around
  $500–599/month**.
- **Offline:** no documented offline mode.

#### 3.13 Tookan (JungleWorks)
- **Features:** **auto-allocation modes** — one-by-one, send-to-all,
  batch-wise, **round robin**, **nearest available**; **geofenced regions**
  that filter which agents can get a task; agent tags and schedules;
  role-based dashboard access.
- **GUI / UX:** G2 ease 8.4, Capterra 4.2; complaints about slow custom
  deployments (6–8 months), support after payment, billing disputes.
- **Offline:** no documented offline mode. From about $99–129/month.
- **Takeaway:** **rule-based auto-assignment by zone/area** is valued and
  needs no cloud.

#### 3.14 Shipday
- **Features:** free tier up to 10 drivers; auto-dispatch by location and
  availability; route optimisation; live tracking; branded customer
  messages; DoorDash/Uber integration.
- **GUI / UX:** Capterra 4.7 (29 reviews). A courier reviewer said it **does
  not suit courier work** — no multiple pickups delivered to one drop, and
  **too many taps while driving** (vendor agreed it is not a fit for pickup
  services).
- **Takeaway:** restaurant-delivery tools are not courier tools; **pickup
  runs and multi-pickup consolidation** matter for couriers.

#### 3.15 Detrack
- **Features:** driver app captures **up to 10 photos and a signature per
  delivery, even offline**, and **submits several PODs at once** when back in
  range; jobs imported by **Excel/CSV**; auto-assignment **by zone**; route
  optimisation with time windows and service time; customer **SMS/email +
  live tracking page with ETA**; **pre-trip vehicle inspection** checklists.
- **GUI / UX:** Capterra **4.7/5**, G2 4.5; praised for ease of use and
  offline POD caching; complaints: **interface cluttered**, wanted a default
  "take photo" step, route preview missing after going offline.
- **Pricing:** from about $26 per vehicle/month.

#### 3.16 Track-POD
- **Features:** POD with **geotag, timestamp, signature, package scans**;
  route planning; customer notifications; order import integrations; driver
  app **works offline and syncs on reconnect**.
- **GUI / UX:** POD rated **4.9** by 50 Capterra reviewers; easy driver app;
  complaints: **routes cannot be edited once started**, **dispatch is
  desktop-only** on standard plans, per-user price high for small teams,
  manual work with variable weights/volumes, can't see carrier shipments in
  the same dashboard as own deliveries.

#### 3.17 LogiNext Mile
- **Features:** route optimisation with **dynamic rerouting**, geofencing,
  ePOD, automatic order allocation, ETA predictions and delay alerts,
  reporting dashboards.
- **GUI / UX:** Capterra ease of use **4.3/5** (75 reviews); "makes
  dispatching less stressful"; complaints: **features inconsistent across
  platforms**, weaker iOS app, limited customisation.

### D. Enterprise orchestration

#### 3.18 Shipsy
- **Features:** first/middle/last mile visibility; **3PL / carrier
  aggregation**; **intelligent allocation** of shipments to vehicles and
  carriers by weight and location; unified tracking dashboard; **driver app
  that works without data on every screen**, reconciling captures later;
  WhatsApp/SMS/email shipment notifications.

#### 3.19 Bringg / DispatchTrack
- **Features:** carrier management, returns, route optimisation, live
  tracking; integrations (Shopify, Stripe, Twilio, Google Maps).
- **Takeaway:** enterprise scale; small review pools; not a model for a
  counter-sized business, but **carrier management + own fleet in one view**
  matches Moklanjo's "agency + own fleet" shape.

---

## 4. What makes these GUIs convenient (and what annoys users)

**Praised again and again**

1. **Quick setup and short training** (Onfleet, Detrack, Express-Pak) —
   operators judge a system in the first hour.
2. **A simple driver app** (Elite EXTRA, Track-POD, Detrack): one stop at a
   time, big buttons, POD in two or three taps.
3. **POD that never gets lost**: cached offline and batch-submitted
   (Detrack, Track-POD, Shipsy).
4. **CSV / Excel import of jobs** instead of retyping (Detrack, Shipox,
   Shiprocket bulk upload).
5. **One panel from booking to label to tracking** (Shiprocket, CXT client
   portal) — fewer windows.
6. **Customer tracking link with ETA** — cuts "where is my parcel" calls.
7. **Rule-based auto-assignment** (Tookan, Detrack by zone, Shipsy) instead
   of dragging every job.

**Complained about again and again**

1. **Too many taps / maze of menus**, especially while driving (Shipday).
2. **Cluttered dispatcher screens** (Detrack) and **tedious drag-and-drop**
   at volume (Onfleet).
3. **Mobile app missing desktop features** or **inconsistent across
   platforms** (LogiNext, Track-POD desktop-only dispatch).
4. **Downtime because it is cloud** (Elite EXTRA, Onfleet glitches) — the
   office stops when the internet or vendor stops.
5. **Per-driver / per-user pricing** that punishes growth (Track-POD,
   Onfleet, CXT).
6. **Rigid flows**: routes not editable after start, reports not
   customisable.
7. **Opaque money**: COD remittance delays, weight-discrepancy deductions,
   wallet balances (Shiprocket).

---

## 5. Feature matrix — market vs Moklanjo

Legend for "Market": where the feature is common (A/B/C/D = groups in §2).

### 5.1 Booking desk and pricing

| Feature | Market | Moklanjo | Offline | Note / recommendation |
|---|---|---|---|---|
| Walk-in booking loop (next customer → weigh → charge → receipt) | A | **Have** | Local | Keep it the fastest path; competitors' web forms are slower |
| Cash booking vs account booking | A | **Have** | Local | — |
| Rate master: zone × service × weight slab, first/extra kg | A, B | **Have** | Local | — |
| Party-wise (customer-specific) rates | A, B | **Have** | Local | `products.party_id` override cards per zone; booking prefers matching consignor then default |
| Chargeable weight: actual vs volumetric, slab rounding | A, B | **Have** | Local | Divisor and slab are shop settings |
| Extra charges: fuel %, docket, ODA, packing, COD fee | A, B | **Have** | Local | Datatrac/Express-Pak also do time/mileage surcharges — not needed for counters |
| **Pincode → zone/service lookup** (serviceability) | A, aggregators | **Have** | Local | `pincode_service` + CSV import; 6-digit autofill plus destination autocomplete from pin prefix / zone / past destinations |
| **Rate quote / compare carriers** for one parcel (price + our margin) | Aggregators | **Have** | Local | Carrier buy cards (`carrier_party_id`) vs sell; booking desk compare line + pick cost. Live carrier rates would need Net |
| Bulk booking (CSV) | A, B, C | **Have** | Local | Add an import preview with row errors |
| Credit limit on account parties | A | **Have** | Local | — |
| Declared value / insurance / content description | A, aggregators | **Have** | Local | Fields on the docket + on the label/receipt |
| E-way bill number on docket | A (India) | **Have** | Local (field) / Net (generation) | Store and print the number; generating it needs the government portal |
| Customer self-booking portal | A premium, B, D | **Have** (LAN `/book`) / **Skipped** (Cloud from home) | LAN (in office) / Cloud (from home) | Office Wi‑Fi HTML form creates a booking on the Server; remote self-booking stays Kumkaj Cloud |

### 5.2 Labels, scanning, documents

| Feature | Market | Moklanjo | Offline | Note / recommendation |
|---|---|---|---|---|
| AWB/CN numbering + Code 128 label | A, B, aggregators | **Have** | Local | — |
| Scan-to-find with USB/Bluetooth wedge scanner | A, B, C | **Have** | Local | — |
| **Phone camera barcode scanning** | C (all driver apps) | **Have** | Local | Wedge remains primary; “Pick image to scan” decodes Moklanjo synthetic / high-contrast strips via `barcode_scan` (no heavy crate) |
| C-note / docket print with and without barcode, thermal and A4 | A | **Have** | Local | Receipt + SVG label + A4/A5 `format_docket_text` consignor/consignee blocks |
| **Printable manifest / run sheet (DRS)** with signatures column | A | **Have** | Local | Manifests exist in the books; print them per carrier bag or per rider run, with counts, COD total and a sign-off line |
| Incoming manifest (bags received from hub for delivery) | A (delivery centres) | **Have** | Local | Scan/type AWBs in → incoming manifest lines + in-transit for own-fleet delivery |
| Period party invoice (unbilled dockets → one tax invoice) | A, B | **Have** | Local | — |

### 5.3 Dispatch, riders and proof of delivery

| Feature | Market | Moklanjo | Offline | Note / recommendation |
|---|---|---|---|---|
| Riders/vehicles master | B, C | **Have** | Local | — |
| Own-fleet run (assign dockets to a rider) | B, C | **Have** | Local | — |
| Carrier handover manifest | A, D | **Have** | Local | — |
| Tracking statuses (booked → handed over → OFD → delivered / RTO) | All | **Have** | Local | — |
| **NDR: failed-delivery reason, reattempt, reschedule** | Aggregators, C | **Have** | Local | Reasons list (door locked, refused, address wrong…), attempt count, next attempt date; RTO after N attempts |
| **Rider app (phone): today's run, stop list, POD, COD collected** | B, C (core product) | **Have** | **Local on the phone + LAN sync at the office** | Same binary, `AppMode::Rider`: download today's run at office, offline stop list, POD/signature/COD/NDR → `lan_outbox`, “N waiting to reach office”; sync on office Wi‑Fi. Optional GPS lat/lon on POD. Live map / Cloud relay **Skipped** |
| **Signature capture** on POD | B, C | **Have** | Local | Stroke list → WebP attachment (`attach_signature_strokes`); desk + rider |
| POD photo + note | B, C | **Have** | Local | Desk + rider mode (same attachment path) |
| Pickup requests (collect from a consignor) and pickup runs | B, C | **Have** | Local | Multi-pickup → one drop run, assign rider |
| Auto-assignment rules (zone/area → rider, round robin) | C | **Have** | Local | Area rules + one-action assign all of area |
| Route order of stops | C | **Have** (area/pincode) / **Skipped** (road routing Net) | Local (by area/pincode order) / Net (road routing) | Sort by area/pincode; reorder anytime (never lock) |
| Live GPS map of riders | C | **Skipped** (Cloud/map) / **Partial** (optional lat/lon on event) | Net / Cloud | Optional GPS points on POD to Server; no map tiles / Cloud ETA |
| Vehicle inspection / fuel log | C | **Have** | Local | Checklist per run + optional fuel note |
| Rider pay / commission statements | B (Datatrac, Courier Mobile) | **Have** | Local | Per-delivery or per-kg + COD handed in → daily commission slip |

### 5.4 COD, money and books

| Feature | Market | Moklanjo | Offline | Note / recommendation |
|---|---|---|---|---|
| COD register (collected, pending, remitted to consignor) | A, B, aggregators | **Have** | Local | — |
| COD cash handover per rider (expected vs handed in) | B, Shipox | **Have** | Local | Rider COD settle + pay/commission slip in one daily settle |
| Carrier cost and margin per docket | A | **Have** | Local | — |
| **Carrier invoice / weight-discrepancy reconciliation** | Aggregators (pain point) | **Have** | Local (CSV import) | Import bill CSV → match AWB, flag weight/charge diffs, apply costs; copy report from Reports |
| Party ledgers, AR/AP, aging, day book, bank book, P&L, balance sheet | A (partial), B (via QuickBooks) | **Have** | Local | Most competitors push this to QuickBooks/Tally; Moklanjo owns it |
| Expense vouchers with bill photo | A | **Have** | Local | — |
| Franchise commission tracking (carrier pays the agency %) | A (franchise) | **Have** | Local | Period statement % of margin/revenue per carrier |
| Accounting export (Tally / CSV) | B | **Have** | Local | Day-book CSV + Tally-friendly voucher CSV (document columns) |

### 5.5 Customers, notifications, tracking

| Feature | Market | Moklanjo | Offline | Note / recommendation |
|---|---|---|---|---|
| Public track-by-AWB page | All | **Have** (on the office LAN) | LAN | Reaches only devices on the office Wi‑Fi; outside customers need Net exposure or Cloud |
| SMS / WhatsApp notifications (booked, OFD, delivered) | A, C, D | **Have** (drain: wa.me / Copy SMS / optional Android `sms:` stub; mark sent) | Net (WhatsApp) / **Local via phone SIM (SMS)** | Outbox drains without requiring Net for booking; WhatsApp needs device data. No Kumkaj cloud |
| Receipt/bill shared on WhatsApp | A (Vyapar) | **Have** | Net on the device | One-tap `wa.me` with receipt text after booking |
| Customer tracking link with live ETA | C | **Skipped** | Cloud | Out of scope until Kumkaj Cloud |
| Customer feedback / rating after delivery | C | **Have** (local score) / **Skipped** (public Net survey) | Local | Score on delivered AWB (desk/rider); no public survey |

### 5.6 Platform, devices and trust

| Feature | Market | Moklanjo | Offline | Note |
|---|---|---|---|---|
| Works with no internet all day | Rare (driver-app caching only) | **Have** | LAN | Our main advantage — dispatch and billing never stop |
| Several counters on one book | All (cloud) | **Have** | LAN | Pairing code, offline queue, no double booking |
| Any OS mix (Windows / Linux / Android) | Web apps; driver apps Android/iOS | **Have** (iOS later) | — | Competitors often split web dispatch + mobile driver |
| Same features on every device | Often not (LogiNext, Track-POD) | **Have** (one codebase) | — | Answer to "mobile lacks desktop features" |
| Staff rights with approval PIN, audit log | B, D | **Have** | Local | Cancel-not-delete with reasons |
| Daily backups, restore | Vendor-side | **Have** | Local | Owner holds the data |
| 29 languages incl. RTL | Rare (mostly English) | **Have** | Local | Strong for Indian staff |
| Free, no per-driver fee | Rare (Shipday free tier ≤10 drivers) | **Have** | — | Removes the #5 complaint in §4 |
| Multi-branch roll-up | B, D (cloud) | **Have** (snapshot export/import) | Local / file | Live roll-up would need Cloud |

---

## 6. Where Moklanjo already wins

1. **Nothing stops when the internet stops.** Booking, labels, scanning,
   COD, books and reports keep running on the office LAN. Every group B–D
   product loses dispatch and billing in an outage; Elite EXTRA and Onfleet
   users complain about exactly that.
2. **Books and desk on one spine.** Group C products have no accounting;
   group B pushes it to QuickBooks; Indian counter tools split billing from
   operations. Moklanjo keeps parties, AR/AP, bank, P&L, period bills, COD and
   carrier margin together.
3. **Agency and own fleet together.** Most tools are either carrier
   aggregators (Shiprocket) or own-fleet dispatch (Onfleet). Moklanjo models
   both on one docket (MOKLANJO.md "Product shape").
4. **No per-driver, per-user or per-order price** — growth is not taxed.
5. **Language and device freedom**: 29 languages, any OS mix, one codebase so
   no "mobile app lacks features".

## 7. Local / LAN catch-up — status 2026-10-27

Every item that could ship **without Kumkaj Cloud** from the earlier
“behind” list is now **Have** (see §5 matrix). Ordered by original value:

| # | Build | Why (evidence) | Offline |
|---|---|---|---|
| 1 | **Rider app mode** on Android: download today's run at the office, stop list, POD photo + **signature**, COD collected, failed-delivery reason; everything stored on the phone, synced on return to office Wi‑Fi | Core of every group B/C product; Detrack/Track-POD/Shipsy prove offline POD caching is what drivers like | Local + LAN — **Have** |
| 2 | **Printable manifest / DRS run sheet** (per carrier bag, per rider) with counts, COD total, sign-off | Standard in Indian courier offices (§3.2) | Local — **Have** |
| 3 | **NDR / RTO handling**: reasons, attempts, reattempt date, auto-RTO after N attempts, RTO charges | Shiprocket's NDR panel; agencies' top daily problem | Local — **Have** (incl. RTO fee line on auto-RTO) |
| 4 | **Pincode → zone/carrier serviceability** from imported CSVs, with destination autocomplete | Aggregators and Indian desktop tools both have it | Local — **Have** (prefix / past destinations beyond exact 6-digit) |
| 5 | **Carrier rate compare + margin** at booking from locally stored carrier rate cards | Aggregator's main selling point; agencies juggle several carriers | Local — **Have** |
| 6 | **Carrier bill reconciliation** (CSV import, match AWB, flag weight/charge differences) | Weight-discrepancy deductions are a top Shiprocket complaint | Local — **Have** |
| 7 | **Rider settlement**: COD expected vs handed in, per-delivery pay, daily slip | Datatrac/Courier Mobile pay statements; Shipox COD settlement | Local — **Have** |
| 8 | **Pickup requests and pickup runs** (multi-pickup → one drop) | Shipday courier reviewer: missing this made it unusable | Local — **Have** |
| 9 | **Incoming manifest** for delivery centres (scan bags in → delivery dockets) | Delhivery parcel-delivery-centre partners, §3.2 | Local — **Have** |
| 10 | **Party-specific rate cards** | Standard in groups A and B | Local — **Have** |
| 11 | **Phone-camera barcode scanning** | Every driver app | Local — **Have** (image pick + best-effort; wedge primary) |
| 12 | **SMS over the device SIM** for booked/OFD/delivered (outbox already exists) and one-tap WhatsApp share | Vyapar's most-praised feature; customers expect messages | Local (SMS) / Net (WhatsApp) — **Have** (drain path) |
| 13 | Docket fields: declared value, insurance, contents, e-way bill no. | Indian compliance and claims | Local — **Have** |
| 14 | Auto-assignment rules (area → rider) and editable stop order by area/pincode | Tookan/Detrack; Onfleet drag-and-drop complaint | Local — **Have** |
| 15 | Tally-friendly voucher export; franchise commission statements | Indian accountants; franchise income | Local — **Have** |

Also Have: LAN corporate `/book`, vehicle inspection/fuel, A4/A5 docket,
optional lat/lon on POD, local delivery feedback score.

### Still left (not Local-complete)

| Item | Status | Why it waits |
|---|---|---|
| Live carrier API book/track (Delhivery, DTDC, …) | Optional Net connector (not built as live API) | Needs carrier credentials + internet; desk stays manual AWB |
| Live GPS **map** of riders / customer ETA page | **Skipped** (map/Cloud); lat/lon on POD **Have** | Map tiles / public ETA need Net or Cloud |
| Road-network route optimisation | **Skipped** | Needs map services; area/pincode sort is Have |
| Self-booking / public track from **outside** the office | **Skipped** | Needs Kumkaj Cloud; LAN `/book` + `/track` Have |
| WhatsApp Business API automation, e-way **generation** | Optional later | Device WhatsApp share + e-way **field** are Have |
| Real-hardware pilot, Store/Play listing | Owner | See `PRODUCTION_STATE_FOR_OFFLINE_USAGE.md` |

## 8. Needs the internet or a cloud (optional, never a gate)

| Feature | Why it can't be purely local | Kumkaj approach (2026-10-27) |
|---|---|---|
| Book AWBs and pull tracking through carrier APIs (Delhivery, DTDC, Blue Dart…) | Carrier systems are online | Optional connector when internet is up (not live yet); desk keeps working with manual AWBs |
| Live GPS tracking of riders, live ETA links for customers | Positions must reach the office from the road / public web | Optional lat/lon on POD → Server **Have**; live map / public ETA **Skipped** (Cloud or map Net) |
| Road-network route optimisation | Needs map data/services | Area/pincode ordering offline **Have**; maps as optional Net later |
| Customer self-booking from home, public tracking page on the internet | Must be reachable from outside the office | Kumkaj Cloud (opt-in); LAN `/book` + `/track` **Have** |
| WhatsApp Business API automation, e-way bill generation, online payment links | Third-party online services | Device WhatsApp/SMS drain **Have**; e-way store/print **Have**; generation/API later |

---

## 9. UX lessons to apply to Moklanjo

1. **Make the rider app a three-tap app**: Delivered → photo/signature →
   COD amount confirmed. Big buttons; nothing to type while driving (Shipday
   complaint).
2. **Never lock a run**: stops can be re-ordered, added or dropped after the
   rider leaves (Track-POD complaint).
3. **Bulk actions beat drag-and-drop**: assign all dockets of an area to a
   rider in one action (Onfleet complaint).
4. **Every screen on every device**: keep one codebase so a phone can do what
   the desk can (LogiNext/Petpooja-style complaint "mobile lacks features").
5. **Money must be explainable**: every COD, deduction and margin traceable to
   a docket and a person (Shiprocket complaints about opaque deductions).
6. **Show sync state plainly** on the rider phone: "12 deliveries waiting to
   reach the office" — Moklanjo already does this for counters.
7. **Import, don't retype**: CSV previews with row-level errors for bulk
   bookings, carrier bills and pincode lists.

---

## 10. Sources

Round-ups and comparisons
- [Shipday — best courier software 2026](https://www.shipday.com/post/best-courier-software-route-planning-apps-2026) (vendor)
- [ClickPost — courier software guide](https://www.clickpost.ai/blog/courier-software)
- [Capterra — best courier software](https://capterra.com/courier-software/)
- [Circuit — Tookan vs Onfleet](https://getcircuit.com/teams/blog/tookan-vs-onfleet) (vendor)
- [G2 — Onfleet vs Tookan](https://www.g2.com/compare/onfleet-vs-tookan) · [Capterra — Onfleet vs Tookan](https://www.capterra.com/compare/142633-218952/Onfleet-vs-Tookan) · [G2 — Detrack vs Onfleet](https://www.g2.com/compare/detrack-vs-onfleet)

Group A
- [FR Billing on Techjockey](https://www.techjockey.com/detail/frbilling) · [Capterra](https://www.capterra.ca/software/1062127/frbilling)
- [Courier software listing (IndiaMART)](https://m.indiamart.com/proddetail/courier-software-2658099788.html)
- [Vyapar courier billing](https://vyaparapp.in/free/courier-billing-software) · [Vyapar reviews (Capterra)](https://www.capterra.in/reviews/180579/vyapar)
- [Delhivery franchise partner](https://www.delhivery.com/partner/local-delivery-franchise) · [DTDC EConnect (Shopify)](https://apps.shopify.com/dtdc-econnect)
- [Shiprocket reviews (Capterra)](https://www.capterra.com/p/159386/Shiprocket/reviews) · [ClickPost Shiprocket review](https://clickpost.ai/blog/shiprocket-review) · [Shiprocket WordPress plugin](https://wordpress.org/plugins/shiprocket/)

Group B
- [CXT Software (X Dispatch) on GetApp](https://www.getapp.com/courier-software/a/x-dispatch) · [Software Advice](https://www.softwareadvice.com/scm/x-dispatch-profile)
- [Datatrac](https://www.datatrac.com/) · [Datatrac Web on Capterra](https://www.capterra.com/p/79287/Datatrac-Web/)
- [Express-Pak on Capterra](https://www.capterra.com/p/79275/Express-Pak/)
- [e-Parcel on Capterra](https://www.capterra.com/p/10003965/e-Parcel/)
- [Elite EXTRA reviews (Capterra)](https://capterra.com/p/131389/Elite-EXTRA/reviews/) · [SoftwareConnect](https://softwareconnect.com/service-dispatch/elite-extra/)
- [Shipox features](https://shipox.dst.uz/features/) · [Shipox on Techjockey](https://www.techjockey.com/detail/shipox)
- [CourierManager on Techjockey](https://www.techjockey.com/detail/couriermanager) · [Courier Mobile](https://www.softwareadvice.com.au/software/254861/courier-mobile) · [Metafour](https://www.metafour.com/news/all-in-one-sameday-courier-software/)

Group C
- [Onfleet review (TechRadar)](https://www.techradar.com/reviews/onfleet) · [Onfleet (SelectHub)](https://selecthub.com/p/last-mile-delivery-software/onfleet) · [Onfleet (GetApp)](https://www.getapp.com/transportation-logistics-software/a/onfleet/)
- [Tookan auto-allocation help](https://help.jungleworks.com/?p=27488) · [Tookan geofencing](https://help.jungleworks.com/knowledge-base/geo-fencing/) · [Tookan reviews (Capterra)](https://www.capterra.com/p/218952/Tookan/reviews/?page=2)
- [Shipday on Capterra](https://www.capterra.ca/reviews/211323/shipday) · [Shipday on Software Advice](https://www.softwareadvice.com/ca/retail/shipday-profile/)
- [Detrack product](https://www.detrack.com/product) · [Detrack reviews (G2)](https://www.g2.com/products/detrack/reviews) · [Detrack offline](https://www.detrack.com/?p=2329)
- [Track-POD review (Upper)](https://www.upperinc.com/reviews/track-pod/) · [Track-POD (Gartner)](https://www.gartner.com/reviews/product/track-pod-1504766898) · [Track-POD (GetApp)](https://www.getapp.co.uk/software/116355/track-pod)
- [LogiNext Mile (Capterra)](https://www.capterra.in/software/155242/loginext-mile) · [LogiNext review (Circuit)](https://getcircuit.com/teams/blog/loginext-mile-reviews)

Group D
- [Shipsy (GetApp)](https://www.getapp.com/courier-software/a/shipsy-1) · [Shipsy driver app design](https://www.shipsy.ai/insights/driver-app-execution-design)
- [Bringg vs DispatchTrack (G2)](https://www.g2.com/compare/bringg-vs-dispatchtrack)

Our own docs
- [`MOKLANJO.md`](MOKLANJO.md) · [`USE_CASES.md`](USE_CASES.md) · [`MOKLANJO_CLIENTS_REQUIREMENTS.md`](MOKLANJO_CLIENTS_REQUIREMENTS.md)
- [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](../../../docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md) §6 (offline-first), §21 (free local, usage-based cloud)
