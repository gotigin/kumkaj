# Moklanjo client requirements (discovery)

Sanitized checklist from a **courier / logistics-agency** discovery chat (Oct 2026).  
**No personal names, phones, chats IDs, or download links** — only product asks.

Companions: [`MOKLANJO.md`](MOKLANJO.md), [`MOKLANJO_APP_TUTORIAL.md`](MOKLANJO_APP_TUTORIAL.md) (practice + demo script), [`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md), [`grokbot/MOKLANJO_BUSINESS_TUTORIAL.md`](grokbot/MOKLANJO_BUSINESS_TUTORIAL.md).

**Scope:** this is Moklanjo's discovery evidence and status tracker —
it stays in Moklanjo's docs. The general BOS needs underneath §A–B
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
| Carrier company accounts | Carriers as parties (owner types names) | **Done** — Parties · Carrier / multi-role · name/phone/address · custom fields |
| Our party accounts | Consignors / shippers as parties | **Done** — Parties · Consignor / multi-role · name/phone/address · optional logo · custom fields |
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

Same product also covers booking desk, rates × kg, tracking / POD, COD, carriers vs own riders — see [`MOKLANJO.md`](MOKLANJO.md).

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

Research Local/LAN catch-up (rider mode, DRS, NDR/RTO, pincode, rate
compare, …): **done 2026-10-27** — see [`COURIER_RESEARCH.md`](COURIER_RESEARCH.md)
§7. Still left there is Cloud/Net/owner-only (live carrier APIs, map ETA,
public track outside LAN).

------------------------------------------------------------------------

## Where to tap in the app

| Client words | Screen |
|--------------|--------|
| Moklanjo / party accounts | Parties |
| Day book / per day | Day book · Reports → Day book (today) |
| Bank posting | Reports → Bank book |
| Expense | Pay out → Expense voucher · or Reports → Expense voucher |
| Invoice | Booking (Account booking) → copy / print; Bookings → receipt/invoice |
| Monthly / P&L / BS | Reports → Profit & loss (month) · Balance sheet |
| Outstanding / debit–credit | Reports → Outstanding / Dr-Cr |
| Settle against one bill | Parties → Pick → Against bill # → Settle cash / Pay UPI / Settle card |
| Bank reconcile | Reports → Bank book → Bank reconciliation → tick Uncleared # |
| Cash in hand | Day book (BANK card) · Balance sheet |
| Spreadsheet in / out | Reports → Spreadsheet (CSV) |
| Barcode print / scan | Ops → AWB / scan · Find / Scan AWB · Print AWB label |

------------------------------------------------------------------------

## Agent rule

When changing books or reports, keep this checklist covered and update the Status column here. Do not put private client contact details in the repo.
