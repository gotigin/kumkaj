# Kumkaj — universal business books spine

Canonical checklist of what **most entities** must track, mapped to **generic
primitives** only. Concrete feel comes from start templates and labels — never
vertical product forks.

Companions: [`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
§1–2, §7, §14–16, §23; [`DESIGN.md`](DESIGN.md); [`.cursor/rules/generic-concrete.mdc`](../.cursor/rules/generic-concrete.mdc);
schema in [`DEV.md`](DEV.md) (**v16**).

**Product center:** a **business operating system** UX for what **most**
entities need on their books — not a till that happens to have reports, and
not a forced chart-of-accounts ERP.

------------------------------------------------------------------------

## Cross-category research → commons

Kumkaj is for **any entity** that sells or keeps books — worldwide, any
trade, size, or ownership shape. It is **not** limited to the rows below.

The table is only a **non-exhaustive sample** used to compare the same
fields and find the **intersection**. Pharmacy, hardware, tuition,
workshop, clinic, wholesale, farm stall, NGO books, multi-branch groups,
and every other entity still map to the same primitives (party, item,
money in/out, ledger, day/period). Concrete feel comes from labels and
capability packs — never from adding that trade to an allowlist.

Goal of the sample pass: what **most** entities share vs what is a
**capability pack**.

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

**Many (same primitives, progressive):** stock tracking, supplier dues (AP), bank posts, team/roles, multi-branch.

**Few / capability packs (hide until enabled):** tables & KOT/KDS, kg-heavy units + account/cash booking labels, appointments, multi-station kitchen.

**Conclusion for shell:** do **not** open on Sell alone. Shipping IA is
**Books · Sell · Buy · More** — overview home; money in and money out as
peers; catalogue, credit, inventory detail, floor/kitchen, team, backup under
More (and Sell/Buy as needed). See [`DESIGN.md`](DESIGN.md).

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
3. **Their language** — Start template labels (Parties, Account booking,
   kg, …) so a courier or kirana recognizes their world without a vertical
   product fork.
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

## Universal checklist → primitive → status

| # | What owners track | Generic primitive | Status (schema v1) |
|---|-------------------|-------------------|---------------------|
| 1 | Who we deal with | **Party** + freeform `kind_label` | Shipped — Sell credit, Buy supplier/carrier, Credit kinds |
| 2 | Catalogue + units | **Item** + unit + `units` / `unit_conversions` | Shipped; Settings → Units; item `unit_label` |
| 3 | Sell (money in) | Sale lines, pay, invoice, bill-type tag, amend/void/refund | Shipped; **Books · Sell · Buy · More** shell |
| 4 | Buy (goods/services in) | Purchases → stock + line cost; party + on-account AP | Shipped — Buy tab; paid or on account |
| 5 | Running costs | **Expenses** | Shipped (Buy tab + Reports) |
| 6 | Who owes us (AR) | Credit ledger + aging + write-off | Shipped |
| 7 | Whom we owe (AP) | Payable ledger + settle + write-off + aging | Shipped |
| 8 | Cash and bank | `money_movements` + day-close drawer count | Shipped; BRS = later |
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

## What to tell a courier agency (BOS in their language)

Courier owners are **not** a special product. Describe the **whole book**, not
a sequence:

> “This is your business books. Same place for carriers and consignors, your
> ₹/kg rates, money you charge and money you pay out, who still owes whom,
> bank, and the day/month picture — Account booking or Cash booking on each
> bill. It is not a courier-only app and not only a sell screen.”

| Their world | In Kumkaj |
|-------------|-----------|
| Carriers (Delhivery, DTDC, …) | Party kind `Carrier` — money out / cost |
| Consignors (Sumit, Bangles King, …) | Party kind `Consignor` — money in / AR |
| Rates ₹/kg | Items with unit `kg` |
| Charge a consignor | Sell · Account booking or Cash booking · bill |
| Pay a carrier / office cost | Buy (purchase on account / expense) · Credit → Pay AP |
| Day / outstanding / bank / month | Books · Day close · Reports · aging · bank posts |

Spine shipped (v16): COGS in period books, supplier/carrier as party + AP
settle/write-off, AR write-off, drawer count on day close, units + conversions.
Shell peers (Books / Sell / Buy) match the category research above.

------------------------------------------------------------------------

## Next coding backlog (priority)

Still generic-only. Spine above is shipped; remaining is depth.

### Later
Stock valuation · bank reconcile (BRS) · open-item bill allocation · balance sheet ·
accountant export · statutory GST/e-invoice (vision §23).

------------------------------------------------------------------------

## Depth extracted from external research (safe after deleting that file)

Condensed from India SMB / Tally / Zoho-style research and cross-category
bookkeeping patterns (retail vs service). **Not** our build order.
Use when implementing a row above or a deferred compliance feature.

### Design ideas worth keeping
- **Document vs money effect:** keep the source bill (lines, party, tax) and
  derive books totals from it; do not maintain a second handwritten total.
- **Immutable history:** correct by amend / void / credit note — not silent edit.
- **Bill-wise AR/AP:** match receipts/payments to specific bills (Tally
  “bill-wise”); we have party balance today, open-item allocation is later.
- **FY and number series:** configurable FY start; gap-free invoice series per
  period (we have `INV-BRANCH-seq`; FY-scoped series is later).
- **Stock valuation:** FIFO or weighted average when we value stock (LIFO not
  for India reporting). Perpetual stock is the default path.
- **Period lock:** after a period is closed / return filed, lock edits; we have
  day close audit, not full period lock yet.
- **Audit trail:** append-only edit log; India company law expects non-disableable
  trails — strengthen as we harden, don’t disable.
- **Retail vs service emphasis:** retail stresses stock + COGS; services stress
  AR and costs without inventory — same shell, progressive stock.

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
   step order, not POS-only, not full Tally on day one.
2. Vertical language → shared primitive; never a courier/food fork.
3. This file is the spine.
4. Do not edit USER_CASES / USE_CASES workbooks — those are owner-only.
