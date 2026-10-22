# Courier client requirements (discovery)

Sanitized checklist from a **courier / logistics-agency** discovery chat (Oct 2026).  
**No personal names, phones, chats IDs, or download links** — only product asks.

Companions: [`KUMKAJ_COURIER.md`](KUMKAJ_COURIER.md), [`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md), [`grokbot/COURIER_BUSINESS_TUTORIAL.md`](grokbot/COURIER_BUSINESS_TUTORIAL.md).

**Scope:** this is Courier's discovery evidence and status tracker —
it stays in Courier's docs. The general BOS needs underneath §A–B
(party accounts, day/period books) are Kumkaj-wide concepts canonical
in root `BUSINESS_BOOKS.md`; do not copy this checklist into another
app's docs, and do not treat a courier client's asks as another app's
requirements without that app's own discovery.

Voice messages in the source chat were **ignored**; only typed requirements are tracked here.

------------------------------------------------------------------------

## Who they are (role, not identity)

- Local **courier agency / franchise counter** (books + desk).
- Deals with **named carriers** (e.g. national networks) and **their own consignors / shippers**.
- Expects proper business books: parties, day book, bank, expenses, invoices, P&L, outstanding.

------------------------------------------------------------------------

## Typed must-haves

### A. Party accounts

| Ask | Meaning | Status |
|-----|---------|--------|
| Courier company accounts | Carriers as parties (owner types names) | **Done** — Parties · Carrier / multi-role |
| Our party accounts | Consignors / shippers as parties | **Done** — Parties · Consignor / multi-role |
| Debtors and creditors | AR (they owe us) and AP (we owe them) | **Done** — separate ledgers + net position |

### B. Day / period books

| Ask | Meaning | Status |
|-----|---------|--------|
| Per day report / day book | Today money in/out | **Done** — Day book · Reports → Today |
| Kgs + credit / debit | Day view with chargeable kg and cash vs credit | **Done** — kg totals on day/period books; tenders include credit |
| Debit–credit report | Clear AR/AP / who-owes-whom | **Done** — Party aging + payable aging + net position + debit/credit pack |
| Party-wise outstanding | Balances per party | **Done** — Parties + aging reports |
| Bank posting / bank book | Till ↔ bank, bank in/out | **Done** — Reports bank |
| Monthly report | Period books | **Done** — Reports → This month |
| Profit and loss | Period P&L shape | **Done** — period books (rev, COGS, expenses, net) |
| Balance sheet | Assets / liabilities snapshot | **Done** — operating balance sheet (bank, AR, AP, net) |
| Expense voucher | Running costs | **Done** — Pay out / Reports (+ bill attachments) |
| Invoice format | Printable tax / account invoice | **Done** — `format_invoice_text` on account bookings; receipt still for cash |

### C. Ops / barcode (voice + desk expectation)

| Ask | Meaning | Status |
|-----|---------|--------|
| Barcode on dockets / labels | Print scannable AWB | **Done** — Code 128 on SVG AWB label + `*AWB*` on receipt |
| Scan at the desk | USB/Bluetooth wedge into Ops AWB field | **Done** — Find / Scan AWB (Enter after scan) → desk card |
| Scan to act | Advance / label / COD without retyping | **Done** — Out for delivery, Delivered, Print label, Collect COD use the scanned AWB |

Same product also covers booking desk, rates × kg, tracking / POD, COD, carriers vs own riders — see [`KUMKAJ_COURIER.md`](KUMKAJ_COURIER.md).

**Not in scope as primary path:** phone-camera barcode AI (wedge scanners are the counter standard).

------------------------------------------------------------------------

## Example party kinds (illustrative — not seeded)

Owners enter their own names. Typical shapes:

- **Carriers:** national / regional courier brands the agency hands bags to  
- **Consignors:** shops and traders who give parcels to the agency  

Do **not** ship sample party lists in the app.

------------------------------------------------------------------------

## Gaps closed in product

1. **Day / month books** show **total chargeable kg** for the period.  
2. **Debit / credit pack** on Reports (aging + AP + net + labelled Debit/Credit lines).  
3. **Invoice format** — dedicated account-invoice text (GSTIN, AWB, weight, tax lines, bill no).  
4. **Balance sheet** — operating snapshot: bank + debtors vs creditors + balancing net.  
5. **Barcode desk loop** — print Code 128 · scan/find AWB · advance / label / COD.

6. **Mixed settlement on expenses** — one expense, several tenders (part bank / part cash), paid-by chips + optional second way; no bill required to save (ground truth, vision §18).
7. **Cash-till perpetual ledger** — cash in hand from every recorded cash movement, on Day book + balance sheet; day-close compares counted vs in−out.
8. **Open-item bill allocation** — Parties shows each open bill; settle against a chosen bill # (rest washes FIFO), AR and AP.
9. **Bank reconcile (BRS)** — tick bank movements off against the statement; book balance vs cleared balance, uncleared listed.
10. **Spreadsheet CSV** — copy day book / outstanding / parties / items / party ledger as CSV; import parties and items/rates from a CSV file.

Still **later** (statutory, needs country filing integration — not promised from this chat alone): statutory e-invoice / GSTR filing, Schedule III formal BS.

------------------------------------------------------------------------

## Where to tap in the app

| Client words | Screen |
|--------------|--------|
| Courier / party accounts | Parties |
| Day book / per day | Day book · Reports → Today books |
| Bank posting | Reports → bank actions |
| Expense | Pay out or Reports → Log expense |
| Invoice | Booking (Account) → copy / print; Sales → receipt/invoice |
| Monthly / P&L / BS | Reports → This month (includes P&L + balance sheet) |
| Outstanding / debit–credit | Reports → Outstanding / Dr-Cr |
| Settle against one bill | Parties → pick party → Open bills → bill # → Pay |
| Bank reconcile | Reports → Bank reconciliation → tick Uncleared # |
| Cash in hand | Day book (BANK card) · balance sheet |
| Spreadsheet in / out | Reports → Spreadsheet (CSV) |
| Barcode print / scan | Ops → Print AWB label · AWB / scan field → Find |

------------------------------------------------------------------------

## Agent rule

When changing books or reports, keep this checklist covered and update the Status column here. Do not put private client contact details in the repo.
