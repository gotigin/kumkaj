# moklanjo.com — website plan

> **Status:** plan, 2026-11-05. Nothing built yet.
> **Scope:** Moklanjo's instantiation of the Kumkaj-wide website plan
> [`docs/WEBSITES.md`](../../../docs/WEBSITES.md) — architecture
> (HTML first + Makepad stage), agent pipeline, budgets, hosting and
> roadmap live there and are not restated here. This file holds only
> what is Moklanjo's: audience, pages, the 3D story, tools and the
> claim sources.
> **Code:** `apps/moklanjo/web/` (Zola) and `apps/moklanjo/web/stage/`
> (Makepad stage crate). No code shared with any other site or app.

------------------------------------------------------------------------

## 1. Who it is for, and the promise

**Visitors:** courier and logistics agency owners; franchise counters
that hand parcels to national carriers; operators running their own
riders and vans; Moklanjo account managers presenting it in an office.

**Promise (headline candidates, English):**

- "Book. Label. Hand over. Track. Settle. One desk, internet or not."
- "A courier desk, not a cash register."

**Main actions:** **Download free** (Android · Windows · Linux) and
**WhatsApp for a demo** (prefilled: "Moklanjo demo — city, parcels a
day").

**What visitors already fear** (from
[`COURIER_RESEARCH.md`](COURIER_RESEARCH.md) §4) — each one gets an
answer on the home page:

| Fear | Answer on the site |
|---|---|
| Too many taps at the counter | Walk-in loop: next customer in one tap (chapter 1) |
| The office stops when the internet or vendor stops | Chapter 8 — booking, labels, scanning, COD and books keep running on the office Wi-Fi |
| Price per rider / user / order that taxes growth | Free forever on your devices; nothing counted per rider, user or parcel |
| The phone app lacks desktop features | One app on Android, Windows and Linux; a rider uses the same app in rider mode |
| Opaque COD money | Chapter 7 — expected vs handed-in per rider; collected vs paid to each consignor |
| Rigid flows | Rates, units, divisors, rounding are the shop's to change (vision §2) |

------------------------------------------------------------------------

## 2. Pages

| Path | Content | Notes |
|---|---|---|
| `/` | The parcel story (§3) | Stage: live / clip / still |
| `/features` | One page, anchored sections: Booking desk · Labels & scanning · Handover & own fleet · Tracking & proof of delivery · COD · Books & billing · Team & devices · Languages | Split into sub-pages only when a section outgrows the page |
| `/for/franchise-counter` | The agency counter: walk-ins, carrier number pools, bags, carrier bill reconcile, franchise commission | Mode 1 of the product shape ([`MOKLANJO.md`](MOKLANJO.md) "Product shape") |
| `/for/own-fleet` | Own riders and vans: pickups, rider runs, run sheet, rider mode, POD, COD settlement | Mode 2 |
| `/tools/volumetric-weight-calculator` | §4 | Search magnet; works with no JS |
| `/download` | Android (Play when listed, APK), Windows MSI, Linux; SHA-256; system needs; first-run steps (Owner PIN set-up lives here, not on the home page) | Reads `releases.toml` |
| `/guide` | Getting started, from [`MOKLANJO_APP_TUTORIAL.md`](MOKLANJO_APP_TUTORIAL.md) (rush-desk demo) and `docs/LAN_QUICK_START.md` | Screenshots from the real app |
| `/pricing` | Free forever on your devices; Kumkaj Cloud planned, usage-based | Links kumkaj.com/principles |
| `/present` | Presenter mode, works offline after one visit | `WEBSITES.md` §6 step 8 |
| `/privacy`, `/terms`, `/contact` | Legal + WhatsApp / email | Privacy first — Play needs it |

------------------------------------------------------------------------

## 3. Home story — "One parcel, from counter to settled"

**Stage:** a stylised, low-poly courier counter in Moklanjo colours —
navy desk, cool blue-grey walls, the orange parcel from the app icon.
The icon's motion streaks become the camera's trails between
chapters. The arc follows the product's own spine:

```text
Booking desk → AWB → (carrier bag | rider run) → track / POD → bill / COD
```

| # | Chapter | On stage | Says (draft) | Claim ids |
|---|---|---|---|---|
| 1 | **Walk-in** (hero) | The parcel slides onto the counter on the icon's speed lines. Three customers queue; each is booked in one beat and the desk resets — "Next booking" | "Book a parcel before the next customer reaches the counter." | `walkin-loop` |
| 2 | **Weigh it right** | Parcel on the scale reads 1.2 kg. A wireframe box grows around it to 30 × 20 × 15 cm: volumetric 30×20×15 ÷ 5000 = **1.8 kg**; chargeable rounds up to **2.0 kg**. A Domestic / International chip flips the divisor to ÷ 6000 for this booking only. **Toy:** drag length, breadth, height | "Actual or volumetric — whichever is higher, rounded your way." | `chargeable-weight`, `units` |
| 3 | **Price it** | Rate cards fan out: zone × service × weight slab; a consignor's own card slides over the default; a thin margin line shows sell vs carrier buy price | "Your rate cards, your party rates — and your margin on every docket." | `rate-master`, `party-rates`, `carrier-compare` |
| 4 | **Label it** | A thermal printer feeds a Code 128 label; the AWB comes from that carrier's number pool; the label lands on the box; a scanner beam reads it back | "AWB from your own number pools. Label, scan, find." | `awb-pools`, `label-scan` |
| 5 | **Two roads** | The parcel goes one of two ways: into a carrier bag with exactly the picked parcels, or onto a rider's run in your own van — split view | "Hand it to a carrier, or send your own rider. Same desk." | `manifest`, `rider-runs` |
| 6 | **Track and prove** | Timeline beads light: Booked → With carrier → Out for delivery → Delivered; a signature draws itself; a POD photo pins to the docket. A side branch: Failed → reason → Out again / Return to sender | "Every move recorded — and only moves that make sense." | `tracking-rules`, `pod`, `ndr-rto` |
| 7 | **Money comes home** | The rider's COD cash flows back: expected vs handed-in. It lands in the COD register, then on each consignor's account; the day book line posts; month-end period bill stacks | "COD you can see: expected, handed in, owed to each consignor." | `cod-register`, `rider-settle`, `day-book`, `period-bill` |
| 8 | **Internet down? Keep booking.** | The office router's light turns red. The desk keeps booking; a second counter and a rider phone stay joined over the office Wi-Fi | "No internet needed. Extra counters join over your own Wi-Fi." | `offline-lan` |
| 9 | **Close** | Counter at rest, label printer quiet | "Free forever on your devices." → Download · WhatsApp | `free-local` |

**Props built in code** (`WEBSITES.md` §6 step 2): counter, scale with
digit display, parcel (taped box like the icon), wireframe dimension
box, rate cards, thermal printer + label strip, handheld scanner,
carrier bag (plain colours — no carrier marks), van body, rider phone,
timeline beads, cash bundle, ledger page, router with status light.
glTF only for the rider and walk-in figures (simple, faceless,
stylised).

------------------------------------------------------------------------

## 4. Volumetric weight calculator (`/tools/volumetric-weight-calculator`)

- **HTML form first:** length, breadth, height (cm or inches), actual
  weight (kg or lb), divisor (5000 domestic default · 6000 air · any
  number the visitor types — carriers differ), round-up step (0.5 kg
  default, editable). Output: volumetric weight, chargeable weight,
  and one line of working ("30 × 20 × 15 ÷ 5000 = 1.8 kg → 2.0 kg").
- **Stage toy on top** when live: the same box as chapter 2, resized by
  the form.
- Runs entirely in the browser; nothing is sent anywhere.
- Same rules as the app's booking desk ([`MOKLANJO.md`](MOKLANJO.md)
  "Chargeable weight"; `docs/UNITS_AND_CONVERSIONS.md`), re-implemented
  in a few lines of JS for the page (copy, not import), with tests
  that pin the same worked examples as `booking.rs`.
- Page copy explains volumetric weight plainly; ends with "Moklanjo
  does this on every booking" → Download.

------------------------------------------------------------------------

## 5. Claim sources (seed for `content/claims.toml`)

Every source below is a **Ship** row in [`MOKLANJO.md`](MOKLANJO.md)
unless noted. The claims check (`WEBSITES.md` §6 step 5) re-reads them.

| Claim id | Sentence on the site | Source row |
|---|---|---|
| `walkin-loop` | Next customer in one tap | B "Walk-in flood" |
| `chargeable-weight` | Actual vs volumetric, ÷5000 domestic / ÷6000 international per booking, rounded up 0.5 kg — all editable | B "Chargeable weight" |
| `units` | Enter in your units; conversions are yours to set | B "Measurement units" |
| `rate-master` | Zone × service × slab rate cards | B "Rate master" |
| `party-rates` | Per-consignor rate cards | B "Party-specific rates" |
| `carrier-compare` | Sell vs carrier price and margin at the desk | B "Carrier rate compare"; C "Carrier cost / margin on docket" |
| `awb-pools` | AWB number pools per carrier | B "Carrier AWB / barcode number pools" |
| `label-scan` | Code 128 labels; scan to find and act | B "AWB / CN + label / barcode"; C "Barcode scan-to-find / act" |
| `manifest` | Carrier bag with exactly the picked parcels | C "Manifest / handover list" |
| `rider-runs` | Own riders, run sheets, rider mode on the same app | C "Riders / vans", "Printable DRS / run sheet", "Rider app mode (same binary)" |
| `tracking-rules` | Status timeline; impossible moves refused | C "Tracking statuses on AWB" |
| `pod` | POD photo and signature | C "POD note / photo", "Signature capture (POD)" |
| `ndr-rto` | Failed delivery reasons, out again, return to sender | C "NDR / failed delivery / RTO" |
| `cod-register` | COD register and remittance | C "COD register + remittance" |
| `rider-settle` | Rider COD expected vs handed in, commission slip | C "Rider COD + pay settlement" |
| `day-book` | Day book, bank book, AR/AP, P&L | A rows |
| `period-bill` | Unbilled account dockets → one period bill | C "Period party bill" |
| `gst-csv` | Month GST summary CSV for your accountant (not filing) | C "Month GST summary CSV" |
| `tally` | Tally voucher export | C "Tally voucher export" |
| `track-link` | Customers can check an AWB on the office Wi-Fi | C "Public track-by-AWB" — **LAN only; say so** |
| `whatsapp-receipt` | One-tap WhatsApp receipt | C "One-tap WhatsApp receipt" |
| `offline-lan` | Works with no internet; extra counters on the office Wi-Fi | "Doctrine follow-ups → Offline office + devices"; vision §6, §20 — the extra-counters half is gated `lan-pilot` (`WEBSITES.md` §6 step 5, §11); see §6 below |
| `languages` | 29 languages, Arabic/Urdu right-to-left | "Languages & RTL" |
| `free-local` | Free forever on your devices | vision §21 |

------------------------------------------------------------------------

## 6. Do not say (until the source changes)

- **Hardware specifics** — printer models, scanners, "tested with …".
  The office/devices work is in code but "not yet run on real
  hardware" ([`MOKLANJO.md`](MOKLANJO.md) "Offline office + devices";
  `docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md`). Say "prints to your
  receipt and label printers" only after the LAN pilot passes.
- **Live GPS maps, AI routing, cloud ETA** — progressive depth /
  cloud, not shipped.
- **SMS delivery** — planned (`docs/SMS_MESSAGING.md`); WhatsApp share
  and copy-SMS text ship.
- **Public tracking over the internet** — tracking pages are on the
  office Wi-Fi only.
- **"Generate & print" dual button on period bills** — listed as a gap.
- **Statutory e-invoicing, GST filing, e-way bill generation** — never
  (docket e-way fields are print-only).
- **iOS / macOS** — deferred.
- **Carrier or competitor names and logos** — none, in copy or on 3D
  props (`WEBSITES.md` §12 item 6).
