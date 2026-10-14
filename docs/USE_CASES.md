# Use Cases
This document is intended to ease the process of implementation
It's intention is to target entities from different categories and 
for each entity we will go through the entire process of operating
throuugh our app.

# Courier Entity
This should cover the entire courier business management system.
We will first do a research on how other top 10 apps of the world
implement such features for it's clients and we also include our 
client requirement as discussed in the WhatsApp chat.

## Cursor Research

Yes — understood. Many entities (especially in India) need **two bill paths on one book**: what is **shareable / declared** (for government or authorized agencies) vs what is the **internal / owner** operational picture (real rates, privacy, theory-vs-practice price gaps). Kumkaj must **never** call this “black/white money.” Preferred wording (already product language):

| Path | Courier labels | General labels | Meaning |
|------|----------------|----------------|---------|
| Shareable / account | **Account booking** | **Tax invoice** | What may be shown externally / on account |
| Internal / spot | **Cash booking** | **Cash memo** | Owner’s internal / cash-path record |

One sales table + a **bill-type tag** (`money_lane`) — not two product forks, not two separate apps. Owner chooses per bill; reports can split or combine. Internal original state stays attributable; shareable view is what they elect to present.

Sources for this section: industry courier-agency products (Bellboy, GoInvoicing, ESE, CourierDesk, Saap, BillMyCourier, aggregators, etc.) + WhatsApp export `Global prg drj.zip` (text only; voice messages omitted).

### Client requirements (WhatsApp — Global prg drj)

**Parties**
- Carriers / partners named: Delhivery, DTDC, Mahaveer, ST, Professional  
- Consignors / customers named: Sumit International, Bangles King, Love India, Balaji Enterprise, Alisha Creation  

**Reports / books asked**
- Per day report  
- Kgs + credit / debit account  
- Debit / credit report  
- Party-wise outstanding  
- Bank posting  
- Monthly report  
- Profit & loss / balance sheet  
- Invoice format  

### Industry — most common / important features

**Core (almost every agency product)**
1. Consignment / docket booking (party, destination, weight, service) + print note/receipt  
2. **Cash booking vs Account booking** (pay now vs party account)  
3. Party master (opening balance, contact, credit)  
4. Party-wise rate master (zone × product/service × weight slab; extra per kg/500g)  
5. City / zone / pincode master  
6. Weight charging (actual; often volumetric / chargeable weight)  
7. Service types (Docs, Parcel, Priority, Air, …)  
8. Party ledger (bookings, bills, receipts → running balance)  
9. Outstanding / collection / aging  
10. Receipts / settlements (cash/UPI/bank; optional discount)  
11. Period / monthly party billing (unbilled account dockets → one tax invoice + surcharges)  
12. Day book / cash book  
13. Multi-carrier / multi-company books  
14. Extra charges (fuel, labour, packing, ODA, misc)  
15. Reports (day value, party sales, billed vs unbilled, outstanding, tax)  
16. Staff rights  

**Very common**
17. Carrier cost / payable (margin = charge consignor − pay carrier)  
18. AWB / CN number + label/barcode  
19. Credit limit on account booking  
20. Opening balances + Excel import  
21. COD / FOD  
22. Bulk booking  
23. Consignment book filters (date, party, city, billed/unbilled)  

**Ops add-ons (later capability packs)**
24. Tracking statuses · 25. POD · 26. Pickup/dispatch/riders · 27. Rate compare/aggregator · 28. Carrier invoice reconcile · 29. WhatsApp reminders · 30. Customer portal/wallet  

### Generic mapping (no courier product fork)

| Need | Primitive |
|------|-----------|
| Carrier vs consignor | Party + `kind_label` |
| Kg on lines | Item/line + unit + conversions (incl. owner-defined units) |
| Account vs cash path | Bill-type tag: Account booking / Cash booking |
| Charge consignor | Sell (money in) |
| Pay carrier / costs | Buy / expense (+ AP later) |
| Outstanding | AR (and AP for carriers) |
| Bank posting | Cashbook / money movements |
| Day / month / P&L | Day close, period books (COGS/BS as backlog) |
| Invoice format | Document / receipt templates |

### BOS spine for this entity (what “good” means)

Same place for carriers and consignors, ₹/kg rates, money in and out, who still owes whom, bank, day/month picture — **Account booking** or **Cash booking** on each bill. Not a courier-only app and not only a sell screen. Shell peers: Books · Sell · Buy · More.
