# Kumkaj — business books: common concepts checklist

Canonical checklist of books most concrete Kumkaj products need, mapped to
common **concepts** (party, line, unit, charge, ledger, day/period). These
are shared *requirements and vocabulary* only — **not shared code**. There
is no Kumkaj core / shared books engine (vision §1): each app implements
these concepts in its own code (copying a sibling app as a starting point,
then diverging), so a change in one app never impacts another.

**Current product focus:** [`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md).

Companions: [`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
§1–2; [`IDENTITY_AND_NAMES.md`](IDENTITY_AND_NAMES.md) (UUID identity —
every concept below is keyed by `id`, never by name);
[`UNITS_AND_CONVERSIONS.md`](UNITS_AND_CONVERSIONS.md);
[`DESIGN.md`](DESIGN.md); [`.cursor/rules/generic-concrete.mdc`](../.cursor/rules/generic-concrete.mdc);
schema in [`DEV.md`](DEV.md).

**Not:** a till with reports bolted on, or a forced chart-of-accounts ERP.

------------------------------------------------------------------------

## Cross-category research → commons

The table below is a **sample** used to find the intersection of books
fields across trades. Kumkaj ships **concrete products**; this checklist
is the common BOS requirement pattern that explains why the apps share a
workspace and a Makepad 2 platform. When two products need the same
thing, each implements it in its own code — nothing here is promoted
into shared code.

### Sample snapshot (comparable fields — not a product catalog)

| Sample entity (illustrative) | Money in | Money out | Parties | Catalogue / units | AR / AP | Cash vs bank | Day ask | Optional pack |
|----------|----------|-----------|---------|-------------------|---------|--------------|---------|---------------|
| Tea / snack counter | Fast tap sell | Buy stock, small expenses | Regulars on credit | Few items, piece | AR common; AP light | Cash + rail | Sales, owed, cash | — |
| Pushcart (fruit / goods) | Sell by piece/weight | Restock buy | Neighbours / regulars | Units (kg/piece), light stock | AR sometimes | Mostly cash | Day take, restock cost | — |
| Grocery / kirana | Counter sell | Supplier purchases | Customers + suppliers | Dense catalogue, stock | AR + supplier dues | Cash + bank | Margin, stock, owed | — |
| Cloth / perfume / shop | Counter sell | Purchases | Customers; wholesalers | SKUs, sizes/variants | AR; AP to wholesaler | Cash + bank | Sales, stock, dues | — |
| Salon / service | Service charge | Supplies, rent, pay | Clients | Services as items (often no stock) | AR / packages | Cash + bank | Bookings optional | Appointments later |
| Restaurant / café | Sell + tables | Purchases, expenses | Diners; suppliers | Menu, modifiers | AR light; AP | Cash + bank + drawer | Day close, food cost | **Tables & kitchen** |
| Courier / agency | Charge consignors | Pay carriers / costs | Consignor + carrier kinds | Rates × kg | AR + AP both | Cash + bank | Outstanding, day book | Units `kg`; bill-type tags |
| Personal / household | Income entries | Spend / buy | People | Categories as items | On-account | Cash + bank | Month in/out | — |
| *(any other entity)* | Same fields | Same fields | Party + `kind_label` | Item + optional unit/stock | AR and/or AP as needed | Cash vs bank | Day / period books | Enable only what they need |

### Intersection (what drives the product — for every entity)

**Most entities (core shell must surface these as peers):**

1. **Books at a glance** — today’s money in, money out, who owes / whom we owe, cash vs bank
2. **Money in** — sell / charge (lines, pay, credit, amend)
3. **Money out** — buy goods/services and/or expense (not buried under stock-only UI)
4. **Parties** — who we deal with (`kind_label` for concrete feel)
5. **Catalogue** — items/rates/services + optional units
6. **Day / period picture** — tenders, outstanding, profit shape

**Many (same concepts, progressive — implemented per app):** stock tracking, supplier dues (AP), bank posts, team/roles, multi-branch.

**Few / capability packs (hide until enabled):** tables & KOT/KDS, kg-heavy units + account/cash booking labels, appointments, multi-station kitchen.

**Conclusion for shell:** do **not** open on Booking/Sell alone. Peers:
**Day book / Books · Booking/Sell · Pay out/Buy · More**. Courier names:
[`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md). See [`DESIGN.md`](DESIGN.md).

------------------------------------------------------------------------

## BOS user experience (what “good” means)

Owners should feel they are running **their books**, not hunting features
inside a Sell screen. Buy vs sell is not a tutorial sequence — both are
equal peers in one mental model. The UX must make that model obvious.

### Experience principles

1. **Books at a glance** — After unlock, the owner lands on **Books**: today’s
   position (money in, money out, who owes / whom we owe, cash vs bank) —
   not only a catalogue grid waiting for a tap.
2. **Equal peers, not a funnel** — Sell and Buy are bottom-nav peers with
   Books. Parties, items, reports, inventory depth live under More (or one tap
   from Books) — not afterthoughts that redefine the product as a till.
3. **Their language** — Concrete product labels (Parties, Account booking,
   kg, …) so the owner recognizes their world without a vague generic UI.
4. **Empty states that teach the model** — Empty Sell / Buy / Items / Parties /
   Reports explain *what this book is for* in one short line (e.g. rates,
   who you deal with, money in and out) — not “add something so we can sell.”
5. **One answer to “how did we do?”** — Day close and period books answer
   profit and cash without the owner assembling it from five screens.
6. **Progressive depth** — Hide unused capability; never rename the whole
   product into one trade; never force GL/CoA on day one.

### The four questions the UI must answer (anytime)

1. Can I record money **in** and money **out**?
2. Do I know **who owes me** / **whom I owe**?
3. Do I know **cash vs bank** today?
4. Do I know **profit** (revenue − costs − expenses) for the period?

```text
                 ┌── Sell (money in)
  Parties/Items ─┤
                 ├── Buy / Expense (money out)
                 └── Settle · Bank · Day/Period books → Profit

  Shell:  Books (home) · Sell · Buy · More
```

------------------------------------------------------------------------

## Universal checklist → concept → status

| # | What owners track | Common concept (each app implements its own) | Status (schema v1) |
|---|-------------------|-------------------|---------------------|
| 1 | Who we deal with | **Party** + multi-role `kind_label` (one identity; AR+AP; net report) | Shipped — dual-role OK; see Dual-role parties |
| 2 | Catalogue + units | **Item** + unit + `units` / `unit_conversions` | Shipped; Settings → Units; item `unit_label` |
| 3 | Sell / Booking (money in) | Sale lines, pay, invoice, bill-type tag, amend/void/refund | Shipped shell; Courier desk UX next — [`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md) |
| 4 | Buy (goods/services in) | Purchases → stock + line cost; party + on-account AP | Shipped — Buy tab; paid or on account |
| 5 | Running costs | **Expenses** | Shipped (Buy tab + Reports) |
| 6 | Who owes us (AR) | Credit ledger + aging + write-off | Shipped |
| 7 | Whom we owe (AP) | Payable ledger + settle + write-off + aging | Shipped |
| 8 | Cash and bank | `money_movements` + day-close drawer count | Shipped — perpetual till (`till_balance_paise`), day-close in−out, BRS tick-off (`cleared_at`; book vs cleared balance) |
| 9 | Stock | Qty + movements | Shipped; valuation = later |
| 10 | Tax on bills | Rates + wording | Shipped; filing/e-invoice = deferred |
| 11 | Day / period / P&L | Day close, `PeriodBooks` (rev, COGS, gross, expenses, net) | Shipped |
| 12 | People, audit, backup | Roles, audit, backup | Shipped |
| 13 | When it happened | Triple timestamps (`user_at` / `device_at` / `server_at`) | Shipped — schema v1 |

### Triple timestamps (books rows)

Every money / stock books entry keeps **three** clocks — not one overloaded `created_at`:

| Field | Meaning |
|-------|---------|
| **`user_at`** | Business time the owner entered or meant (backdatable). Drives day books. |
| **`device_at`** | Local device clock when the row was written offline. |
| **`server_at`** | When the cloud accepted the upload; **empty** until sync. |

`created_at` remains on the row and mirrors `user_at` so existing day filters keep working. Applies to sales, credit/payable ledgers, purchases, expenses, money movements, and stock movements.

### Bill-type wording (not “white/black books”)

One sales table + a tag:

| Template | Shareable / account path | Spot / cash path |
|----------|--------------------------|------------------|
| Courier agency | Account booking | Cash booking |
| Others | Tax invoice | Cash memo |

------------------------------------------------------------------------

### Ground-truth settlement (how money actually moves)

Doctrine: vision §18, *“Ground truth: record money as it actually
moved.”* Mechanics for whoever builds a money feature:

| Reality | Books pattern (per app) |
|---------|-----------|
| One obligation settled part bank / part cash (or any mix) | **One** books entry (sale / purchase / expense / settlement) + **N** `money_movements`, one per tender, summing to the entry total |
| Part of a settlement has a bill / receipt / bank trail, part does not | Supporting document is an **optional** attachment / note on the entry. Validation never requires a bill no, cheque no, or bank reference to save |
| Cash-settled part | A first-class movement with tender `cash`, same weight as bank — day book, P&L, aging, and party balances all include it by default |
| Till must tally | Expected cash = Σ actual cash movements. Day-close counted-vs-expected only works if every cash movement was recorded, so entry flows must make cash as easy to record as bank |
| Owner shares a document | Invoice / receipt / statement is generated from the record for the scope the owner picks, and states that scope. Nothing is fabricated for money that did not move |
| Filing / statutory view (later) | A derived, explicitly-scoped presentation over the same rows — never a second, divergent ledger, and never allowed to shrink the owner's own totals |

Wording: tender / channel (`cash`, `bank`, `upi`, `card`, `credit`),
“with / without a supporting document” when that distinction is
needed. Never slang or moral labels for a settlement, and never
gatekeeping copy (“bill required”, “bank proof required”) on a money
form.

------------------------------------------------------------------------

## Courier agency — product is Kumkaj Courier

Full product brief, competitor matrix, walk-in/barcode/tracking:
[`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md).

> “This is your courier business system: carriers and consignors, ₹/kg rates,
> booking (Account or Cash), who owes whom, bank, day/month P&L — and the
> booking desk (walk-ins, AWB/barcode, tracking) as we ship those ops.”

| Their world | In Kumkaj Courier |
|-------------|-------------------|
| Carriers / consignors | **One party master** with role tag(s) — see dual-role rule below |
| Rates ₹/kg | Rates & services (unit `kg`) |
| Charge a consignor | Booking · Account or Cash booking → **AR** (`credit_ledger`) |
| Pay a carrier / office cost | Pay out · Parties → Pay AP → **AP** (`payable_ledger`) |
| Day / outstanding / bank / month | Day book · Bank book & P&L · party **net** position |

### Dual-role parties (decision)

A peer courier (or any firm) may be **both** someone you bill **and** someone who bills you.

| Layer | Practice |
|-------|----------|
| **Identity** | Uniqueness is the party **UUID** (`id`); names are aliases only — display names are **not** unique and may be renamed (canonical: [`IDENTITY_AND_NAMES.md`](IDENTITY_AND_NAMES.md)). Prefer **one UUID** for one real firm with dual roles, not two rows that only differ by kind |
| **Roles** | Tags on that party (`Consignor`, `Carrier`, `Supplier`, …). Multiple roles allowed (e.g. `Carrier, Consignor`) on the **same** id |
| **Ledgers** | Keep **two** books faces: AR (`credit_ledger`) and AP (`payable_ledger`). Do **not** merge into one running balance |
| **Overall status** | Report **AR, AP, and net** (`AR − AP`): net > 0 → they owe you; net < 0 → you owe them |
| **Settlement** | Settle each side on its ledger; optional cash netting later still posts both sides |

`kind_label` is the freeform / multi-role field (roles, not product forks). All books and ops key off **party id**, never uniqueness-by-name — the general Kumkaj identity rule (UUID decides, names are aliases) is canonical in [`IDENTITY_AND_NAMES.md`](IDENTITY_AND_NAMES.md).

### Entry notes + attachments

Sales, parties, expenses (and tracking/POD) may carry **notes**, **photos**, and **PDFs** via `entry_attachments`:

| Kind | Input | Stored |
|------|--------|--------|
| Note | Free text | DB row only (`kind = note`) |
| Image | JPG / JPEG / PNG / WebP | Always **WebP** under `attachments/{entity}/{id}/` |
| File | PDF / txt | Copied as-is |

Catalogue / brand photos reuse that app's own WebP encode helper. Reject unknown types (e.g. `.exe`).

------------------------------------------------------------------------

## Next coding backlog (priority)

Checklist above is shipped in Courier (and copied/diverged in Khaynjo's own code); remaining is depth.

### Later
Stock valuation · statutory GST/e-invoice (vision §23).

Shipped since: bank reconcile (BRS tick-off against the statement) ·
open-item bill allocation (linked settle, FIFO for the rest) ·
spreadsheet CSV export (day book / outstanding / parties / items /
party ledger) + CSV import (parties, items) · expense tenders
(`expense_payments` — one expense, several tenders; ground truth §18).

**Operating balance sheet** (bank + till + AR vs AP + net) shipped for courier client books —
not a statutory Schedule III statement.

------------------------------------------------------------------------

## Depth extracted from external research (safe after deleting that file)

Condensed from India SMB bookkeeping research and cross-category
patterns (retail vs service). **Not** our build order.
Use when implementing a row above or a deferred compliance feature.

### Design ideas worth keeping
- **Document vs money effect:** keep the source bill (lines, party, tax) and
  derive books totals from it; do not maintain a second handwritten total.
- **Immutable history:** correct by amend / void / credit note — not silent edit.
- **Bill-wise AR/AP:** match receipts/payments to specific bills
  (bill-wise allocation); we have party balance today, open-item allocation is later.
- **FY and number series:** configurable FY start; gap-free invoice series per
  period (we have `INV-BRANCH-seq`; FY-scoped series is later).
- **Stock valuation:** FIFO or weighted average when we value stock (LIFO not
  for India reporting). Perpetual stock is the default path.
- **Period lock:** after a period is closed / return filed, lock edits; we have
  day close audit, not full period lock yet.
- **Audit trail:** append-only edit log; India company law expects non-disableable
  trails — strengthen as we harden, don’t disable.
- **Retail vs service emphasis:** retail stresses stock + COGS; services stress
  AR and costs without inventory — same checklist, progressive stock in each app's own shell.

### India compliance (deferred — config rates, don’t hard-code)
- GST: place of supply → CGST/SGST vs IGST; HSN/SAC on items; tax invoice fields
  (Rule 46); bill of supply / composition later.
- Returns data later: GSTR-1 / 3B exports, GSTR-2B vs purchase register, IMS.
- E-invoice (IRN/QR) and e-way bill via GSP when AATO thresholds apply — vision §23.
- TDS/TCS: nature master with **effective-dated** section codes (Income-tax Act
  2025 mappings from Apr 2026); not day-1 local POS.
- MSME / 43B(h) overdue vendor payment warnings — AP exists; warnings later.
- Cash expense limits warnings — later.

### Explicitly not day-1 Kumkaj
Full chart of accounts, double-entry GL as the only truth UI, payroll, fixed
assets, multi-currency FX, manufacturing BoM, Schedule III balance sheet.

Those stay research depth for “most enhanced” entities — progressive enablement —
not the common-books center.

------------------------------------------------------------------------

## Agent rule

1. Optimize for **BOS UX** (books model clear in the shell) — not literal
   step order, not POS-only, not a full GL accounting package on day one.
2. Vertical language → common concept, implemented in that app's own
   code; copying a sibling app is welcome, extracting shared code is not.
3. This file is a concepts checklist, not a shared-code spine — there is
   no Kumkaj core (vision §1).
4. Do not edit USER_CASES / USE_CASES workbooks — those are owner-only.
