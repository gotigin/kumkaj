# Use Cases

Implementation guide per **concrete Kumkaj product**. Current focus:
**[`KUMKAJ_COURIER.md`](KUMKAJ_COURIER.md)** (Kumkaj Courier).

------------------------------------------------------------------------

# Courier Entity → Kumkaj Courier

This product is the full **courier business operating system** for an
agency / franchise counter: books **and** desk ops an app can own.

Research inputs:

- Industry agency products: Bellboy, GoInvoicing, ESE, CourierDesk, Saap,
  BillMyCourier, aggregators (Shiprocket-class), etc.
- Client WhatsApp export `Global prg drj.zip` (text only; voice omitted)
- Ops/tax depth: [`grokbot/COURIER_BUSINESS_TUTORIAL.md`](grokbot/COURIER_BUSINESS_TUTORIAL.md)

Feature matrix, build order, and walk-in / barcode / tracking split:
**[`KUMKAJ_COURIER.md`](KUMKAJ_COURIER.md)**.

## Account vs cash path (required language)

| Path | Courier labels | Meaning |
|------|----------------|---------|
| Shareable / account | **Account booking** | Shown externally / on party ledger |
| Internal / spot | **Cash booking** | Pay now (cash/UPI/card) |

One sales/booking table + bill-type tag — not two money apps. Never call
this “black/white money.”

## Client requirements (WhatsApp — Global prg drj)

**Parties (examples they named — owner enters their own)**  
Carriers: Delhivery, DTDC, Mahaveer, ST, Professional.  
Consignors: Sumit International, Bangles King, Love India, Balaji
Enterprise, Alisha Creation.

**Reports / books**  
Per day · kg + credit/debit · debit/credit · party outstanding · bank
posting · monthly · P&L / balance sheet · invoice format.

## Industry — most common / important features

**Core (almost every agency product)**

1. Consignment / docket booking (party, destination, weight, service) + print  
2. Cash booking vs Account booking  
3. Party master (opening balance, contact, credit)  
4. Party-wise rate master (zone × service × weight slab)  
5. City / zone / pincode master  
6. Weight charging (actual; volumetric / chargeable)  
7. Service types (Docs, Parcel, Priority, Air, …)  
8. Party ledger (bookings, bills, receipts → balance)  
9. Outstanding / collection / aging  
10. Receipts / settlements  
11. Period party billing (unbilled dockets → one tax invoice)  
12. Day book / cash book  
13. Multi-carrier books  
14. Extra charges (fuel, labour, packing, ODA, misc)  
15. Reports (day, party, billed/unbilled, tax)  
16. Staff rights  

**Very common**

17. Carrier cost / payable (margin)  
18. AWB / CN + **label/barcode**  
19. Credit limit on account booking  
20. Opening balances + Excel import  
21. COD / FOD  
22. Bulk booking  
23. Consignment filters (date, party, city, billed/unbilled)  

**Ops add-ons**

24. Tracking · 25. POD · 26. Pickup/dispatch/riders · 27. Aggregator compare ·
28. Carrier invoice reconcile · 29. WhatsApp reminders · 30. Customer portal  

## Shared primitives (when extracting later)

| Need | Shared shape |
|------|----------------|
| Carrier vs consignor | Party + `kind_label` |
| Kg on lines | Item/line + unit |
| Account vs cash | Bill-type tag |
| Charge consignor | Money-in booking |
| Pay carrier / costs | Money-out / expense + AP |
| Outstanding | AR / AP |
| Bank | Money movements |
| Day / month / P&L | Day/period books |
| AWB / barcode | Document / line reference + print/scan |
| Invoice | Document template |

## Other products

Further concrete products (counter, food, personal, …) get their own
sections when that product becomes the active focus. Until then, do not
dilute courier delivery with parallel SKU work.
