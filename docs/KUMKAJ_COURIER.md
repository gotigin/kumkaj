# Kumkaj Courier — current product focus

**Product:** Kumkaj Courier (`kumkaj-courier`)  
**Brand:** Kumkaj · kumkaj.com  
**Role:** concrete **courier / logistics-agency business operating system** — books **and** booking-desk ops, including **handover to named carriers** *and* **own-fleet rider dispatch**.

Companions: [`USE_CASES.md`](USE_CASES.md) (industry + client research), [`grokbot/COURIER_BUSINESS_TUTORIAL.md`](grokbot/COURIER_BUSINESS_TUTORIAL.md) (ops + tax + Part 10 mapping), [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) (shared books spine), vision §1–2.

**Build rule:** implement this product concretely first. Promote logic into shared crates only when another Kumkaj product needs the same thing. Prefer party / line / unit / charge / ledger / day book / document shapes so extraction stays possible.

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

| Feature | Notes | Kumkaj Courier target |
|---------|--------|------------------------|
| Consignor + carrier accounts | One party; roles Consignor and/or Carrier (same firm may be both) | **Ship** — Parties multi-role; AR+AP ledgers; net position in reports |
| Debtors & creditors / aging | AR + AP (separate); net who-owes-whom | **Ship** — Parties + Reports |
| Day book | Today money in/out | **Ship** — Day book tab / Reports |
| Bank book | Till ↔ bank | **Ship** — Bank book & P&L |
| Expense voucher | Running costs + bill photo/PDF | **Ship** — Pay out / Reports + attachments |
| Entry notes / attachments | Docket photos, party docs, expense bills | **Ship** — notes + WebP images + PDF on sale / party / expense |
| Invoice | Account booking path | **Ship** — Booking + Account booking |
| Cash booking vs Account booking | Pay now vs party ledger | **Ship** — bill-type tag |
| Party ledger + outstanding | Running balance | **Ship** / deepen |
| P&L / monthly | Period books | **Ship** — Reports month |
| Staff rights | PINs / roles | **Ship** (Team) |

### B. Booking desk (must-have for “real courier app”)

| Feature | Notes | Target |
|---------|--------|--------|
| Consignment / docket booking | Party, destination, weight, service + receipt | **Ship** — Booking desk |
| Walk-in flood | Clear cart / “new booking” every customer | **Ship** — Next booking |
| Rate master (zone × service × slab) | ₹/kg, first/extra slab | **Ship** — zone master + first/extra kg on Rates |
| Chargeable weight | Actual / volumetric / slab | **Ship** — defaults L×B×H ÷ 5000, round up 0.5 kg; divisor + slab are shop settings |
| Measurement units | User-chosen entry units, standard + custom conversions | **Ship** — Settings → Booking measures: dimensions/weight in cm, m, in, ft · kg, g, lb, oz, or a custom unit linked to cm/kg directly or through other custom units (Settings → Units; loops and weight↔length links refused). Books always record kg |
| Extra charges | Fuel, docket, ODA, packing, COD fee | **Ship** — fuel/docket on desk; catalog seeded |
| AWB / CN + **label / barcode** | Unique id; print; scan | **Ship** — Code 128 SVG label + Ops Find/Scan (wedge) → desk card → advance/COD |
| City / zone / pincode | Destination masters | **Ship** — zone master + booking zone field |
| Bulk booking | Many dockets | **Ship** — CSV parse (`notify::parse_bulk_booking_csv`) |
| Credit limit on account | Block over-limit | **Ship** — `credit_limit_paise` + gate on finalize |

### C. Ops (carrier handover **and** own fleet)

| Feature | Target |
|---------|--------|
| Tracking statuses on AWB | **Ship** — timeline events + status advance |
| Barcode scan-to-find / act | **Ship** — Ops AWB field + Find (scanner Enter); card + OFD / Delivered / label / COD |
| POD note / photo | **Ship** — note + WebP POD on sale (`pod_ref` = attachment id); ops desk picker |
| Manifest / handover list | **Ship** — carrier bag **or** own-fleet rider run |
| Riders / vans | **Ship** — riders master + assign on manifest |
| Pickup / dispatch lists | **Ship** — same manifest + status filters |
| COD register + remittance | **Ship** — COD entries + remit batch |
| Carrier cost / margin on docket | **Ship** — `carrier_cost_paise` + margin helper |
| Period party bill | **Ship** — unbilled account dockets → one invoice |
| Consignment search/filters | **Ship** — AWB / party / date / billed / status |
| WhatsApp / SMS notify hook | **Ship** — outbox queue (connector later) |
| Public track-by-AWB | **Ship** — `GET /track/{awb}` on LAN server (no pairing code) |
| Aggregator rate compare | Optional later |
| Live GPS map / AI routes | Progressive depth on dispatch spine |

### D. Good features competitors have that we should not skip

Prioritize in this order for Kumkaj Courier:

1. ~~**Booking desk** (walk-in loop + weight + cash/account)~~ **done**
2. ~~**AWB + barcode/label print**~~ **done** (SVG label + text AWB)
3. ~~**Rate/zone/slab** deepen~~ **done**
4. ~~**Period party bill**~~ **done** (store API + Reports)
5. ~~**Tracking + manifest** (carrier **and** own fleet)~~ **done**
6. ~~**COD register**~~ **done**
7. ~~**Carrier cost / margin**~~ **done** (CSV reconcile later)

------------------------------------------------------------------------

## Shell (courier names)

| Tab | Courier meaning |
|-----|-----------------|
| Day book | Today’s position |
| Booking | Money in — book parcels |
| Pay out | Money out — carrier bills & expenses |
| Rates · Ops · COD · Riders · Sales · Day close · Reports · Parties · Team · Settings · Lock | Former More destinations — first-class bottom/rail peers (scrollable) |

**Rearrangeable tabs.** The tab order is the shop's, not ours: press a tab and drag it onto another to swap them (bottom bar and rail alike). The arrangement is saved in the books (`ui_nav_order`), so it survives reinstall and follows the shop, not the device. Scrolling stays one gesture away: the bars keep whitespace lanes around the tabs — swipe the lane to scroll, grab a tab to drag. Holding a dragged tab against the left/right edge of the bar (top/bottom of the rail) auto-scrolls it, so tabs that start off-screen can still be reached and swapped with. Implementation rules that keep this working are in `docs/DEV.md` (“Nav drag-to-swap”).

------------------------------------------------------------------------

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
| `media.rs` | JPG/PNG/WebP → always store WebP; used by attachments + catalogue/brand photos |

**Desk UI:** Ops · COD · Riders are first-class bottom/rail peers that open the Reports screen with the matching facet. **Barcode loop:** scan/type AWB → Find → desk card → Out for delivery / Delivered / Print label / Collect COD. Carrier/own-fleet dispatch, COD remit, rider add, and period bill run from the same desk; credit limit + period bill also sit on Parties.

------------------------------------------------------------------------

## Client WhatsApp checklist (must remain covered)

Canonical sanitized tracker: [`COURIER_CLIENTS_REQUIREMENTS.md`](COURIER_CLIENTS_REQUIREMENTS.md).

- Per day report · kg + credit/debit · party outstanding · bank posting · monthly · P&L / BS · invoice format  
- Named carriers & consignors as **parties** (owner enters their own names)

------------------------------------------------------------------------

## Out of scope (for now)

- Replacing Delhivery/DTDC as a **national network** (hub TMS competing with carriers)  
- Claiming statutory e-invoicing / GSTR filing before implemented and reviewed  
- Pre-loading sample rates or parties  
- AI route optimisation / live GPS fleet map as v1 must-haves (same spine; ship later)
