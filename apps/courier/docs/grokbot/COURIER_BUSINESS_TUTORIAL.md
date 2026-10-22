# Running a Courier Agency / Franchise in India: The Complete Simple Guide

### Operations, money, books (hisaab-kitaab), invoices and taxes, explained step by step

> **Scope / maintenance:** this is a **Courier-only research reference**
> (courier-agency operations, worked books, India tax depth as it
> applies to a courier counter). It stays in Courier's docs. Generic
> bookkeeping/tax *doctrine* for all Kumkaj apps is canonical — in
> condensed form — in root
> [`BUSINESS_BOOKS.md`](../../../../docs/BUSINESS_BOOKS.md) (incl.
> India compliance, deferred) and vision §23; if a general rule here
> changes, update those root docs, not just this tutorial. Part 10's
> modelling suggestions are courier instantiations of the root
> concepts, not Kumkaj-wide requirements.

*Written for Mearaj, who is building **Kumkaj** (a general business-books app). This one document has two jobs:*

1. *Anyone who has passed 10th class, even with no commerce or accounting knowledge, should be able to **run a courier counter and keep all its books** using only this guide. Parts 1 to 9 are written for that person.*
2. *It is also the full research reference: all the facts, tax rules, sources and "ask your CA" points, plus developer notes on how Kumkaj should model a courier business (Part 10).*

---

## Before you start: how to read this guide

**Who this is for.** Someone who wants to run a courier shop (a franchise counter, an agency, or a small courier business of their own) and wants to keep the money records correctly. You do not need to know accounting. Every new word is explained the first time it comes up, usually with an everyday example and a Hindi/Hinglish word in brackets.

**How the guide is arranged.** It follows the story of one example shop, from opening to the end of its first full month:

- **Part 1**: what a courier business is and how it makes money
- **Part 2**: setting up the shop
- **Part 3**: one full working day, hour by hour, with filled example pages of every book
- **Part 4**: weekly and monthly work
- **Part 5**: understanding debit and credit, profit and loss, and the balance sheet, using one example month
- **Part 6**: invoices (bills) in detail
- **Part 7**: taxes in simple words, and what to give your CA
- **Part 8**: honest records, and why you must record every booking
- **Part 9**: common mistakes, checklists, how long to keep records, numbers to watch
- **Part 10**: developer notes for Kumkaj (reference)
- **Part 11**: glossary (word meanings)
- **Part 12**: sources and the list of things to confirm with a CA

> ⚠️ **EXAMPLE NUMBERS ONLY.** Every rupee (₹) figure in this guide is a **made-up example** to help you learn the maths. Real rates, commissions, surcharges and salaries change by carrier, contract, city and month. Always use your own carrier's rate card and your own agreement.
>
> ⚠️ **Tax and law warning.** Tax rules change often. GST rates were rationalised (re-arranged) in September 2025, and a new **Income-tax Act, 2025** replaced the old 1961 Act from **1 April 2026**. Wherever you see **🔎 Ask your CA to confirm**, check that point with a Chartered Accountant (CA) before relying on it. This guide is for learning. It is not tax advice.

### Meet our example shop

Throughout this guide we follow one **example** business:

| Detail | Example |
|---|---|
| Shop name | **Raju Courier Point** (example) |
| Owner | Raju, running it alone as a **proprietor** (one-person business: maalik khud) |
| Place | Pune, Maharashtra |
| What it does | Books parcels with a main carrier, **"FastTrack Couriers"** (a made-up carrier name), and also earns a commission as a franchise counter of a second brand |
| Staff | 1 counter assistant, 1 delivery/pickup rider, 1 helper |
| GST | Registered (it has a GSTIN, explained in Part 2) |
| Financial year | 1 April 2026 to 31 March 2027 (written "2026-27") |

The day we follow in Part 3 is **Wednesday, 14 October 2026**. The month we study in Part 5 is **October 2026**.

---

## Table of contents

- [Part 1: What a courier business is and how money is made](#part-1-what-a-courier-business-is-and-how-money-is-made)
- [Part 2: Setting up the shop](#part-2-setting-up-the-shop)
- [Part 3: A full working day, hour by hour](#part-3-a-full-working-day-hour-by-hour)
- [Part 4: Weekly and monthly work](#part-4-weekly-and-monthly-work)
- [Part 5: Understanding the books, profit and loss, and balance sheet](#part-5-understanding-the-books-profit-and-loss-and-balance-sheet)
- [Part 6: Invoices in depth](#part-6-invoices-in-depth)
- [Part 7: Taxes made simple](#part-7-taxes-made-simple)
- [Part 8: Honest records: why every booking must be recorded](#part-8-honest-records-why-every-booking-must-be-recorded)
- [Part 9: Mistakes, checklists, record keeping and numbers to watch](#part-9-mistakes-checklists-record-keeping-and-numbers-to-watch)
- [Part 10: Developer notes: how this maps to Kumkaj](#part-10-developer-notes-how-this-maps-to-kumkaj)
- [Part 11: Glossary](#part-11-glossary)
- [Part 12: Sources and items to confirm with a CA](#part-12-sources-and-items-to-confirm-with-a-ca)

---

# Part 1: What a courier business is and how money is made

## 1.1 The simple idea

A courier shop is like a **travel agent for parcels**. A travel agent does not own the train or the plane. He books your ticket with the railway or the airline, charges you a little more than he pays, and keeps the difference.

A courier counter works the same way:

1. A customer brings a parcel (or you collect it from them).
2. You weigh it, work out the price, take the money or note it on their account, and give a bill.
3. You hand the parcel to a big courier company (the **carrier**), for example Delhivery, DTDC, Blue Dart, Ekart, Xpressbees, Shree Maruti, Professional, or India Post (Speed Post).
4. The carrier carries it across the country and delivers it.
5. The carrier sends **you** a bill for its charges (or takes them from your prepaid balance).
6. **What the customer paid you, minus what the carrier charges you, is your gross profit.** Rent, salaries and other costs come out of that.

Two kinds of people matter in your books:

- **Consignor** (the sender; *bhejne wala*). This is **your customer**. Money comes **in** from them.
- **Carrier** (the courier company). Money goes **out** to them.

The **consignee** (receiver; *paane wala*) is the person at the other end. They are usually **not** your customer (except in "To-Pay" and COD cases, explained in 2.7).

## 1.2 The life of a parcel

```
Consignor (sender) ──booking──► Your counter / pickup
      │                               │  AWB/docket generated, weight measured, charged
      ▼                               ▼
   First mile  ──►  Origin hub/branch  ──manifest──►  Linehaul (air/surface)
                                                          │
                                                          ▼
                                           Destination hub ──► Delivery centre
                                                                     │
                                                    Last mile (out for delivery)
                                                                     │
                       ┌───────────────┬──────────────────────────────┤
                       ▼               ▼                              ▼
                  Delivered (POD)   NDR (attempt failed)        RTO (returned to origin)
```

In simple words: you book it, the carrier picks it up from you, it goes to a big sorting centre (**hub**), travels by truck or plane (**linehaul**), reaches a hub near the receiver, and a delivery boy takes it to the door (**last mile**). Then one of three things happens: it is **delivered** (with proof), the delivery **fails** (NDR), or it comes **back** to the sender (RTO).

## 1.3 Words you will hear every day

| Word | Simple meaning | Everyday example |
|---|---|---|
| **Consignment / shipment** | The parcel (or set of parcels) sent under one booking. | One box of sarees sent to Surat. |
| **Consignor** | The sender. Your customer. In Kumkaj, your *money-in* party. | The shopkeeper who brings the box. |
| **Consignee** | The receiver at the destination. Usually not your customer (unless "To-Pay"). | His relative in Surat. |
| **AWB / Docket / Waybill / CN number** | The parcel's unique tracking number, like a **PNR number on a train ticket**. *AWB* = Air Waybill (it started in air cargo, now used for everything); *docket* and *CN (consignment note)* are common counter words (*parchi number*). **This is your most important reference number.** Every booking, carrier bill line, COD payment and complaint is linked by it. | 5001002 |
| **Booking** | Accepting the parcel: writing sender and receiver details, contents, declared value, weight and size, service and price, and giving a bill. | |
| **Pickup** | The carrier or your rider collects parcels from the customer's place, instead of the customer walking in. | Your rider goes to a shop at 2 pm. |
| **Manifest** | The list of all parcels you hand to the carrier in one batch (AWB, weight, pieces, COD amount), signed or scanned at handover. Your **proof of what you gave** them, like a *challan* or delivery list. | Evening list of 7 parcels. |
| **Hub** | A big sorting centre where parcels from many branches are collected and sent on. | |
| **Linehaul** | The long-distance trip between hubs, by plane or truck. | Pune hub to Delhi hub. |
| **First mile / last mile** | Pickup up to the origin hub / destination hub up to the doorstep. The last mile costs the most and fails the most. | |
| **NDR (Non-Delivery Report)** | The delivery attempt failed (receiver not home, wrong address, refused). Someone must act: try again, correct the address, or return it. | "Customer not available." |
| **RTO (Return to Origin)** | The parcel is sent back to the sender after delivery fails. Carriers usually **charge RTO freight** (often equal to the forward charge). That is a real cost: either bill it to the consignor or bear it yourself. | |
| **POD (Proof of Delivery)** | Signature, OTP or photo proving delivery. Needed to settle complaints and to claim COD money. | |
| **COD (Cash on Delivery)** | The receiver pays for the goods at the door. The carrier collects that money and later sends it to the sender (through you, if you are the agent or aggregator). | An online buyer pays ₹1,499 to the delivery boy. |
| **ODA (Out of Delivery Area)** | A remote pincode outside the carrier's normal area. There is an extra **ODA charge**, and delivery takes longer. | A village far from any town. |
| **Serviceability / pincode check** | Whether a carrier delivers to a pincode, and which services (COD, express) work there. | Check before you accept the parcel. |
| **TAT (Turn-around time)** | The promised number of delivery days. | "3–4 days." |
| **MPS (Multi-piece shipment)** | One booking with many boxes: one master AWB plus child AWBs. | 5 cartons, one booking. |

## 1.4 Five ways to run a courier shop (business models)

A courier shop on the street can be one of several quite different businesses. The type decides **who your customer is, who owns the money you collect, and what your real income is**. That changes all your bookkeeping, so first find out which type you are. Your agreement with the carrier tells you.

### Model 1: Franchise of a carrier (for example a DTDC, Professional or Shree Maruti franchise)

- You run a counter under the carrier's brand name. The carrier gives you a franchise code, stationery and dockets (or a booking app), and a price list.
- There are two common ways the money flows. Your franchise agreement says which one applies:
  1. **Buy-and-sell (rate-difference) model.** The carrier bills **you** at a "franchise rate". You charge customers your own, higher rate. Your income = what customers pay you. Your cost = the carrier's bill. **Profit = the difference.** (Like a shopkeeper who buys at wholesale and sells at retail.)
  2. **Commission model.** You collect the carrier's published price **on behalf of** the carrier, pass the money on, and the carrier pays you a commission (for example, 20–30% of booking value). Your income = **the commission only**. The booked amount was never really yours. (Like a lottery or recharge agent who keeps only his cut.)
- There is often a **franchise deposit** (security deposit; *jamaanat*) that you get back when you leave. It is an **asset** (something you own), **not an expense**. There may also be a one-time franchise fee.

**Commission example (example numbers):** In October you booked ₹3,00,000 of freight (before GST) at the carrier's price. The agreement says 25% commission on bookings, minus deductions by the carrier.
- Commission = 3,00,000 × 25% = **₹75,000**. This is your **income**.
- You collected ₹3,00,000 + GST from customers, but ₹2,25,000 (plus the related GST, as your agreement says) belongs to the carrier. That is a **payable** (money you owe), **not income**. Check: 3,00,000 − 75,000 = 2,25,000.
- 🔎 **Ask your CA to confirm:** who issues the GST bill to the end customer (you or the franchisor); whether you must raise an 18% GST invoice on your commission to the franchisor; and the TDS the franchisor cuts from your commission (old section 194H, now new section 393(1) Table Sl. No. 1(ii), reported as 2% above ₹20,000 per year in secondary sources [19]).

### Model 2: Aggregator / multi-carrier agency

- You hold accounts with several carriers (directly, or through an aggregator platform such as Shiprocket, Shipmozo or NimbusPost) and choose the cheapest or best carrier for each parcel.
- Customers see **your** price. You pay the carrier, often from a prepaid **wallet** (balance you top up in advance, like a prepaid mobile recharge). Income = what you bill customers. Cost = carrier charges. Kumkaj's Carrier/Consignor design fits this model best.

### Model 3: Booking agent of one carrier

- Like a franchise, but usually smaller and less formal. You book on the carrier's system and get a commission or a fixed fee per docket. Treat it like the commission model above.
- 🔎 **Ask your CA to confirm:** a business earning "commission or brokerage", or doing "agency business", is **not allowed** to use the simple presumptive tax scheme under old section 44AD (see 7.10). So how your franchise income is described matters.

### Model 4: Own network (your own pickup and delivery staff, local or regional)

- You move the parcels yourself (bikes, vans, local hubs), and maybe tie up with other carriers for long distance (called "interline").
- Costs now also include riders' salaries, fuel, vehicle EMI or depreciation (wear and tear, see Part 5), and hub rent. Income = the full freight billed to customers.
- Watch the **GTA vs courier** difference for GST if you start doing truck-load transport with consignment notes (see 7.6).

### Model 5: Hybrid (a mix; most real shops)

Most small courier owners run a franchise counter for one brand, **and** book other carriers through accounts or aggregators, **and** do some local same-day delivery. So your books must handle **several carriers, each with its own price list and its own way of settling money**.

Our example shop, Raju Courier Point, is a hybrid. It mostly does **buy-and-sell** through FastTrack Couriers, and it also earns a **commission** from a second brand's franchise counter.

## 1.5 How money is made: two price lists for every parcel

For every parcel you always have **two prices**:

- **Carrier rate** (buying price; *khareed bhaav*): what the carrier charges you. It may be a franchise rate, a contract rate, or the amount taken from your aggregator wallet.
- **Customer rate** (selling price; *bechne ka bhaav*): what you charge the sender. Regular account customers often get a lower negotiated rate. Walk-in customers pay the counter rate, which works like an MRP.

**The difference between the two is your margin** (*munafa* on that parcel).

**Example (example numbers, worked out fully in Part 3):** Sunita's 5 kg box to Bengaluru.
- Raju charges Sunita **₹704.00** (before GST).
- FastTrack charges Raju **₹428.60** (before GST).
- **Margin = 704.00 − 428.60 = ₹275.40**, which is about 39% of the sale (275.40 ÷ 704 = 0.391).

GST is left out of the margin. If you are registered and claim input tax credit, GST just passes through you: you collect it from the customer and pay it to the government, after deducting the GST you paid the carrier (Part 7).

## 1.6 Where the money leaks (*paisa kahan se nikalta hai*)

Margins shrink quickly because of:
- **Wrong weight entered at booking.** The carrier re-weighs the parcel, finds it heavier, and charges you extra (4.5).
- **RTOs you cannot recover** from the customer.
- **ODA charges you forgot to bill.**
- **Credit customers** who pay late, or never.
- **Cash not recorded**, or cash that goes missing (Part 8).

Each of these has a fix in this guide.

---

# Part 2: Setting up the shop

## 2.1 Decide the form of business and get the basic papers

Most small courier shops start as a **sole proprietorship** (one owner; the business and the owner are legally the same person). A partnership firm, LLP or company is also possible. The choice changes taxes and paperwork, so discuss it with a CA.

Basic papers you will usually need (the exact list depends on your state and city, so **🔎 ask your CA to confirm**):

| Paper | What it is, simply | Why you need it |
|---|---|---|
| **PAN** | Your income-tax identity number. | Needed for the bank account, GST, TDS and income-tax return. |
| **Bank current account** in the business name | A bank account only for the shop (2.9). | Keeps shop money apart from home money. |
| **GST registration (GSTIN)** | Your GST identity number (2.2). | Needed once turnover crosses the limit, and usually asked for by business customers. |
| **Shop / local trade licence** | Permission from local or state authorities to run a shop. | Rules differ by state. |
| **Franchise / carrier agreement** | The contract with each carrier (2.3). | Decides your rates, model and settlement. |
| **Rent agreement** | For the shop premises. | Rent proof, address proof for GST. |

## 2.2 GST basics: what it is and when you must register

**What GST is.** GST (Goods and Services Tax) is a tax the government puts on almost everything sold, including courier services. You **add** it to your bill, **collect** it from the customer, and **pay** it to the government. Think of yourself as a **postman for tax**: the money passes through your hands, but it is not yours.

**The rate for courier is 18%** of the bill amount before tax. It is split in one of two ways:
- **CGST 9% + SGST 9%** (Central GST + State GST) when the supply is **within one state** (intra-state). Example: Pune shop, Pune customer.
- **IGST 18%** (Integrated GST) when the supply is **between two states** (inter-state). Example: Pune shop, business customer registered in Karnataka.

Which one applies depends on the **place of supply** rules (7.2).

**When must you register?** A service business must register once its **aggregate turnover** (total sales in a financial year) crosses the threshold, generally **₹20 lakh** (**₹10 lakh** in special-category states). You may also register voluntarily before that, and many business customers will insist you are registered so they can claim the GST back (input tax credit, 7.3). 🔎 **Ask your CA to confirm** the current limit, and whether your franchise agreement requires registration.

**After registering** you get a 15-character **GSTIN** (GST number). It must be printed on every tax invoice.

**Very important:** if you are **not** registered, you must **not** charge or collect GST from anyone. Give a simple bill without GST (6.4).

**For franchisees:** find out whether **you** or the **franchisor** is legally the supplier to the end customer. That decides who makes the invoice and pays GST on the booking, and whether your own GST supply is only the commission (1.4). 🔎 **Ask your CA to confirm.**

## 2.3 Tie up with carriers

For each carrier, get these points **in writing** (in the agreement or rate card) before you start:

| Point to get in writing | Why it matters |
|---|---|
| Model: buy-and-sell, commission, or wallet | Decides what your income is (1.4). |
| Your buying rates by service, zone and weight slab | Needed for your rate card and margin. |
| **Volumetric divisor** (5000? 4500?) | Changes the chargeable weight (2.4). |
| Fuel surcharge % and how often it changes | It is added to every bill. |
| Docket, FOV, COD, ODA, RTO and other charges | All extras you will be billed. |
| Billing cycle (weekly or monthly bill, or wallet) and payment due date | For planning payments (4.4). |
| **COD remittance cycle** (for example weekly, or D+2/D+7 after delivery) | When COD money reaches you (4.1). |
| **Weight dispute window** (often only a few days) | Late complaints are rejected (4.5). |
| Security deposit and franchise fee | Deposit = asset; fee = check with CA. |
| Who issues the GST invoice to the customer | GST responsibility (2.2). |
| Liability for lost or damaged parcels | What you can claim, and what you promise customers. |

## 2.4 Weight: actual, volumetric, chargeable, slabs and zones

Courier prices depend on **weight**, but not only on what the scale shows.

**Actual weight** (dead weight; *asli wazan*): what the weighing scale shows, packaging included.

**Volumetric weight** (size-based weight; *aakaar wala wazan*): a big, light box (for example a box of pillows) fills a lot of space in a plane or truck. So carriers work out a weight **from the size**:

> **Volumetric weight (kg) = Length × Breadth × Height (all in cm) ÷ divisor**

The **divisor** is set by the carrier contract. **5000** is the common domestic standard (Delhivery uses 5000 for both Surface and Express [18]). Some services or contracts use 4500 or other figures, so always take it from your carrier's rate card.

**Chargeable weight** = **whichever is higher, actual or volumetric**, then **rounded UP to the next slab**.

**Slab** = the weight step used for pricing, usually 0.5 kg. So 0.3 kg becomes 0.5 kg; 1.1 kg becomes 1.5 kg; 4.8 kg becomes 5.0 kg.

**Worked example (example numbers):** A box measures 40 cm × 30 cm × 20 cm and weighs 3.2 kg on the scale.
1. Multiply the sizes: 40 × 30 = 1,200; then 1,200 × 20 = 24,000.
2. Divide by 5000: 24,000 ÷ 5000 = **4.8 kg** volumetric.
3. Compare: actual 3.2 kg, volumetric 4.8 kg. The higher one is **4.8 kg**.
4. Round up to the next 0.5 kg slab: **5.0 kg chargeable**.

Delhivery's own example: a 60 × 25 × 40 cm box with 6 kg dead weight has a volumetric weight of 12 kg (60,000 ÷ 5000 = 12), so 12 kg is charged [18].

**How slab pricing works.** Rates are rarely "₹X per kg flat". Usually:
- **First slab** (for example the first 0.5 kg) has a fixed price, then
- **each additional slab** (each extra 0.5 kg) has a lower price,
- and sometimes there is a separate "per kg above 5 kg / 10 kg" rate for heavy surface parcels, with a **minimum chargeable weight** (for example, a surface minimum of 3 kg or 5 kg for business parcels).

> **Freight formula for 0.5 kg slabs:** number of slabs = chargeable kg ÷ 0.5. Freight = first-slab price + (number of slabs − 1) × additional-slab price.
> Example: 5.0 kg = 10 slabs. Freight = first price + 9 × additional price.

**Zones.** Carriers group destinations into zones, counted from where the parcel starts. A typical structure (names differ by carrier):

| Zone (typical) | Meaning | From Pune, for example |
|---|---|---|
| A / Local / Within City | Same city | Pune to Pune |
| B / Regional / Within State | Same state or nearby region | Pune to Nashik, Nagpur, Mumbai |
| C / Metro to Metro | Between metro cities | Pune to Delhi, Bengaluru, Chennai, Hyderabad |
| D / Rest of India | Everything else | Pune to Lucknow, Patna |
| E / Special | North-East, J&K, Andaman, Lakshadweep, some hill areas | Pune to Guwahati |

So: **price = (service, zone, chargeable weight) from the rate card + surcharges.** Your rate card is basically a lookup table on those three things.

## 2.5 The charges that can appear on a courier bill

| Charge | What it is, simply | Typical basis (EXAMPLES only) |
|---|---|---|
| **Freight / basic charge** | The price for the chargeable weight, zone and service | Rate card |
| **Fuel surcharge (FSC)** | A % added on freight, changed from time to time with diesel/ATF prices | e.g., 15–35% of freight |
| **Docket / AWB charge** | Fixed fee per parcel for paperwork | e.g., ₹10–₹100 per AWB |
| **FOV (Freight on Value) / Insurance / Risk surcharge** | Carrier's liability charge based on the **declared value** of the goods. "Owner's risk" (low %) vs "carrier's risk" (higher %) | e.g., 0.1%–0.5% of value, or a minimum ₹ amount |
| **COD charge** | Fee for collecting cash at delivery | e.g., ₹30–₹60 or 1.5–2.5% of COD value, whichever is higher |
| **ODA charge** | Delivery to a remote area | e.g., a flat ₹ amount, or per kg with a minimum |
| **RTO charge** | Return freight on undelivered parcels | Often equal to forward freight |
| **Handling / packing** | Extra for odd shapes, heavy items, or your packing material | Per piece |
| **Appointment / to-pay / special delivery charges** | Extras for special services | As per card |
| **GST** | 18% on the taxable total (Part 7) | 9% CGST + 9% SGST, or 18% IGST |

> **"Freight on value"** means the *value-based* liability or insurance charge (FOV). Do not confuse it with the value of the goods. The goods' value is **not** your income. You only hold it for someone else in COD (4.1).

## 2.6 Make your rate card (*bhaav ki list*)

You need **three** price tables: what the carrier charges you, what walk-in customers pay, and what account customers pay.

**Raju Courier Point rate card: Surface service, 0.5 kg slabs, divisor 5000 (EXAMPLE numbers, ₹ before GST)**

| Zone | Carrier (FastTrack) first 0.5 kg | Carrier each extra 0.5 kg | Counter (walk-in) first 0.5 kg | Counter each extra 0.5 kg | Account customer first 0.5 kg | Account each extra 0.5 kg |
|---|---|---|---|---|---|---|
| A Local | 28 | 20 | 50 | 35 | 40 | 28 |
| B Regional | 34 | 26 | 60 | 45 | 48 | 36 |
| C Metro | 40 | 32 | 70 | 50 | 55 | 40 |
| D Rest of India | 48 | 38 | 85 | 60 | 65 | 48 |

**Extras (EXAMPLE numbers):**

| Extra | Carrier charges Raju | Raju charges counter customer | Raju charges account customer |
|---|---|---|---|
| Fuel surcharge | 20% of freight | 20% of freight | 20% of freight |
| Docket charge | ₹15 per AWB | ₹40 per AWB | ₹20 per AWB |
| FOV (only when value is declared for carrier's risk) | 0.1% of declared value | 0.2% of declared value | 0.2% of declared value |
| COD charge | ₹35 or 1.5% of COD value, whichever is higher | (not offered to walk-ins) | ₹50 or 2% of COD value, whichever is higher |

Print the counter rate card and keep it on the counter, so customers and staff see the same prices. Update it whenever the carrier changes the fuel surcharge or rates, and **write the date on it**.

## 2.7 Who pays and when: Paid, To-Pay, TBB, COD

| Billing type | Who pays freight | When | How it goes in the books |
|---|---|---|---|
| **Paid** | Consignor, at booking | Immediately | Cash booking (or noted on an account customer's ledger). |
| **To-Pay** | **Consignee**, at delivery | On delivery | The carrier collects freight from the receiver. You get no cash at booking; it shows up in your carrier settlement. Record the booking with a "to-pay" status; income may be counted only as your agreement says. 🔎 **Ask your CA to confirm** how GST invoicing works for to-pay parcels with your franchisor. |
| **TBB (To Be Billed)** | Consignor, later | Every week or month | **Account booking**: invoice raised on credit (*udhaar*); the customer becomes a debtor in your ledger; follow up the amount due. |
| **COD** | Freight is paid by the consignor (Paid or TBB); the *goods value* is collected from the consignee | On delivery | The goods value is money you **owe** the consignor until you pass it on (4.1). |

In Kumkaj terms: **Paid = Cash booking**, **TBB = Account booking**. To-Pay and COD are *features* of a parcel, not separate parties.

### Account booking vs cash booking

| | **Account booking** (*udhaar / khata customer*) | **Cash booking** (*nakad / turant*) |
|---|---|---|
| Customer | Regular business customer with credit terms (often GST-registered) | Walk-in person or occasional sender |
| Payment | Later (weekly or monthly), by bank or UPI | Immediately: cash, UPI, card |
| Document | Tax invoice with the customer's GSTIN (B2B: business to business), shareable as PDF/link; monthly statement | Tax invoice/receipt (B2C: business to consumer); no GSTIN needed |
| Effect in ledger | Customer owes you money until paid; appears in the aging list (4.3) | Nothing owed (settled at once) |
| Rates | Negotiated contract rates | Counter rates |
| Risk | Bad debts (money never paid), slow payment | Cash handling, theft, unrecorded sales |
| TDS | Business customers may cut TDS (1% or 2%) from your bill (7.9) | Not applicable |

**Both are sales. Both are income. Both must be in the books and in the GST returns.** The only difference is *when* and *how* you get paid (see Part 8 for why this matters).

**Tip:** a walk-in customer who pays by UPI is still a "cash booking" (paid on the spot), but the money lands in the **bank book**, not the cash book. Always note the **payment mode** (cash / UPI / card / bank) separately from the **booking type** (cash booking / account booking).

## 2.8 The books and registers to open (*bahi-khata*)

"Books of account" just means organised written records of money. You can keep them in a notebook, in Excel, or in an app like Kumkaj. The rules are the same.

| Book / register | What goes in it | Everyday comparison | Courier example |
|---|---|---|---|
| **Day book** (journal; *rojmel*) | Every transaction (money event), in date and time order | A diary of the shop | Every booking, carrier bill, expense, receipt |
| **Cash book** (*rokad bahi*) | Only physical cash in and out; its balance must match the cash in the drawer | Your wallet's diary | Walk-in cash bookings; tea, auto fare paid in cash; cash deposited in bank |
| **Bank book** | Only bank money in and out, including UPI and card settlements; match it with the bank statement | Your passbook, kept by you | UPI receipts, NEFT from account customers, wallet top-ups, COD remittances |
| **Party ledgers** (*khata*) | One page per **party id**, with **AR** and **AP** kept separate; a **net** line shows who owes whom overall | A *udhaar* notebook — one firm can be both customer and supplier on the **same** account | Same peer may be consignor **and** carrier on one UUID (names are aliases, need not be unique). Kumkaj: UUID identity, multi-role tags, two ledgers, net report ([`IDENTITY_AND_NAMES.md`](../../../../docs/IDENTITY_AND_NAMES.md), [`BUSINESS_BOOKS.md`](../../../../docs/BUSINESS_BOOKS.md)) |
| **Sales register** | Every invoice (bill) you issue | Carbon copies of all bills, in a list | Needed for the GST return GSTR-1 |
| **Purchase / expense register** | Every bill you receive | A file of all bills you got | Carrier invoices (for input tax credit), rent, electricity |
| **Account booking (TBB) register** | Each credit booking, to be billed at month end | A list of *udhaar* parcels | Patil Electricals' parcels |
| **COD register** | Each COD AWB: amount, delivered?, received from carrier?, paid to sender? | A list of other people's money you are holding | Meena Online Store's COD parcels |
| **Petty cash book** | Small cash expenses from the petty cash box (2.10) | A tea-and-auto notebook | Tea ₹60, auto ₹120 |
| **Manifest file** | Signed copy of each day's handover list | Proof of what you gave the carrier | Daily manifests |

**What a ledger is.** A **ledger** (*khata*) is a page (or account) for **one** person or **one** type of thing, showing every entry for it with a running balance. "Patil Electricals' ledger" shows everything Patil was billed and paid. "Rent ledger" shows all rent paid. The day book is in date order; ledgers are sorted by party or by type.

**What "debit" and "credit" mean** is explained in Part 5. For the daily books in Part 3 you only need "money in" and "money out".

## 2.9 Bank account and payment modes

- Open a **current account** in the business name. **Never mix** shop money and home money. When you take money for home, write it down as **drawings** (owner's withdrawal; *maalik ne nikala*), explained in Part 5.
- Get a **UPI QR code** linked to the business account, and a card machine if customers need it.
- Ask account customers to pay by **NEFT/RTGS/UPI** to the business account, not in cash.
- Deposit extra counter cash in the bank regularly (do not keep large cash in the shop).
- Remember the cash limits (7.11): you must not receive **₹2,00,000 or more** in cash from one person in a day, and business expenses paid in cash **above ₹10,000** to one person in a day are not allowed as expenses for income tax.

## 2.10 Petty cash (*chhota kharcha*): the imprest system

**Petty cash** is a small, fixed amount of cash kept in a separate box for small expenses like tea, auto fare or tape. The "imprest" system works like this:

1. Fix a float (fixed amount; for example ₹2,000 at Raju's shop; any amount you choose, such as ₹5,000).
2. Every time money is taken out, write a **petty cash voucher** (a small slip: date, what for, amount, who took it, signature) and keep the shop bill with it if there is one.
3. When the box runs low, **top it up by exactly the total of the vouchers**.
4. At any moment: **cash in the box + total of vouchers = the fixed float** (₹2,000).

If this check fails, something is missing. Find out the same day.

Keep **a bill or voucher for every expense**. If you are GST-registered, you need a proper GST invoice from the supplier to claim the GST back as input tax credit.

## 2.11 Decide your bill numbering before the first bill

Bills (tax invoices) must be numbered **one after another, without gaps, and unique for each financial year** (details in 6.3). Raju uses (example):

| Series | Format | First number of 2026-27 |
|---|---|---|
| Counter tax invoices | RCP/26-27/0001 | RCP/26-27/0001 |
| Monthly account invoices | RCPA/26-27/001 | RCPA/26-27/001 |
| Credit notes | RCPCN/26-27/001 | RCPCN/26-27/001 |
| Debit notes | RCPDN/26-27/001 | RCPDN/26-27/001 |
| Receipt vouchers (money received on account) | RV-001 | RV-001 |
| Petty cash vouchers | PCV-001 | PCV-001 |

## 2.12 Setup checklist

- [ ] Business form decided with CA; PAN ready
- [ ] Current account opened; UPI QR linked to it
- [ ] GST registration (if required or chosen); GSTIN printed on bill format
- [ ] Carrier agreements signed; all points from 2.3 noted in writing
- [ ] Rate card made (2.6), printed and dated
- [ ] Weighing scale (checked for accuracy), measuring tape, camera/phone for parcel photos
- [ ] Bill book or billing software with numbering set (2.11)
- [ ] Books opened: day book, cash book, bank book, ledgers, sales register, purchase register, TBB register, COD register, petty cash book (2.8)
- [ ] Petty cash float fixed and placed in a separate box (2.10)
- [ ] Opening entries written: owner's capital put in, security deposit paid, equipment bought (Part 5)
- [ ] File or folder for manifests, carrier bills, COD remittance files, bank statements

---

# Part 3: A full working day, hour by hour

**Example day: Wednesday, 14 October 2026, at Raju Courier Point.** All names, AWB numbers and amounts are examples. We follow the day, then show every book page filled in (3.13).

### The booking routine (do this for every parcel)

1. **Ask what is inside.** Refuse items your carrier does not allow (check the carrier's banned/restricted list: for example cash, liquids, batteries, flammable items).
2. **Check the destination pincode** for serviceability (does the carrier deliver there? COD? ODA charge?).
3. **Write full details:** sender name, phone, address; receiver name, phone, full address with pincode; contents; number of pieces.
4. **Ask the declared value** (value of the goods). If it is **above ₹50,000**, ask for the **e-way bill number** before accepting (7.7).
5. **Weigh** the packed parcel. **Measure** length, breadth and height in cm. **Take a photo** of the parcel on the scale and with the tape showing the sizes. Save the photos under the AWB number (this protects you in weight disputes, 4.5).
6. **Work out the chargeable weight** (2.4).
7. **Work out the price** from the rate card (2.6).
8. **Generate the AWB**, stick the label, and **make the bill** (or note it in the TBB register for an account customer).
9. **Take payment** (cash or UPI) and **write it in the books at once**. Do not wait until evening.

## 3.1 9:00 am: opening the shop

Raju opens the shop and does four things:

1. **Counts the cash drawer.** It has **₹3,200**, the same as last night's closing figure in the cash book. Good. (If it did not match, he would find out why before starting.)
2. **Checks the petty cash box.** ₹2,000 in cash, no vouchers. Matches the float.
3. **Checks the rate card date.** FastTrack has not changed the fuel surcharge; it is still 20%.
4. **Checks yesterday's NDR and RTO list** in the carrier app. One parcel had "receiver not available". He calls the sender and gives the carrier a new delivery time.

## 3.2 10:05 am: Ramesh sends documents to Pune (cash)

Ramesh, a walk-in customer, brings an envelope of documents for an office in Pune city.

- **Actual weight:** 0.2 kg. **Size:** 35 × 25 × 1 cm, so volumetric = 875 ÷ 5000 = 0.175 kg.
- **Higher weight:** 0.2 kg. Rounded up to the slab: **0.5 kg chargeable**.
- **Zone:** A (local). Counter rate.

**Making the bill, line by line:**

| Line | Working | ₹ |
|---|---|---|
| Freight, Zone A, 0.5 kg | first slab only | 50.00 |
| Fuel surcharge 20% | 50 × 20% | 10.00 |
| Docket charge | flat | 40.00 |
| **Taxable value** (amount before GST) | 50 + 10 + 40 | **100.00** |
| CGST 9% | 100 × 9% | 9.00 |
| SGST 9% | 100 × 9% | 9.00 |
| **Total** | 100 + 9 + 9 | **118.00** |

Bill number **RCP/26-27/0412**, AWB **5001001**. Ramesh pays **₹118 cash**. Raju writes it straight away in the day book and cash book.

## 3.3 10:40 am: Sunita sends a big box to Bengaluru (UPI)

Sunita brings a box of clothes and gifts for her daughter in Bengaluru. She says the goods are worth **₹20,000** and wants them covered for carrier's risk, so FOV applies.

**Step 1: Weight.**
- Actual weight on the scale: **3.2 kg**.
- Size: 40 × 30 × 20 cm. Volumetric = 40 × 30 × 20 ÷ 5000 = 24,000 ÷ 5000 = **4.8 kg**.
- Higher of 3.2 and 4.8 = 4.8 kg. Round up to the slab = **5.0 kg chargeable**.

Raju shows Sunita why: "Your box is light but big, so the carrier charges by size." Some customers then repack in a smaller box, which is their choice.

**Step 2: Price** (Zone C metro, counter rate). 5.0 kg = 10 slabs of 0.5 kg.

| Line | Working | ₹ |
|---|---|---|
| Freight, Zone C, 5.0 kg | ₹70 first 0.5 kg + 9 × ₹50 extra slabs = 70 + 450 | 520.00 |
| Fuel surcharge 20% | 520 × 20% | 104.00 |
| Docket charge | flat | 40.00 |
| FOV 0.2% of declared value ₹20,000 | 20,000 × 0.2% | 40.00 |
| **Taxable value** | 520 + 104 + 40 + 40 | **704.00** |
| CGST 9% | 704 × 9% | 63.36 |
| SGST 9% | 704 × 9% | 63.36 |
| **Invoice total** | 704 + 63.36 + 63.36 | **830.72** |
| Round off | to the nearest rupee | +0.28 |
| **Amount to pay** | | **831.00** |

**What is "round off"?** Paise are hard to handle, so many shops round the final amount to the nearest rupee (50 paise and above go up, below 50 paise go down) and show the small difference as a "Round off" line. The tax amounts stay exact. Over a month, round-offs are a tiny income or expense in a "Round off" account. 🔎 **Ask your CA to confirm** that your rounding method is fine.

**Step 3: Payment.** Sunita pays **₹831 by UPI**. That goes in the **bank book**, not the cash book. Raju checks the UPI app shows ₹831 received **before** handing over the bill.

**Step 4: The bill.** Bill number **RCP/26-27/0413**, AWB **5001002**. Because Sunita is not registered and the value is below ₹50,000, her full address is not required on the tax invoice unless she asks for it (Rule 46) [20]. Raju still keeps the sender and receiver address on the AWB booking. (See the full B2C invoice layout in 6.6.)

**Margin on this parcel** (for Raju's own understanding, not on the bill):

| | Working | ₹ |
|---|---|---|
| Sale (taxable value) | from above | 704.00 |
| Carrier freight | ₹40 + 9 × ₹32 = 40 + 288 | 328.00 |
| Carrier fuel surcharge 20% | 328 × 20% | 65.60 |
| Carrier docket | flat | 15.00 |
| Carrier FOV 0.1% | 20,000 × 0.1% | 20.00 |
| **Carrier cost** | 328 + 65.60 + 15 + 20 | **428.60** |
| **Margin** | 704.00 − 428.60 | **275.40** (about 39%) |

## 3.4 11:00 am: tea for the staff (petty cash)

The helper buys tea for ₹60. Raju writes **petty cash voucher PCV-061: Tea for staff, ₹60**, and the helper signs it. Petty box: 2,000 − 60 = ₹1,940 cash + ₹60 voucher = ₹2,000. ✔

## 3.5 12:15 pm: Meena Online Store drops a COD parcel (account customer)

Meena runs a small online store and is an **account customer** (billed weekly, together with her COD settlement). She drops one parcel for a buyer in Delhi. The buyer will pay **₹1,499** to the delivery boy, so this is a **COD** parcel.

- Actual 0.4 kg; size 20 × 15 × 10 = 3,000 ÷ 5000 = 0.6 kg volumetric; higher = 0.6 kg; chargeable = **1.0 kg**.
- Zone C, **account rate**. 1.0 kg = 2 slabs.

| Line | Working | ₹ |
|---|---|---|
| Freight | ₹55 + 1 × ₹40 | 95.00 |
| Fuel surcharge 20% | 95 × 20% | 19.00 |
| Docket | account rate | 20.00 |
| COD charge | higher of ₹50 or 2% of 1,499 (= ₹29.98), so ₹50 | 50.00 |
| **Amount before GST** (billed at month end) | 95 + 19 + 20 + 50 | **184.00** |

**No money changes hands now.** Raju:
1. Writes the parcel in the **TBB register**. Meena's agreement is **weekly billing**, so this goes on her next weekly invoice, and GST will be added on that invoice (4.1).
2. Writes a new line in the **COD register**: AWB 5001003, COD ₹1,499, status "Booked".

**Why COD is special.** The ₹1,499 the buyer pays is the price of Meena's goods. It is **not Raju's income**. Only the ₹50 COD charge (and the freight) is income. Until Raju passes the ₹1,499 on to Meena, it is money he **owes** her, a **liability** (*doosre ka paisa*). Mixing COD money with your own income is one of the biggest mistakes in courier books.

## 3.6 1:00 pm: Shree Textiles pays old dues in cash

Shree Textiles, an account customer, sends a staff member with **₹1,500 cash** towards its old unpaid bills. Raju:
1. Counts the cash and gives a **receipt voucher RV-018** ("Received from Shree Textiles ₹1,500 cash against outstanding bills").
2. Writes it in the **cash book** (money in) and in **Shree Textiles' ledger** (reduces what they owe).

This is **not** a new sale. The sale was recorded when the bill was made. Today's cash only reduces their *udhaar*.

## 3.7 2:00 pm: rider picks up two parcels from Patil Electricals (account customer)

The rider goes to Patil Electricals (a registered company, account customer) and brings back two parcels. The rider's auto fare home is ₹120 (**PCV-062: Auto fare for pickup, ₹120**).

| AWB | To | Actual kg | Size (cm) and volumetric kg | Chargeable kg | Zone | Working (account rate) | Before GST ₹ |
|---|---|---|---|---|---|---|---|
| 5001004 | Lucknow | 1.3 | 25×20×15 = 7,500 ÷ 5000 = 1.5 | 1.5 | D | Freight 65 + 2×48 = 161; FSC 32.20; docket 20 | 213.20 |
| 5001005 | Nashik | 0.8 | 20×15×10 = 3,000 ÷ 5000 = 0.6 | 1.0 | B | Freight 48 + 36 = 84; FSC 16.80; docket 20 | 120.80 |

Both go into the **TBB register** for the October invoice. Total today for Patil: 213.20 + 120.80 = **₹334.00** before GST.

## 3.8 3:00 pm: Abdul sends a carton to Nagpur (cash)

- Actual **2.0 kg**; size 30 × 30 × 20 = 18,000 ÷ 5000 = **3.6 kg** volumetric. Higher = 3.6 kg. Chargeable = **4.0 kg** (8 slabs).
- Zone B (same state), counter rate.

| Line | Working | ₹ |
|---|---|---|
| Freight | ₹60 + 7 × ₹45 = 60 + 315 | 375.00 |
| Fuel surcharge 20% | 375 × 20% | 75.00 |
| Docket | | 40.00 |
| **Taxable value** | | **490.00** |
| CGST 9% | 490 × 9% | 44.10 |
| SGST 9% | 490 × 9% | 44.10 |
| **Invoice total** | | **578.20** |
| Round off | | −0.20 |
| **Amount to pay** | | **578.00** |

Bill **RCP/26-27/0414**, AWB **5001006**. **₹578 cash.**

## 3.9 4:00 pm: bank deposit, packing material, follow-up calls

- The drawer has a lot of cash, so Raju deposits **₹4,000** in the bank. Cash book: money out ₹4,000. Bank book: money in ₹4,000. This is a **contra entry** (moving your own money from one pocket to another). It is **not** income and **not** an expense.
- The helper buys tape and bubble wrap for ₹240 (**PCV-063: Packing material, ₹240**), with the shop's bill attached.
- Raju calls two receivers whose parcels got an NDR yesterday and updates their addresses in the carrier app.

## 3.10 5:30 pm: Kavita sends a parcel to Patna (UPI)

- Actual 0.9 kg; size 25 × 20 × 10 = 5,000 ÷ 5000 = 1.0 kg; chargeable = **1.0 kg** (2 slabs). Zone D, counter rate.

| Line | Working | ₹ |
|---|---|---|
| Freight | ₹85 + ₹60 | 145.00 |
| Fuel surcharge 20% | 145 × 20% | 29.00 |
| Docket | | 40.00 |
| **Taxable value** | | **214.00** |
| CGST 9% | 214 × 9% | 19.26 |
| SGST 9% | 214 × 9% | 19.26 |
| **Invoice total** | | **252.52** |
| Round off | | +0.48 |
| **Amount to pay** | | **253.00** |

Bill **RCP/26-27/0415**, AWB **5001007**. **₹253 by UPI.**

## 3.11 6:00–6:30 pm: making the manifest and handing over to the carrier

At 6:00 pm Raju prints the **manifest**: the list of every parcel going to FastTrack today. He checks that the **number of parcels on the table equals the number of rows on the list** (7 = 7), and that every AWB label matches.

At 6:30 pm the FastTrack pickup person scans each parcel. Raju and the pickup person **both sign** the manifest. Raju keeps a signed copy (or the scan confirmation in the app) in the manifest file. If a parcel is later "lost", this signed list proves it was handed over.

## 3.12 7:00–7:30 pm: counting cash and closing the day

**Step 1: Work out what the cash should be (expected cash).**

| Line | ₹ |
|---|---|
| Opening cash (yesterday's closing) | 3,200.00 |
| + Cash bookings today (Ramesh 118 + Abdul 578) | 696.00 |
| + Cash received from account customers (Shree Textiles) | 1,500.00 |
| − Cash expenses paid from the drawer (all small expenses went through petty cash today) | 0.00 |
| − Cash deposited in bank | (4,000.00) |
| **= Expected closing cash** | **1,396.00** |

Check: 3,200 + 696 + 1,500 − 4,000 = 1,396. ✔

**Step 2: Count the actual cash, note by note (denomination count).**

| Note / coin | How many | ₹ |
|---|---|---|
| ₹500 | 1 | 500 |
| ₹200 | 2 | 400 |
| ₹100 | 3 | 300 |
| ₹50 | 1 | 50 |
| ₹20 | 4 | 80 |
| ₹10 | 4 | 40 |
| Coins | | 16 |
| **Total counted** | | **1,386** |

**Step 3: Compare.** Expected 1,396 − counted 1,386 = **₹10 short**.

Raju re-checks the bills and the count but cannot find the ₹10. Probably wrong change was given. He writes it in the cash book as **"Cash short ₹10"** (an expense in a "Cash short/excess" account) so that the cash book now shows ₹1,386, the same as the drawer.

**Rules for cash closing:**
- Count every day. Do not skip days.
- **Never "fix" a difference by changing a bill or a booking.** Record the shortage or excess honestly as "Cash short/excess".
- If shortages repeat, find the cause (wrong change, a missing bill, theft).
- Check UPI and card totals separately against the payment app's report. Those go to the **bank book**: today, UPI = 831 + 253 = **₹1,084**.

**Step 4: Check petty cash.** Box: 2,000 − 60 − 120 − 240 = **₹1,580** cash. Vouchers: 60 + 120 + 240 = ₹420. 1,580 + 420 = **₹2,000**. ✔

**Step 5: Daily summary and backup.** Raju fills the daily summary (3.13 i), takes a backup of the software or photographs the book pages, and locks the cash.

## 3.13 The filled book pages for 14 October 2026 (EXAMPLE)

### (a) Day book (*rojmel*): every event in time order

| Time | Voucher / bill no. | Particulars | Mode | Money in ₹ | Money out ₹ |
|---|---|---|---|---|---|
| 09:00 | – | Opening cash in drawer ₹3,200; petty cash box ₹2,000 | – | – | – |
| 10:05 | RCP/26-27/0412 | Ramesh, AWB 5001001, Pune local, 0.5 kg (taxable 100 + GST 18) | Cash | 118.00 | |
| 10:40 | RCP/26-27/0413 | Sunita, AWB 5001002, Bengaluru, 5.0 kg (taxable 704 + GST 126.72 + round off 0.28) | UPI | 831.00 | |
| 11:00 | PCV-061 | Tea for staff | Petty cash | | 60.00 |
| 12:15 | TBB | Meena Online Store, AWB 5001003, Delhi, 1.0 kg, COD ₹1,499; ₹184 to be billed | Credit | – | – |
| 13:00 | RV-018 | Received from Shree Textiles against old bills | Cash | 1,500.00 | |
| 14:00 | TBB | Patil Electricals, AWB 5001004 Lucknow (213.20) and 5001005 Nashik (120.80); to be billed | Credit | – | – |
| 14:20 | PCV-062 | Auto fare for pickup | Petty cash | | 120.00 |
| 15:00 | RCP/26-27/0414 | Abdul, AWB 5001006, Nagpur, 4.0 kg (taxable 490 + GST 88.20 − round off 0.20) | Cash | 578.00 | |
| 16:00 | Contra | Cash deposited in bank | Cash → Bank | (bank) 4,000.00 | (cash) 4,000.00 |
| 16:30 | PCV-063 | Tape and bubble wrap (bill attached) | Petty cash | | 240.00 |
| 17:30 | RCP/26-27/0415 | Kavita, AWB 5001007, Patna, 1.0 kg (taxable 214 + GST 38.52 + round off 0.48) | UPI | 253.00 | |
| 18:30 | M-1014 | Manifest of 7 AWBs handed to FastTrack (signed) | – | – | – |
| 19:30 | – | Cash short found at closing | Cash | | 10.00 |

### (b) Cash book (*rokad bahi*): only the cash drawer

| Time | Particulars | Cash in ₹ | Cash out ₹ | Balance ₹ |
|---|---|---|---|---|
| 09:00 | Opening balance | | | 3,200.00 |
| 10:05 | Bill 0412, Ramesh | 118.00 | | 3,318.00 |
| 13:00 | RV-018, Shree Textiles | 1,500.00 | | 4,818.00 |
| 15:00 | Bill 0414, Abdul | 578.00 | | 5,396.00 |
| 16:00 | Deposited in bank (contra) | | 4,000.00 | 1,396.00 |
| 19:30 | Cash short | | 10.00 | 1,386.00 |
| | **Totals** | **2,196.00** | **4,010.00** | **Closing 1,386.00** |

Check: 3,200 + 2,196 − 4,010 = 1,386 = cash counted. ✔

### (c) Bank book: only the bank account

| Time | Particulars | In ₹ | Out ₹ | Balance ₹ |
|---|---|---|---|---|
| | Opening balance (example) | | | 52,300.00 |
| 10:40 | UPI, bill 0413, Sunita | 831.00 | | 53,131.00 |
| 16:00 | Cash deposit (contra) | 4,000.00 | | 57,131.00 |
| 17:30 | UPI, bill 0415, Kavita | 253.00 | | 57,384.00 |

### (d) Sales register: counter tax invoices of the day

| Bill no. | Customer | AWB | Taxable ₹ | CGST 9% ₹ | SGST 9% ₹ | Round off ₹ | Total ₹ | Mode |
|---|---|---|---|---|---|---|---|---|
| RCP/26-27/0412 | Ramesh | 5001001 | 100.00 | 9.00 | 9.00 | 0.00 | 118.00 | Cash |
| RCP/26-27/0413 | Sunita | 5001002 | 704.00 | 63.36 | 63.36 | +0.28 | 831.00 | UPI |
| RCP/26-27/0414 | Abdul | 5001006 | 490.00 | 44.10 | 44.10 | −0.20 | 578.00 | Cash |
| RCP/26-27/0415 | Kavita | 5001007 | 214.00 | 19.26 | 19.26 | +0.48 | 253.00 | UPI |
| | **Totals** | | **1,508.00** | **135.72** | **135.72** | **+0.56** | **1,780.00** | |

Check: 1,508.00 + 135.72 + 135.72 + 0.56 = 1,780.00. Also cash 118 + 578 = 696 and UPI 831 + 253 = 1,084; 696 + 1,084 = 1,780. ✔

### (e) Account booking (TBB) register: to be billed at the end of each customer's billing cycle (Meena weekly, Patil monthly)

| Date | Customer | AWB | To | Chargeable kg | Zone | Freight ₹ | FSC ₹ | Docket ₹ | COD charge ₹ | Before GST ₹ |
|---|---|---|---|---|---|---|---|---|---|---|
| 14-Oct | Meena Online Store | 5001003 | Delhi | 1.0 | C | 95.00 | 19.00 | 20.00 | 50.00 | 184.00 |
| 14-Oct | Patil Electricals | 5001004 | Lucknow | 1.5 | D | 161.00 | 32.20 | 20.00 | – | 213.20 |
| 14-Oct | Patil Electricals | 5001005 | Nashik | 1.0 | B | 84.00 | 16.80 | 20.00 | – | 120.80 |
| | **Day total** | | | | | | | | | **518.00** |

### (f) COD register (Meena Online Store rows; other AWBs are examples from earlier days)

| AWB | Consignor | COD value ₹ | Booked | Delivered on | Carrier remittance ref / date | Amount received ₹ | Paid to consignor (date / ref) | Status |
|---|---|---|---|---|---|---|---|---|
| 5000990 | Meena Online Store | 899 | 10-Oct | 13-Oct | – | – | – | **Delivered, not remitted** |
| 5000995 | Meena Online Store | 2,250 | 11-Oct | RTO | n/a | n/a | n/a | RTO, no COD |
| 5001003 | Meena Online Store | 1,499 | 14-Oct | – | – | – | – | Booked |

(Later the 5001003 row is completed: delivered 16-Oct, remitted by carrier 23-Oct, paid to Meena 24-Oct. See 4.1.)

### (g) Petty cash book (float ₹2,000)

| Time | Voucher | Particulars | Expense head | Paid ₹ | Balance ₹ |
|---|---|---|---|---|---|
| 09:00 | | Opening balance | | | 2,000.00 |
| 11:00 | PCV-061 | Tea for staff | Staff welfare | 60.00 | 1,940.00 |
| 14:20 | PCV-062 | Auto fare, pickup | Local conveyance | 120.00 | 1,820.00 |
| 16:30 | PCV-063 | Tape, bubble wrap | Packing material | 240.00 | 1,580.00 |
| | | **Total vouchers** | | **420.00** | |

### (h) Manifest M-1014 (handed to FastTrack, 6:30 pm)

| # | AWB | Consignee city | Pieces | Actual kg | Chargeable kg | Service | COD ₹ |
|---|---|---|---|---|---|---|---|
| 1 | 5001001 | Pune | 1 | 0.2 | 0.5 | Surface | – |
| 2 | 5001002 | Bengaluru | 1 | 3.2 | 5.0 | Surface | – |
| 3 | 5001003 | Delhi | 1 | 0.4 | 1.0 | Surface | 1,499 |
| 4 | 5001004 | Lucknow | 1 | 1.3 | 1.5 | Surface | – |
| 5 | 5001005 | Nashik | 1 | 0.8 | 1.0 | Surface | – |
| 6 | 5001006 | Nagpur | 1 | 2.0 | 4.0 | Surface | – |
| 7 | 5001007 | Patna | 1 | 0.9 | 1.0 | Surface | – |
| | **Totals** | | **7** | **8.8** | **14.0** | | **1,499** |

Signed: Raju (shop) ______  FastTrack pickup ______  Time: 18:30

### (i) Daily summary and margin

| AWB | Customer | Sale before GST ₹ | Expected carrier cost before GST ₹ | Margin ₹ |
|---|---|---|---|---|
| 5001001 | Ramesh | 100.00 | 48.60 (28 + 5.60 FSC + 15 docket) | 51.40 |
| 5001002 | Sunita | 704.00 | 428.60 | 275.40 |
| 5001003 | Meena | 184.00 | 136.40 (72 + 14.40 + 15 + 35 COD) | 47.60 |
| 5001004 | Patil | 213.20 | 163.80 (124 + 24.80 + 15) | 49.40 |
| 5001005 | Patil | 120.80 | 87.00 (60 + 12 + 15) | 33.80 |
| 5001006 | Abdul | 490.00 | 274.20 (216 + 43.20 + 15) | 215.80 |
| 5001007 | Kavita | 214.00 | 118.20 (86 + 17.20 + 15) | 95.80 |
| | **Total** | **2,026.00** | **1,256.80** | **769.20 (about 38%)** |

Money received today: cash ₹696 (bookings) + ₹1,500 (old dues); UPI ₹1,084. Account bookings to bill later: ₹518.00 + GST. This was a quiet example day. Part 5 looks at a full, busy month.

---

# Part 4: Weekly and monthly work

Daily work keeps the counter running. Weekly and monthly work makes sure **you actually get the money you earned, pay only what you really owe, and hand over other people's money correctly.**

## 4.1 Weekly: COD money reconciliation (*COD ka hisaab milana*)

### Why COD is special (recap)

When an online seller (your consignor) sends COD parcels, the carrier collects the **goods value** from the buyer. That money:
- is **not your income** (only your COD *charge* is),
- is kept by the carrier, then sent on (**remitted**, minus deductions) to you or the aggregator on a cycle (for example weekly, or D+2/D+7 after delivery),
- must then be paid to the consignor. Until then it is a **liability** called "COD payable to consignor".

**"Reconciliation"** (*milaan*) means comparing two lists that should agree, and finding out why they do not.

### Worked example (example numbers): Meena Online Store, parcels delivered 12–18 October

1. **Your list (tracking/POD):** 40 COD parcels delivered, total COD value **₹62,000**.
2. **Carrier's remittance file** (received 23 October): 37 AWBs, **₹57,400**, and ₹57,400 credited to the bank. ✔ bank matches file.
3. **Difference:** 3 AWBs worth **₹4,600** were delivered but not remitted (62,000 − 57,400 = 4,600). One of them is AWB 5000990 (₹899). Raise all three with the carrier, and keep them as "pending" in the COD register.
4. **Meena's charges for the week** (her weekly invoice): forward freight ₹3,200 + COD charges ₹1,200 + RTO ₹450 = ₹4,850, + GST 18% ₹873 (4,850 × 18% = 873) = **₹5,723**.
5. **Payment to Meena** = COD received 57,400 − her invoice 5,723 = **₹51,677**, paid 24 October by bank.

Netting (cutting your bill from her COD money) is only allowed if your agreement says so. Otherwise pay the full COD and collect the invoice separately. Even when netting, the books must show **both** the invoice **and** the full COD payment clearly (the journal entries are in 5.2).

**Golden rules for COD:**
- Match **by AWB**, never by totals alone.
- Keep the carrier's remittance file (CSV/PDF) as evidence.
- Track "delivered but not remitted" amounts by age, like money customers owe you. Chase anything older than the carrier's cycle.
- Never use COD money to pay your own expenses, even for a day.

## 4.2 Weekly: other jobs

- **Petty cash top-up.** Add up the week's vouchers (for example ₹1,350) and put exactly that much cash back in the box from the bank or drawer, with an entry. The box is back to ₹2,000.
- **Follow up NDR and RTO** parcels; bill RTO charges to the customer if your terms allow.
- **Payment follow-up** for account customers (4.3).
- **Check payables due** to carriers this week (4.4).

## 4.3 Collecting money from account customers

### Month end: send the monthly invoices

On the last day of the month, Raju takes each monthly customer's rows from the TBB register and makes **one invoice listing all their AWBs**. Patil Electricals' October invoice (full layout in 6.5):

| Date | AWB | To | Chargeable kg | Zone | Before GST ₹ |
|---|---|---|---|---|---|
| 02-Oct | 5000811 | Mumbai | 2.0 | B | 207.20 |
| 09-Oct | 5000940 | Chennai | 0.5 | C | 86.00 |
| 14-Oct | 5001004 | Lucknow | 1.5 | D | 213.20 |
| 14-Oct | 5001005 | Nashik | 1.0 | B | 120.80 |
| 22-Oct | 5001210 | Hyderabad | 3.0 | C | 326.00 |
| | | | | **Taxable value** | **953.20** |
| | | | | CGST 9% | 85.79 |
| | | | | SGST 9% | 85.79 |
| | | | | Round off | +0.22 |
| | | | | **Invoice total** | **1,125.00** |

Working: 207.20 + 86.00 + 213.20 + 120.80 + 326.00 = 953.20. 953.20 × 9% = 85.788, rounded to 85.79. 953.20 + 85.79 + 85.79 = 1,124.78; round off +0.22 = 1,125.00.

Send the invoice (PDF or link) with a **statement** showing the opening balance, new invoices, payments and the balance due. Write the due date (for example "payable within 15 days").

### When the customer pays, and TDS is cut

On 10 November, Patil pays the October invoice. Patil Electricals is a company that must deduct TDS (Tax Deducted at Source: the payer cuts a small part of your bill and pays it to the income-tax department in **your** name; you get credit for it later). Here the example rate is 1% (rate for an individual/proprietor payee) on the amount before GST:
- TDS = 953.20 × 1% = **₹9.53**
- Patil pays by NEFT: 1,125.00 − 9.53 = **₹1,115.47**
- Raju records: bank ₹1,115.47 + **TDS receivable** ₹9.53 = ₹1,125.00 cleared from Patil's ledger.

TDS is **not a loss**. It is tax already paid for you. Check it appears in your **Form 26AS / AIS** (income-tax statements against your PAN) every quarter, and claim it in your income-tax return (7.9).

### Patil Electricals' ledger page (*khata*), example

| Date | Particulars | They owe you (Debit) ₹ | They paid (Credit) ₹ | Balance ₹ |
|---|---|---|---|---|
| 01-Oct | Opening balance | | | 0.00 |
| 31-Oct | Invoice RCPA/26-27/031 | 1,125.00 | | 1,125.00 Dr |
| 10-Nov | NEFT received | | 1,115.47 | 9.53 Dr |
| 10-Nov | TDS deducted by Patil (1% of 953.20) | | 9.53 | 0.00 |

("Dr" after a balance means the customer still owes you. Debit and credit are explained in 5.1.)

### Aging: how old are the unpaid bills? (*kitne din se baaki*)

For account customers the key report is **receivables aging**: how much each customer owes, grouped by how old the unpaid bills are.

**Example aging as of 31 October (example numbers):**

| Consignor | 0–30 days ₹ | 31–60 ₹ | 61–90 ₹ | 90+ ₹ | Total ₹ |
|---|---|---|---|---|---|
| Desai Pharma | 17,375 | 6,200 | – | – | 23,575 |
| Patil Electricals | 1,125 | – | – | – | 1,125 |
| Shree Textiles | 9,000 | 9,400 | 7,800 | 3,100 | 29,300 |
| Meena Online Store | 4,200 | – | – | – | 4,200 |
| **Total** | **31,700** | **15,600** | **7,800** | **3,100** | **58,200** |

Check rows: 17,375 + 6,200 = 23,575; 9,000 + 9,400 + 7,800 + 3,100 = 29,300. Check total: 23,575 + 1,125 + 29,300 + 4,200 = 58,200; and 31,700 + 15,600 + 7,800 + 3,100 = 58,200. ✔

**Action:** Shree Textiles owes ₹10,900 that is older than 60 days (7,800 + 3,100). Send a reminder, stop further credit, or move them to cash bookings. A similar **payables aging** shows what you owe each carrier.

If a customer will clearly never pay, the CA may advise writing it off as a **bad debt** (an expense). Do not just delete the bill.

## 4.4 Paying carrier bills (payables)

Carriers bill you every week or month (or deduct from a prepaid wallet per parcel). **Before paying, check the bill line by line against your own booking records**: AWB, chargeable weight, zone, rate, surcharges, RTO and COD charges.

**Example check: FastTrack's bill lines for 14 October (example numbers, before GST)**

| AWB | Your chargeable kg | Carrier's kg | Your expected cost ₹ | Carrier billed ₹ | Difference ₹ |
|---|---|---|---|---|---|
| 5001001 | 0.5 | 0.5 | 48.60 | 48.60 | 0.00 |
| 5001002 | 5.0 | 5.0 | 428.60 | 428.60 | 0.00 |
| 5001003 | 1.0 | **2.5** | 136.40 | 251.60 | **115.20** |
| 5001004 | 1.5 | 1.5 | 163.80 | 163.80 | 0.00 |
| 5001005 | 1.0 | 1.0 | 87.00 | 87.00 | 0.00 |
| 5001006 | 4.0 | 4.0 | 274.20 | 274.20 | 0.00 |
| 5001007 | 1.0 | 1.0 | 118.20 | 118.20 | 0.00 |
| **Total** | | | **1,256.80** | **1,372.00** | **115.20** |

One line is wrong: AWB 5001003 (see 4.5).

**Keeping the carrier's ledger (payables).** Each carrier is a creditor (someone you owe money to). Track: bill number and date, amount, GST on it (for input tax credit), due date, payments, any TDS you deducted (if applicable, 7.9), and credit notes for disputes.

**Wallet-based carriers or aggregators.** A wallet top-up is an **advance** (an asset, like money kept with a shop in advance), not an expense. The expense happens as parcels are booked and charged. Match the wallet statement against your bookings every month.

## 4.5 Weight disputes (*wazan ka jhagda*)

The carrier's hub measures parcels again. If its weight (often volumetric) is higher than what you declared, it adds a **weight discrepancy** charge.

**Example (AWB 5001003):** You declared 1.0 kg. The hub measured 30 × 25 × 15 cm: 11,250 ÷ 5000 = 2.25 kg, so 2.5 kg chargeable. Extra = 2.5 − 1.0 = 1.5 kg = 3 extra slabs.
- Extra carrier freight: 3 × ₹32 = ₹96; fuel surcharge 20% = ₹19.20; **extra charge = ₹115.20** + GST ₹20.74 (115.20 × 18%) = ₹135.94.

**What to do:**
1. **Keep evidence at booking:** photo of the parcel on the scale, and photo with the tape showing L × B × H, saved against the AWB. (Raju's photo shows 20 × 15 × 10 cm, so he has a strong case.)
2. **Raise a dispute within the carrier's window** (often only a few days).
3. **Record the extra charge as a payable** for now. If you win, record the carrier's **credit note** (a document that reduces the bill; 6.8).
4. **Decide your policy:** pass the extra to the customer with a **debit note** (a document that adds to an earlier bill; 6.8) or absorb it. If Raju had lost and passed it on to Meena at her account rate: 3 × ₹40 = ₹120 + FSC ₹24 = ₹144 + GST ₹25.92 = **₹169.92 debit note**.
5. Track **"weight-discrepancy cost"** as its own expense head. It is a leakage number to watch (9.4).

## 4.6 Bank matching (bank reconciliation)

Once a month, compare your **bank book** with the **bank statement**. They often differ for timing reasons, and you must explain every difference.

**Example on 31 October (example numbers):**
1. Your bank book showed ₹1,35,180. The statement shows bank charges of ₹180 that you had not written. **Write them in the bank book now:** 1,35,180 − 180 = **₹1,35,000** (corrected bank book).
2. Now explain the remaining difference:

| Line | ₹ |
|---|---|
| Balance as per corrected bank book | 1,35,000 |
| + Cheque you gave a supplier, not yet cashed by them (the bank has not paid it yet) | 8,000 |
| − Customer cheque you deposited, not yet credited by the bank | (2,500) |
| **= Balance as per bank statement** | **1,40,500** |

Check: 1,35,000 + 8,000 − 2,500 = 1,40,500. ✔ If anything is still unexplained, find it before closing the month. Also match UPI and card settlement reports with the statement.

## 4.7 Expenses

**Typical expense heads:** rent, electricity, internet and phone, staff salaries, rider incentives, fuel, vehicle maintenance, packing material, printer, labels and stationery, software subscriptions, bank/UPI/card charges, franchise fees or royalty, insurance, accountant fees, repairs, local conveyance, tea and refreshments, marketing.

**Rules:**
- A **bill or voucher for every expense**. For GST input tax credit, the supplier's bill must be a proper GST invoice with your GSTIN on it.
- Pay larger bills by **bank or UPI**. A business expense paid in cash **above ₹10,000 to one person in a day** is not allowed as an expense for income tax (₹35,000 for payments to goods-carriage operators) [11] (7.11).
- Write each expense under the right head every time, so the monthly totals are meaningful.

## 4.8 Salary

**Example (October):** counter assistant ₹20,000 + rider ₹18,000 + helper ₹16,000 = **₹54,000**.
- Pay by bank transfer and keep a **salary register**: name, days present, salary, advances deducted, net paid, date, signature or bank reference.
- Raju paid the counter assistant and helper (₹36,000) by bank on 31 October. The rider's ₹18,000 was paid on 1 November. On 31 October that ₹18,000 is **salary payable** (owed, not yet paid). It is still October's expense.
- Salary **advances** to staff are not expenses when given; they are recovered from later salary.
- Rules on PF, ESI, professional tax and minimum wages depend on staff numbers and your state. 🔎 **Ask your CA to confirm** what applies to you.

## 4.9 Month-end closing steps (in order)

1. All bookings of the month entered; the sales register's last bill number noted.
2. Monthly invoices raised to all monthly account customers, and statements sent.
3. COD register updated; every pending item has a follow-up.
4. Carrier bills checked line by line; disputes raised; bills entered in the purchase register.
5. All expense bills entered; petty cash topped up and checked.
6. Salaries entered (paid and payable).
7. Bank book matched with the bank statement (4.6).
8. Cash book balance equals counted cash.
9. Aging lists made (receivables and payables).
10. Profit and loss worked out (Part 5) and documents sent to the CA for GST (7.12).

---

# Part 5: Understanding the books, profit and loss, and balance sheet

## 5.1 Debit and credit, in simple words

Every money event has **two sides**. When Ramesh pays ₹118 cash: cash **comes in** to the drawer, **and** the shop **earns** income (and owes GST). Both sides are written. This is called **double entry** (*dohri entry*). One side is called **debit (Dr)**, the other **credit (Cr)**, and **the two sides are always equal**.

Think of a see-saw: whatever goes on one side must be balanced by the same amount on the other.

**Simple rules of thumb:**

| Write as **Debit (Dr)** | Write as **Credit (Cr)** |
|---|---|
| What **comes in** (cash or bank money received) | What **goes out** (cash or bank money paid) |
| **Who owes you** (a customer you billed on credit; *lena*) | **Whom you owe** (a carrier's unpaid bill, GST to pay, COD money to pass on; *dena*) |
| **Expenses and losses** (rent, salary, carrier cost) | **Income and gains** (freight income, commission) |
| Things you own going up (assets) | Owner's capital and loans |

"Debit" does not mean "bad" and "credit" does not mean "good". They are just the **left side** and **right side** of every entry.

## 5.2 The day's and month's events as journal entries (example ₹)

A **journal entry** is one money event written with its debit and credit sides.

| # | Event | Debit (Dr) | Credit (Cr) |
|---|---|---|---|
| 1 | Ramesh, cash booking ₹118 (100 + GST 18) | Cash 118.00 | Freight income 100.00; Output CGST 9.00; Output SGST 9.00 |
| 2 | Sunita, UPI booking ₹831 | Bank 831.00 | Freight income 704.00; Output CGST 63.36; Output SGST 63.36; Round off 0.28 |
| 3 | Abdul, cash booking ₹578 | Cash 578.00; Round off 0.20 | Freight income 490.00; Output CGST 44.10; Output SGST 44.10 |
| 4 | Patil's October invoice ₹1,125 | Patil Electricals 1,125.00 | Freight income 953.20; Output CGST 85.79; Output SGST 85.79; Round off 0.22 |
| 5 | Patil pays ₹1,115.47 by NEFT after TDS ₹9.53 | Bank 1,115.47; TDS receivable 9.53 | Patil Electricals 1,125.00 |
| 6 | Shree Textiles pays ₹1,500 cash for old bills | Cash 1,500.00 | Shree Textiles 1,500.00 |
| 7 | FastTrack's bill for 14-Oct parcels: ₹1,372.00 + GST | Carrier freight cost 1,372.00; Input CGST 123.48; Input SGST 123.48 | FastTrack (payable) 1,618.96 |
| 8 | FastTrack accepts the weight dispute: credit note ₹115.20 + GST 20.74 | FastTrack 135.94 | Carrier freight cost 115.20; Input CGST 10.37; Input SGST 10.37 |
| 9 | Raju pays FastTrack the balance (1,618.96 − 135.94) | FastTrack 1,483.02 | Bank 1,483.02 |
| 10 | Shop rent ₹25,000 by bank | Rent expense 25,000 | Bank 25,000 |
| 11 | Cash ₹4,000 deposited in bank | Bank 4,000 | Cash 4,000 (a **contra** entry: no income or expense) |
| 12 | Petty cash vouchers ₹420 | Staff welfare 60; Local conveyance 120; Packing material 240 | Petty cash 420 |
| 13 | Cash short ₹10 | Cash short/excess 10 | Cash 10 |
| 14 | October salaries ₹54,000 (₹36,000 paid, ₹18,000 due) | Salary expense 54,000 | Bank 36,000; Salary payable 18,000 |
| 15 | Carrier remits Meena's COD ₹57,400 | Bank 57,400 | COD payable – Meena 57,400 |
| 16 | Meena's weekly invoice ₹5,723 | Meena Online Store 5,723 | Freight & COD income 4,850; Output CGST 436.50; Output SGST 436.50 |
| 17 | Pay Meena ₹51,677 (COD net of her invoice) | COD payable – Meena 57,400 | Meena Online Store 5,723; Bank 51,677 |
| 18 | Raju takes ₹30,000 for home expenses | Drawings 30,000 | Bank 30,000 |
| 19 | Monthly depreciation on scale, computers, bike | Depreciation 3,000 | Equipment 3,000 |

Check a few: entry 2: 704 + 63.36 + 63.36 + 0.28 = 831.00 ✔. Entry 3: 578.00 + 0.20 = 578.20 = 490 + 44.10 + 44.10 ✔. Entry 7: 1,372.00 × 9% = 123.48; 1,372.00 + 123.48 + 123.48 = 1,618.96 ✔. Entry 17: 51,677 + 5,723 = 57,400 ✔.

Words used above:
- **Output GST**: GST you collected on your sales. You owe it to the government (a liability).
- **Input GST**: GST you paid on your purchases (carrier bills, rent, phone). You can usually subtract it from output GST (7.3).
- **Drawings**: money the owner takes for personal use. It is **not** a business expense. It reduces the owner's capital.
- **Depreciation** (*ghisaai*): equipment wears out over years. Instead of counting the full price of a ₹40,000 computer as one month's expense, you spread its cost over its useful life. 🔎 **Ask your CA to confirm** the rates.

**Trial balance (check of the books).** At month end, list the balance of every ledger. Total of debit balances must equal total of credit balances. If they do not, an entry was written only half-way. Find it.

## 5.3 Income is counted when earned, not only when cash arrives

Patil's October parcels are **October income**, even though Patil pays in November. The rider's October salary is an **October expense**, even though it is paid on 1 November. This way of counting is called the **accrual basis**, and it gives the true picture of each month. (Your CA will tell you which basis to use for tax.)

## 5.4 The story of Raju's October: profit and loss (P&L)

The **P&L** (profit and loss statement; *nafa-nuksaan khata*) answers one question: **"Did the business earn more than it spent this month?"** It is like a **movie of the month**.

Raju's quiet day in Part 3 was only one day. Over the whole of October his shop was much busier. These are his totals (**example numbers, all before GST**, because GST just passes through a registered business):

**Step 1: What did the shop earn? (Income)**

| **Income** | ₹ |
|---|---|
| Freight & charges: account bookings | 4,20,000 |
| Freight & charges: cash bookings | 1,80,000 |
| COD handling charges | 18,000 |
| Franchise commission received (second brand) | 25,000 |
| **Total income** | **6,43,000** |

Check: 4,20,000 + 1,80,000 + 18,000 + 25,000 = 6,43,000. ✔

**Step 2: What did it cost to send those parcels? (Direct costs)** These are costs that rise and fall with the number of parcels.

| **Direct costs** | ₹ |
|---|---|
| Carrier charges (FastTrack and others) | 3,85,000 |
| RTO charges not recovered from customers | 9,500 |
| Weight discrepancy charges absorbed | 6,500 |
| Packing material | 8,000 |
| **Total direct costs** | **4,09,000** |
| **Gross profit** (6,43,000 − 4,09,000) | **2,34,000** (36.4% of income) |

Gross profit % = 2,34,000 ÷ 6,43,000 = 0.364 = 36.4%. So out of every ₹100 earned, about ₹36 is left after paying the carriers and parcel costs.

**Step 3: What did it cost to run the shop? (Operating expenses)** These must be paid even on a slow month.

| **Operating expenses** | ₹ |
|---|---|
| Staff salaries (3 people) | 54,000 |
| Rider fuel & incentives | 22,000 |
| Shop rent | 25,000 |
| Electricity, internet, phone | 6,500 |
| Software, printer, stationery | 4,500 |
| Bank, UPI & card charges | 2,000 |
| Depreciation (scale, computers, bike) | 3,000 |
| Bad debt written off | 4,000 |
| Misc. (tea, small items, cash short, round off) | 3,000 |
| **Total operating expenses** | **1,24,000** |
| **Net profit before tax** (2,34,000 − 1,24,000) | **1,10,000** (17.1% of income) |

Check: 54,000 + 22,000 + 25,000 + 6,500 + 4,500 + 2,000 + 3,000 + 4,000 + 3,000 = 1,24,000. ✔ And 1,10,000 ÷ 6,43,000 = 0.171 = 17.1%.

**The story in one line:** the shop earned ₹6.43 lakh, paid ₹4.09 lakh to send the parcels, spent ₹1.24 lakh to run the shop, and made **₹1.10 lakh net profit before tax** (example).

**What is NOT in the P&L, and why:**

| Item | Why it is not income or expense |
|---|---|
| GST collected | It belongs to the government (liability). |
| COD values collected | They belong to the sellers (liability). |
| Security deposit given to the franchisor | You get it back (asset). |
| Wallet top-ups | Advance, until used for parcels (asset). |
| Loan repayments (the principal part) | Reduce a liability. (Interest **is** an expense.) |
| Owner's drawings | The owner's own money taken out (reduces capital). |
| Cash deposited in bank | Just moving money (contra). |

## 5.5 The balance sheet: a photo on one date

If the P&L is a movie of the month, the **balance sheet** is a **photo** taken on one date. It shows:
- **Assets** (*sampatti*): what the business **owns**, or what others owe it.
- **Liabilities** (*dena*): what the business **owes** others.
- **Capital** (*poonji*): what belongs to the owner = assets − liabilities.

So: **Assets = Liabilities + Capital**. Always.

**Raju Courier Point: balance sheet as of 31 October 2026 (example numbers)**

| **Assets** (what it owns) | ₹ | **Liabilities & Capital** (what it owes) | ₹ |
|---|---|---|---|
| Cash in hand (drawer + petty cash) | 4,900 | Payable to carriers | 74,540 |
| Bank balance (4.6) | 1,35,000 | COD payable to consignors | 38,500 |
| Receivables from account customers (4.3) | 58,200 | GST payable for October (7.4) | 39,060 |
| Aggregator wallet balance | 15,000 | Salary payable (rider, 4.8) | 18,000 |
| TDS receivable (cut by customers) | 6,300 | Loan (bike) | 40,000 |
| Franchise security deposit | 50,000 | **Owner's capital** (see below) | 1,49,300 |
| Equipment (after depreciation) | 90,000 | | |
| **Total** | **3,59,400** | **Total** | **3,59,400** |

Check assets: 4,900 + 1,35,000 + 58,200 + 15,000 + 6,300 + 50,000 + 90,000 = 3,59,400. ✔
Check other side: 74,540 + 38,500 + 39,060 + 18,000 + 40,000 + 1,49,300 = 3,59,400. ✔

**Owner's capital, step by step (example):**

| | ₹ |
|---|---|
| Capital on 1 October | 69,300 |
| + Net profit for October (5.4) | 1,10,000 |
| − Drawings in October (entry 18 in 5.2) | (30,000) |
| **= Capital on 31 October** | **1,49,300** |

**This is how the P&L and the balance sheet connect:** profit increases the owner's share; drawings reduce it.

**Reading the photo like a story:**
- Raju has ₹1,39,900 in cash and bank (4,900 + 1,35,000). But ₹38,500 of it is **sellers' COD money** and ₹39,060 is **GST owed** to the government. That money is **not his to spend**.
- Customers owe him ₹58,200, and ₹10,900 of that is old (Shree Textiles). He must collect it.
- He owes carriers ₹74,540 and the rider ₹18,000.

**Both sides must be equal.** If they are not, something was recorded only half-way. That is exactly why double entry catches mistakes.

---

# Part 6: Invoices in depth

## 6.1 What an invoice is, and why it matters so much

An **invoice** (bill; *bill / pakki raseed*) is the written proof of a sale: what you did, for whom, for how much, and how much tax. It is:
- the customer's proof of what they paid,
- **your** proof of income,
- the base of your GST return (every invoice goes into GSTR-1),
- and, for a business customer, their right to claim input tax credit.

So every invoice must be correct, numbered, and kept.

## 6.2 What must be on a GST tax invoice

Under CGST Rule 46 [20], a tax invoice generally needs:

| # | Item | Raju's example |
|---|---|---|
| 1 | Your (supplier's) name, address and **GSTIN** | Raju Courier Point, Pune, GSTIN 27XXXXX0000X1Z5 (sample) |
| 2 | **Invoice number**: consecutive, up to 16 characters (letters, numbers, "-" and "/" allowed), **unique for the financial year** | RCP/26-27/0413 |
| 3 | Date of issue | 14-10-2026 |
| 4 | Customer's name, address and **GSTIN** if the customer is registered (B2B) | Patil Electricals, GSTIN … |
| 5 | For an **unregistered** customer: name, delivery address, state and state code are needed if the taxable value is **₹50,000 or more**, or if the customer asks for them | Sunita: not required (below ₹50,000) |
| 6 | **SAC code** (service code; for courier 9968 / 996812) | 996812 |
| 7 | Description of the service | Courier service, AWB 5001002, Pune to Bengaluru, 5.0 kg |
| 8 | Total value and **taxable value** (after any discount) | 704.00 |
| 9 | GST **rate** and **amount**, split into CGST + SGST, or IGST | CGST 9% 63.36, SGST 9% 63.36 |
| 10 | **Place of supply** with state name (needed for inter-state supplies; good practice always) | Maharashtra (27) |
| 11 | Whether tax is payable on reverse charge | No |
| 12 | Signature or digital signature (not needed on an electronic invoice issued under the IT Act) | |
| 13 | QR code with IRN, only if e-invoicing applies (6.10) | Not applicable |

## 6.3 Invoice numbering: one series, no gaps, new each financial year

- The **financial year** in India runs **1 April to 31 March**. 14 October 2026 is in FY **2026-27**.
- Numbers must run **in order without gaps** in each series: 0412, 0413, 0414, 0415 …
- The same number must **never** be used twice in the same financial year. Putting the year inside the number (RCP/**26-27**/0413) makes this easy.
- On **1 April** start again (RCP/27-28/0001), or continue, as long as each number is unique for that year.
- You may run **more than one series** (for example counter bills and monthly account bills), but each series must be consecutive.
- **Never tear out, skip or reuse a number.** A missing number looks like a hidden sale. If a bill is spoiled, keep it and mark it "CANCELLED" (6.8).

## 6.4 Types of bills: which one to give

| Situation | Document | GST shown? | Notes |
|---|---|---|---|
| You are **not** GST-registered | **Bill / cash memo** (simple bill) | **No.** You must not charge GST | Still number it and keep a copy. Use it for every booking. |
| Registered, walk-in customer (B2C) | **Tax invoice** (customer details only if ₹50,000+ or requested) | Yes | Most counter bills. |
| Registered, small walk-in bookings **below ₹200** and the customer does **not** want a bill | You may skip a separate invoice per booking, but must issue **one consolidated tax invoice at the end of the day** for all such bookings | Yes | Allowed by section 31(3)(b) and Rule 46 [20]. The bookings are still invoiced and taxed. 🔎 **Ask your CA to confirm.** |
| Registered, business customer (B2B), each booking | **Tax invoice** with customer GSTIN | Yes | |
| Registered, business customer, monthly or weekly billing | **Periodic (consolidated) tax invoice** listing all AWBs (6.5) | Yes | 🔎 **Ask your CA to confirm** the timing rules (time of supply for continuous services). |
| Money received **before** the service (for example a customer pays an advance) | **Receipt voucher** (GST advance receipt) | Usually yes | 🔎 **Ask your CA to confirm.** |
| Money received against an **old bill** | **Receipt / money receipt** (like RV-018 in 3.6) | No (the tax was on the bill) | Not a new sale. |
| Composition scheme taxpayer | **Bill of supply** | No GST charged to customer | 🔎 **Ask your CA** whether composition is available or sensible for you. |

**A "cash memo" from a registered business is still a tax invoice** if it has all the Rule 46 details. Give it whatever name you like, but the content decides what it is.

## 6.5 Sample monthly account invoice (B2B, same state)

```
-------------------------------------------------------------------------------
                         RAJU COURIER POINT                       TAX INVOICE
      Shop 4, Example Road, Pune, Maharashtra 411000 | Ph: 9XXXXXXXXX
      GSTIN: 27XXXXX0000X1Z5 (sample)         State: Maharashtra (27)
-------------------------------------------------------------------------------
Invoice No: RCPA/26-27/031                 Date: 31-10-2026
Bill to:    Patil Electricals Pvt Ltd, Example Industrial Area, Pune
GSTIN:      27YYYYY1111Y1Z2 (sample)        Place of supply: Maharashtra (27)
Period:     01-10-2026 to 31-10-2026        Reverse charge: No
SAC:        996812 (Courier services)
-------------------------------------------------------------------------------
Date    AWB      To         Zone  Chg kg  Freight   FSC 20%  Docket   Amount ₹
02-Oct  5000811  Mumbai      B     2.0    156.00    31.20    20.00    207.20
09-Oct  5000940  Chennai     C     0.5     55.00    11.00    20.00     86.00
14-Oct  5001004  Lucknow     D     1.5    161.00    32.20    20.00    213.20
14-Oct  5001005  Nashik      B     1.0     84.00    16.80    20.00    120.80
22-Oct  5001210  Hyderabad   C     3.0    255.00    51.00    20.00    326.00
-------------------------------------------------------------------------------
                                              Taxable value           953.20
                                              CGST @ 9%                85.79
                                              SGST @ 9%                85.79
                                              Round off                +0.22
                                              INVOICE TOTAL         1,125.00
Amount in words: Rupees One Thousand One Hundred Twenty-Five only
Payment due: within 15 days | Bank: [business current account details]
                                              For Raju Courier Point
                                              (Authorised signatory)
-------------------------------------------------------------------------------
```

Freight check: Mumbai 48 + 3×36 = 156; Chennai 55; Lucknow 65 + 2×48 = 161; Nashik 48 + 36 = 84; Hyderabad 55 + 5×40 = 255. Total 953.20 (4.3). ✔

## 6.6 Sample counter tax invoice (B2C walk-in)

```
-----------------------------------------------------------------
RAJU COURIER POINT  | GSTIN 27XXXXX0000X1Z5 (sample) | TAX INVOICE
Invoice No: RCP/26-27/0413        Date: 14-10-2026  10:40
Customer: Sunita (unregistered)   Place of supply: Maharashtra (27)
SAC 996812 | AWB 5001002 | Pune → Bengaluru | Surface
Actual 3.2 kg | Size 40×30×20 cm = 4.8 kg vol | Chargeable 5.0 kg
Declared value ₹20,000 (carrier's risk)
-----------------------------------------------------------------
Freight (Zone C, 5.0 kg)                               520.00
Fuel surcharge 20%                                     104.00
Docket charge                                           40.00
FOV 0.2% on ₹20,000                                     40.00
Taxable value                                          704.00
CGST 9%                                                 63.36
SGST 9%                                                 63.36
Round off                                               +0.28
TOTAL PAID (UPI, ref …4471)                            831.00
-----------------------------------------------------------------
```

## 6.7 Sample inter-state invoice (IGST)

Kaveri Handicrafts is a **registered business in Bengaluru, Karnataka** (state code 29) and an account customer. For a registered customer, the place of supply is generally the customer's location, so this is **inter-state**: charge **IGST 18%**, not CGST + SGST. 🔎 **Ask your CA to confirm** place of supply for your customer mix.

| AWB | Zone | Chg kg | Working (account rate) | Amount ₹ |
|---|---|---|---|---|
| 5001301 | C | 2.0 | 55 + 3×40 = 175; FSC 35; docket 20 | 230.00 |
| 5001302 | D | 1.0 | 65 + 48 = 113; FSC 22.60; docket 20 | 155.60 |
| | | | **Taxable value** | **385.60** |
| | | | IGST 18% (385.60 × 18% = 69.408) | 69.41 |
| | | | Round off | −0.01 |
| | | | **Invoice total** | **455.00** |

Place of supply: Karnataka (29).

## 6.8 Credit notes and debit notes

You **cannot** simply change an invoice after giving it. You correct it with a separate document that refers to the original bill.

- **Credit note** (*bill kam karna*): reduces an earlier invoice. Used for: wrong weight or rate charged too high, cancelled booking after billing, a discount agreed later.
- **Debit note** (*bill badhana*; a supplementary invoice): increases an earlier invoice. Used for: weight discrepancy passed on, ODA or RTO charges billed later, rate charged too low.

**Credit note example (example numbers):** Desai Pharma's invoice RCPA/26-27/029 charged AWB 5001188 (Zone C) at 3.0 kg, but it should have been 2.5 kg. One account slab (₹40) was charged extra.

| Line | ₹ |
|---|---|
| Freight reduced: 1 slab × ₹40 | 40.00 |
| Fuel surcharge 20% on 40 | 8.00 |
| Taxable value reduced | 48.00 |
| CGST 9% | 4.32 |
| SGST 9% | 4.32 |
| **Credit note RCPCN/26-27/004 total** | **56.64** |

**Debit note example:** if Raju had lost the weight dispute on AWB 5001003 and passed the charge on to Meena (4.5), the debit note would be: extra 3 slabs × ₹40 = ₹120 + FSC ₹24 = ₹144 taxable; CGST ₹12.96 + SGST ₹12.96 = **₹169.92**, numbered in the debit note series (for example RCPDN/26-27/002).

**Rules:**
- Each note has its own number series, date, a reference to the original invoice number and date, the reason, and the GST effect.
- Report notes in the GST return of the month they are issued.
- A **credit note** for a supply must be declared **no later than 30 November after the end of that financial year**, or the date of filing the annual return, whichever is earlier (CGST section 34) [21]. Also, GST can be reduced only if the registered customer reverses the matching input credit. 🔎 **Ask your CA to confirm.**
- Use carrier credit notes in the same way on your purchase side (4.5).

## 6.9 Cancelling or correcting a bill

| Situation | What to do |
|---|---|
| Mistake noticed **before** giving the bill to the customer, same day | Mark that bill **"CANCELLED"** with the reason, keep all copies (or keep it in the software as cancelled), and make a new bill with the next number. **Never delete it or reuse its number.** |
| Bill already given, but not yet in a GST return | Your CA may allow cancelling it and issuing a fresh one, or issuing a credit note. 🔎 **Ask your CA to confirm.** |
| Bill already in a filed GST return | Issue a **credit note** (to reduce or cancel) or a **debit note** (to increase). |
| Customer details wrong (for example a wrong GSTIN) | Correct it before filing the return if possible; after filing, the CA corrects it through the next return (amendment). 🔎 **Ask your CA.** |
| Booking cancelled before the parcel left | Cancel the AWB in the carrier system too, so the carrier does not bill you. |

## 6.10 E-invoicing

**E-invoicing** means registering each B2B invoice on the government's Invoice Registration Portal (IRP), which returns an **IRN** (invoice reference number) and a **QR code** to print on the invoice. It applies once your aggregate turnover has crossed **₹5 crore** in any financial year from 2017-18 onwards. A shop like Raju's (about ₹77 lakh a year) is far below this. 🔎 **Ask your CA to confirm** the current limit.

## 6.11 Simple bill for a shop that is NOT GST-registered

```
-----------------------------------------------------
NEW CITY COURIER (example)          BILL / CASH MEMO
Bill No: NCC/26-27/0057            Date: 14-10-2026
Customer: Ramesh     AWB 5001901   Pune → Pune, 0.5 kg
Courier charges (all inclusive)                 90.00
TOTAL RECEIVED (cash)                           90.00
(Not registered under GST, so no GST charged)
-----------------------------------------------------
```

---

# Part 7: Taxes made simple

## 7.1 GST in three words: charge, collect, pay

1. **Charge:** add 18% GST to every taxable bill (2.2).
2. **Collect:** take it from the customer together with your charges.
3. **Pay:** every month (or quarter), pay the government the GST you collected **minus** the GST you paid on your own business purchases (input tax credit, 7.3).

Courier is **forward charge**: **you** charge GST on your invoice and pay it. A registered customer can then claim it back as input tax credit [2].

## 7.2 Classification, rate and place of supply

- Courier services fall under **SAC heading 9968 (Postal and courier services)**. The specific code for courier services is **996812**. You report 4 or 6 digits on invoices and GSTR-1, depending on your turnover [1].
- Rate: **18%**, split **9% CGST + 9% SGST** within a state, or **18% IGST** between states [1][2]. According to secondary sources, the GST 2.0 rationalisation from 22 September 2025 did not change the courier rate [2]. 🔎 **Ask your CA to confirm** the current notification before invoicing.
- **Place of supply** decides which tax applies (CGST + SGST or IGST). For transport of goods, it is generally the **recipient's location** for a registered recipient; for an unregistered recipient, typically **where the goods are handed over for transport**. 🔎 **Ask your CA to confirm** for your exact cases, especially multi-state account customers.

## 7.3 Input tax credit (ITC): getting back the GST you paid

**Input tax credit** (ITC; GST *ki wapsi*) means: the GST you paid on business purchases is subtracted from the GST you collected.

- GST on **carrier bills** (18%), rent, software, phone, printers and so on can be claimed as ITC, **if** the supplier's invoice shows in your **GSTR-2B** (an automatic statement of purchase invoices filed by your suppliers), you have the invoice, and you have received the service.
- Some ITC is **blocked** (not allowed) under section 17(5): for example most motor vehicles, food and drinks, and personal-use items. 🔎 **Ask your CA to confirm** for vans or bikes used for delivery.
- **Tip:** give your GSTIN to every supplier and check each month that their bills appear in GSTR-2B. If a supplier does not file, you may lose the credit.

## 7.4 Raju's October GST, worked out (example numbers)

Assume all of October's income is taxable at 18% and all within Maharashtra (🔎 **ask your CA to confirm** how GST applies to the franchise commission in your agreement).

| Step | Working | ₹ |
|---|---|---|
| Output GST on income | 18% × 6,43,000 | 1,15,740 |
| ITC on carrier charges (carrier 3,85,000 + RTO 9,500 + weight charges 6,500 = 4,01,000) | 18% × 4,01,000 | (72,180) |
| Other ITC (packing, phone, software, rent, where the supplier charged GST) | example | (4,500) |
| **Net GST payable for October** | 1,15,740 − 72,180 − 4,500 | **39,060** |

That is CGST ₹19,530 + SGST ₹19,530 (39,060 ÷ 2), due with GSTR-3B by 20 November (if filing monthly). This ₹39,060 is the "GST payable" shown on the 31 October balance sheet (5.5).

## 7.5 GST returns and due dates

| Return | What it is | Typical due date (🔎 ask your CA to confirm) |
|---|---|---|
| **GSTR-1** | Details of your sales (B2B invoice by invoice; B2C as a summary) | 11th of next month (monthly), or 13th after the quarter (QRMP) |
| **GSTR-3B** | Summary return + paying the tax | 20th of next month (monthly); 22nd/24th after the quarter for QRMP, with monthly PMT-06 payments |
| **GSTR-9 / 9C** | Annual return / reconciliation | 31 December after the year ends; optional or relaxed for smaller taxpayers (🔎) |

**QRMP** = Quarterly Return, Monthly Payment, a scheme for small taxpayers.

GSTR-1 data comes straight from your sales register, so **every booking, account or cash, must be in the system** for the returns to be right. Late returns attract late fees and interest.

## 7.6 GTA vs courier: do not mix them up

| | **Courier** | **GTA (Goods Transport Agency)** |
|---|---|---|
| What | Door-to-door express carriage of documents and parcels, collected from many senders, against an AWB/docket | Road transport of goods (often truck-load or part-load) against a **consignment note (LR / bilty)** |
| SAC | 9968 (996812) | 9965 |
| GST | 18%, forward charge, ITC available | Default 5% under **reverse charge** (the recipient pays), with no ITC to the GTA; or forward-charge options (5% without ITC / 18% with ITC) [3][4] |

Courier bills must **not** be treated under GTA's 5%/reverse-charge scheme [2][3]. If your own-network business starts doing truck transport with LRs, that part may be GTA. 🔎 **Ask your CA to confirm** before choosing a scheme.

## 7.7 E-way bills (for high-value goods)

- An **e-way bill** is an online document needed to move **goods** worth more than **₹50,000** (consignment value), under CGST **Rule 138** [14][15].
- The main responsibility is the **registered consignor** (or consignee), who generates Part A. A **courier agency may fill Part A with the consignor's authorisation**. If the consignor has not generated it and the transporter carries goods over the limit, the transporter's duties apply [14].
- Ordinary parcels below ₹50,000 generally do not need one [15]. Documents do not need one.
- **Shop rule:** for any parcel with **declared value above ₹50,000**, write the e-way bill number on the booking, or refuse to book without it. 🔎 **Ask your CA to confirm** state-specific limits for movement within the state.

## 7.8 Income tax: the basics

- Your business profit is taxed under income tax. For a proprietor like Raju, the shop's profit is added to his own income and taxed at his personal slab rates. 🔎 **Ask your CA** about rates, deductions and advance tax.
- **Big change:** the **Income-tax Act, 2025** replaced the Income-tax Act, 1961 from **1 April 2026**. "Previous year / assessment year" became a single **"tax year"**. Old section numbers still apply to transactions up to 31 March 2026; new numbers apply from 1 April 2026 [6][12]. The mappings below come from the Income Tax Department and secondary mapping tools. 🔎 **Ask your CA to confirm every new section number.**
- **Advance tax** may be payable in instalments during the year (15 June, 15 September, 15 December, 15 March) if your tax is above the limit. 🔎 **Ask your CA.**

## 7.9 TDS: tax cut from payments

**TDS** (Tax Deducted at Source) means the **payer** cuts a small % from a payment and deposits it with the government in the **receiver's** name.

### TDS that YOU may have to cut (when you pay others)

| Payment | Old (1961 Act) | New (2025 Act) | Rate / limit (as reported) |
|---|---|---|---|
| To contractors, including carriage of goods (your payments to carriers/transporters) | 194C | **393(1), Table Sl. No. 6(i)** | 1% (individual/HUF payee), 2% (others); only if a single payment is above ₹30,000 or the year's total is above ₹1,00,000 [5][6][7] |
| Commission/brokerage (for example if **you** pay commission to sub-agents) | 194H | **393(1), Table Sl. No. 1(ii)** | 2%; ₹20,000 per year limit [19] |
| Rent (shop) | 194-I | Within the section 393 table | 🔎 ask your CA to confirm rate and limit |

- Only a **"designated/specified person"** must deduct: for example companies, firms, and individuals/HUFs whose accounts were audited in the previous year. A small proprietor like Raju (no audit) may not need to deduct. 🔎 **Ask your CA to confirm.**
- From April 2026, TDS returns and challans must quote the **new section/table reference**. Quoting 194C for a payment after April 2026 may fail validation [6].
- There is a special exception for payees in the business of plying/hiring goods carriages who own 10 or fewer vehicles and give a declaration with PAN [5].
- If you deduct TDS: deposit it by the **7th of the next month** and file quarterly TDS returns.

### TDS cut FROM you (when customers pay you)

Corporate account customers may cut 1% or 2% TDS when paying your invoices (Patil example, 4.3). Record it as **TDS receivable**, match it with **Form 26AS / AIS** every quarter, chase customers whose TDS does not appear, and claim it in your income-tax return.

## 7.10 Presumptive tax: a simpler way, but with a big catch

**Presumptive taxation** means: instead of working out exact profit for tax, you declare profit as a fixed % of turnover. It reduces paperwork for tax. It does **not** reduce your duty to record all sales.

- **Old 44AD (business):** an eligible resident individual, HUF or partnership firm (not an LLP) can declare profit at **8% of turnover**, or **6%** on the part received through banking/digital modes, for turnover up to **₹2 crore**, or **₹3 crore** if cash receipts are **5% or less** of turnover [8][9].
- **It does not apply** to people earning **commission or brokerage** or doing **agency business** [9][8b]. So a commission-model courier franchisee may be outside 44AD, while a buy-and-sell agency may be inside. Raju earns some commission too. 🔎 **Ask your CA to confirm. This is important.**
- **44ADA** is for specified professionals (50% of receipts). Normally *not* relevant to a courier business.
- **44AE** is for people plying or hiring goods carriages (fixed income per vehicle). Relevant only if you hire out trucks.
- In the 2025 Act, 44AD/44ADA/44AE are combined in **section 58**; books of account (old 44AA) → **section 62**; tax audit (old 44AB) → **section 63** [12][13]. Tax audit limits were ₹1 crore turnover, or ₹10 crore where cash receipts and cash payments are each 5% or less [9]. 🔎 **Ask your CA to confirm** under the new Act.
- **Example (example numbers):** turnover ₹60,00,000, of which ₹54,00,000 came by bank/UPI and ₹6,00,000 in cash. Presumptive profit = 6% × 54,00,000 + 8% × 6,00,000 = 3,24,000 + 48,000 = **₹3,72,000**. You still need accurate records of turnover, and GST needs full records anyway.

## 7.11 Cash limits (very important for a cash-heavy counter)

| Rule | Old → New (🔎 ask your CA to confirm) | What it says |
|---|---|---|
| **269ST** | 269ST → **section 186** [10][10b] | You must not **receive** ₹2,00,000 or more in cash from one person in a day, for one transaction, or for one event. Penalty on the **receiver** = 100% of the amount (old 271DA) [10c][11]. |
| **40A(3)** | 40A(3) → **section 36** [12] | A business expense paid in cash **above ₹10,000** to one person in a day is **fully disallowed** (₹35,000 for payments to goods-carriage operators) [11]. |
| **269SS / 269T** | (moved within the same chapter; 🔎) | Do not accept or repay loans/deposits of **₹20,000 or more** in cash. Penalty = 100% [11]. |

**In practice:** a big account customer must not settle ₹2 lakh or more in cash; pay carriers, rent and salaries above the limits by bank/UPI; and use cash only for genuine small walk-in bookings and petty expenses.

## 7.12 What to give your CA, and when

| When | Give the CA | Why |
|---|---|---|
| **Every month, by about the 5th** (or as your CA asks) | Sales register (all invoices: counter + account), list of B2B invoices with customer GSTINs, credit and debit notes, cancelled-bill list | GSTR-1 |
| | Purchase register with all carrier bills, rent, phone, software and other GST bills | ITC and GSTR-3B |
| | Bank statements, cash book, COD register summary | Checking and reconciliation |
| | Payments to carriers/contractors and commission paid (if you must deduct TDS) | TDS by the 7th |
| **Every quarter** | TDS certificates received from customers; your own 26AS/AIS check | TDS credit; TDS returns |
| | Estimate of the year's profit | Advance tax |
| **Every year (after 31 March)** | Full books: day book, cash book, bank book, all ledgers, aging lists, fixed asset bills (scale, computer, bike), loan statements, franchise agreement and deposit receipts, list of drawings, closing stock of packing material, salary register | Income-tax return, GSTR-9, tax audit if applicable |
| **Whenever it happens** | New franchise/carrier agreement, a vehicle purchase, a new staff hire, any tax notice, a customer paying a big amount in cash | So the CA can advise before mistakes happen |

---

# Part 8: Honest records: why every booking must be recorded

## 8.1 "White" money and "black" money

In everyday talk, **"white" money** (*safed paisa*) is income that is recorded in the books and taxed. **"Black" money** (*kaala dhan*) is income that is hidden from the books and from the tax authorities.

**Plainly:** recording only some cash bookings, or deleting or leaving out some bookings to show lower income, is **tax evasion** under both GST and income-tax law. It can lead to a tax demand on the hidden turnover, interest, penalties (which can be a large multiple of the tax), and in serious cases **prosecution** (a criminal case). Under GST, collecting GST from a customer and not paying it to the government is treated especially seriously.

## 8.2 Why cash bookings must be recorded too

1. **It is a taxable sale.** GST is due on every courier booking, whatever the payment mode. The B2C invoice rules (6.4) exist exactly because cash sales must be billed.
2. **Your carrier's records exist anyway.** Every AWB you book is in the carrier's system, in its GST returns, and on your carrier bill. Your carrier costs (and ITC) show parcel volumes that your sales must logically match. Gaps are easy to see when purchases do not match sales.
3. **Digital trails.** UPI, bank deposits, AIS/26AS data, GST analytics and franchisor reports all leave traces.
4. **Your own business suffers.** Unrecorded cash is where theft and leakage hide. You cannot see your real margins, cannot get a bank loan or overdraft on low declared profits, and cannot sell or value the business properly.
5. **Presumptive and audit limits depend on true figures**, for example the 5% cash-receipt test (7.10).

## 8.3 Penalties to remember (summary)

| Wrong action | Possible result |
|---|---|
| Hiding sales / not issuing bills | Tax on hidden turnover, interest, penalties, possible prosecution |
| Collecting GST and not paying it | Treated very seriously under GST; interest and penalties |
| Receiving ₹2 lakh or more in cash from one person in a day | Penalty equal to 100% of the amount (7.11) |
| Paying business expenses in cash above ₹10,000 to one person in a day | The expense is disallowed for income tax (7.11) |
| Taking or repaying loans/deposits of ₹20,000 or more in cash | Penalty equal to 100% (7.11) |
| Not keeping records for the required years | Penalties, and you cannot defend yourself in an inquiry (9.3) |

🔎 **Ask your CA to confirm** the exact penalty sections under the new Income-tax Act.

## 8.4 Lawful things that people confuse with hiding income

- **Cash vs account bookings** (2.7) are only different *payment terms*. Both are fully recorded.
- **Presumptive taxation** (7.10) is a *lawful* way to work out tax on declared turnover. It reduces paperwork, not the duty to declare all turnover.
- **Cash limits** (7.11) control *how* large amounts move. Staying within them is following the law, not hiding anything.
- **Legitimate tax planning:** claiming all genuine expenses and ITC, choosing the right business structure, and using permitted deductions, with a CA's advice.

## 8.5 Habits that keep a shop honest and safe

- A bill for **every** booking, at the moment of booking.
- One numbered bill series per type, no gaps, no reuse.
- Daily cash count, with differences written as "cash short/excess".
- All UPI to the business account, never to a personal number.
- Monthly match of your bookings with the carrier's bill (counts and AWBs).
- Never delete. Cancel with a reason, and keep the record.

This guide deliberately gives **no** guidance on keeping parallel or unrecorded books. For Kumkaj, a good design choice is to make every booking, cash or account, land in the same ledger, with fixed invoice numbering and an audit trail for edits and deletions.

---

# Part 9: Mistakes, checklists, record keeping and numbers to watch

## 9.1 Common mistakes and how to fix them

| Mistake | What goes wrong | Fix |
|---|---|---|
| Using only actual weight, forgetting volumetric | Carrier bills more than you charged; you lose money | Measure every parcel; use the formula (2.4); photo evidence |
| Not rounding up to the slab | Undercharging | Always round **up** to the next slab |
| Old rate card / old fuel surcharge | Wrong prices | Date the rate card; update when the carrier changes rates |
| COD money counted as income | Profit looks huge; you spend money that belongs to sellers | Keep a COD register and a "COD payable" account (4.1) |
| Paying sellers before the carrier remits | Your own cash is stuck | Pay only against the carrier's remittance file, by AWB |
| Not checking carrier bills | Paying for wrong weights, duplicates, wrong zones | Line-by-line check against your records (4.4) |
| Missing the dispute window | Extra weight charges cannot be challenged | Check carrier bills and weight reports every week |
| Forgetting to bill RTO or ODA | Lost income | Have a "charges to bill later" list; use debit notes |
| Writing UPI receipts in the cash book | Cash book never matches the drawer | Cash in cash book; UPI/card in bank book |
| Mixing home and shop money | Cannot see real profit | Separate bank account; write drawings (5.2) |
| Skipping bill numbers or deleting bills | Looks like hidden sales; GST mismatch | Cancel with reason; never delete (6.9) |
| GST on wrong type (CGST/SGST vs IGST) | Customer cannot take credit; corrections needed | Check the place of supply (7.2) |
| No bill for expenses | Expense and ITC may be lost | Voucher and supplier bill for everything |
| Large cash payments or receipts | Disallowance or 100% penalty | Use bank/UPI above the limits (7.11) |
| Not tracking TDS cut by customers | Tax paid twice | Record TDS receivable; check 26AS/AIS quarterly |
| Credit given too freely | Bad debts | Credit limits; aging list every month (4.3) |
| No backup | All records lost if the phone/computer breaks | Daily backup; keep paper files in order |

## 9.2 Checklists

### Daily

- [ ] Opening cash counted and matched with last night's closing
- [ ] Petty cash checked (cash + vouchers = float)
- [ ] Rate card and fuel surcharge current
- [ ] Every booking: weight, size, photo, bill, entered at once
- [ ] E-way bill taken for any parcel above ₹50,000
- [ ] Manifest made, counted, signed; copy filed
- [ ] NDR/RTO follow-up done
- [ ] Cash counted by denomination; difference recorded as cash short/excess
- [ ] UPI/card total matched with the payment app
- [ ] Extra cash deposited in bank
- [ ] Backup taken

### Weekly

- [ ] COD reconciliation by AWB for each carrier and each seller (4.1)
- [ ] Sellers paid only for remitted COD
- [ ] Weight discrepancy reports checked; disputes raised within the window
- [ ] Petty cash topped up by the voucher total
- [ ] Payment reminders sent to account customers
- [ ] Carrier payables due this week planned

### Monthly

- [ ] Monthly invoices and statements sent to account customers
- [ ] Carrier bills checked line by line and entered in the purchase register
- [ ] Bank reconciliation done (4.6)
- [ ] Wallet statement matched with bookings
- [ ] Salaries paid and entered; salary register signed
- [ ] Receivables and payables aging prepared
- [ ] P&L prepared; margins and leakages reviewed
- [ ] Documents sent to the CA (7.12)
- [ ] TDS deposited by the 7th (if you are a deductor)
- [ ] GSTR-1 by the 11th (monthly) or 13th after the quarter (QRMP)
- [ ] GSTR-3B and GST payment by the 20th (monthly) or 22nd/24th after the quarter (QRMP), with PMT-06 in the other months

### Quarterly

- [ ] TDS returns filed (if you are a deductor)
- [ ] TDS credits checked in 26AS/AIS
- [ ] Advance tax instalments (15 June, 15 September, 15 December, 15 March) if applicable

### Yearly

- [ ] Year-end closing: all ledgers balanced; balance sheet prepared
- [ ] New invoice series for the new financial year from 1 April
- [ ] Income-tax return; GSTR-9 if applicable; tax audit if applicable
- [ ] Credit notes for last year's supplies declared by 30 November (6.8)
- [ ] Renew insurance and the franchise/carrier agreements; review rate cards
- [ ] Old records archived safely (9.3)

🔎 **Ask your CA to confirm the due dates each year.**

## 9.3 How long to keep records

- **GST:** keep books and records for **72 months (6 years) from the due date of the annual return** for that year. Longer if any appeal or proceeding is pending (until one year after it is finally decided) [16][17].
- **Income tax:** books and documents must generally be kept for several years after the end of the relevant year, and longer if a reassessment is opened (old Rule 6F / new Income-tax Rules 2026) [4b]. 🔎 **Ask your CA to confirm** the period under the new rules.
- **Simple safe practice:** keep **at least 8 years** of all books, invoices, carrier bills, COD remittance files, bank statements, manifests and dispute evidence, with a digital backup.
- Records can be electronic. Make sure they can be read, are backed up, and that edits can be traced.

## 9.4 Numbers to watch (KPIs)

A **KPI** (key performance indicator) is a number that tells you quickly how the business is doing.

| KPI | Formula | Why it matters |
|---|---|---|
| Shipments per day | count | Volume trend |
| Revenue per shipment | Income ÷ shipments | Pricing power |
| **Margin per shipment** | (Customer charge − carrier cost) ÷ shipments | Core earnings per parcel (Raju's example day: 769.20 ÷ 7 = ₹109.89) |
| Gross margin % | Gross profit ÷ income | Health of rate cards (Raju's October: 36.4%) |
| Revenue per kg | Income ÷ chargeable kg | Compare across carriers/customers (example day: 2,026 ÷ 14.0 = ₹144.71) |
| Account vs cash mix | Account income ÷ total | Credit risk vs cash handling |
| **DSO (days sales outstanding)** | Receivables ÷ (credit sales per day) | How fast customers pay |
| Receivables older than 60 days % | | Bad-debt risk |
| **RTO %** | RTO shipments ÷ shipped | Big hidden cost, especially for COD |
| NDR % and NDR-to-delivered conversion | | Last-mile quality |
| COD delivered but not remitted (₹ and days) | | Cash stuck with carriers |
| Weight-discrepancy cost % | Discrepancy charges ÷ carrier cost | Booking discipline |
| Carrier mix & cost per kg per carrier | | Negotiation |
| On-time delivery % (vs TAT) | | Customer retention |
| Cash shortage/excess per month | | Counter control |
| Customer concentration | Top 5 customers ÷ income | Dependency risk |

**DSO in words:** if you sell ₹10,000 a day on credit and customers owe you ₹3,00,000, DSO = 3,00,000 ÷ 10,000 = **30 days** (example numbers). That means customers take about a month to pay. Lower is better.

---

# Part 10: Developer notes: how this maps to Kumkaj Courier

*This part is for the Kumkaj developer. Shop owners can skip it.*  
*Product brief + competitor feature matrix: [`../KUMKAJ_COURIER.md`](../KUMKAJ_COURIER.md).*

## 10.1 Kumkaj Courier concepts used in this guide

| Kumkaj concept | Courier meaning |
|---|---|
| **Carrier** (party, money **out**) | The courier company you book through: Delhivery, DTDC, Blue Dart, Ekart, Xpressbees, Shree Maruti, Professional, India Post (Speed Post) etc. You pay them (or they settle with you). |
| **Consignor** (party, money **in**) | Your customer who sends the parcel: a shop, an e-commerce seller, an office, a walk-in person. They pay you. |
| **Item, unit = kg** | A rate line, e.g. "Delhivery Surface – Zone B". Quantity = chargeable kg, rate = ₹ per kg / per slab. |
| **Tag: Account booking** | A credit customer (monthly billing, shareable invoice, ledger, outstanding). |
| **Tag: Cash booking** | A walk-in/spot customer who pays immediately (cash/UPI/card). |

## 10.2 Suggested data model

| Need | Suggested Kumkaj modelling |
|---|---|
| Carriers | Party type **Carrier** (payable). Store GSTIN, carrier account code, settlement style (bill / wallet / commission), volumetric divisor, dispute window. |
| Consignors | Party type **Consignor** (receivable). Store GSTIN (B2B), state code (for place of supply), credit limit, credit days, billing cycle (weekly/monthly), TDS deduction %, default rate card, COD payout bank details. |
| Rates | **Items with unit kg**, one per carrier × service × zone, with fields: first-slab weight, first-slab price, additional-slab weight, additional-slab price, min chargeable kg. Keep a **buy rate** (carrier) and **sell rate(s)** (counter/customer-specific). Version rate cards with an effective date (FSC changes). |
| Bill line | AWB number (unique, indexed), carrier, service, origin/destination pincode, zone, pieces, actual kg, L×B×H, volumetric kg, **chargeable kg** (computed: max(actual, volumetric) rounded up to slab), declared value, freight, FSC %, docket, FOV, COD charge, ODA, RTO, taxable value, GST split, payment type (Paid / To-Pay / TBB / COD), e-way bill no. (mandatory when declared value > ₹50,000), photo attachments (scale + tape). |
| Booking tag | **Account booking** → credit invoice, appears in ledger & aging, shareable invoice link; **Cash booking** → immediate receipt with payment mode (cash / UPI / card). Payment *mode* kept separate from booking *tag*. |
| TBB accumulation | Account bookings accumulate unbilled per consignor; "Generate periodic invoice" pulls all unbilled AWBs for the period into one Rule-46-compliant invoice. |
| Margin | Per line: sell − buy cost; roll up by carrier, customer, zone, day. |
| COD | Separate COD register / liability account per consignor; statuses: booked → delivered → remitted by carrier → paid to consignor (or RTO). Reconcile by AWB against carrier remittance CSV import. Support netting with both the invoice and the COD payout visible. |
| Carrier bill import | Import carrier invoice CSV, auto-match by AWB, flag weight/rate differences → dispute workflow (with deadline from dispute window) → credit note. Optional debit note to consignor. |
| Receipts | Allow split: bank amount + TDS deducted + discount/short. |
| Cash closing | Daily screen: opening, cash bookings, cash receipts, cash expenses, deposits, counted denominations, difference posted to "Cash short/excess". |
| Petty cash | Imprest float, vouchers with expense head and attachment, top-up = voucher total. |
| Rounding | Round-off line per invoice (nearest rupee), posted to a Round-off account; tax amounts kept exact. |
| Tax | GST summary for GSTR-1 (B2B invoice-wise, B2C summary) and 3B; SAC 9968/996812 default on courier items; HSN/SAC summary; CGST+SGST vs IGST chosen from supplier state vs place of supply. |
| Numbering | Multiple series (counter invoice, periodic invoice, credit note, debit note, receipt voucher), each ≤16 characters, consecutive, unique per FY, auto-reset or continue at 1 April. |
| Integrity | Sequential invoice numbers per FY, no hard delete (cancel with reason), edit audit log. This supports compliance and discourages unrecorded bookings. |
| Reports | Day book, cash book, bank book (with reconciliation), party ledgers, receivables/payables aging, COD register, P&L, balance sheet, KPI dashboard (9.4). |

---

# Part 11: Glossary

Simple meanings, with everyday Hindi/Hinglish words where helpful.

| Term | Meaning |
|---|---|
| Account booking | Credit booking for a regular customer, billed later (TBB). *Udhaar / khata booking.* |
| Accrual basis | Counting income when earned and expenses when incurred, not only when cash moves. |
| Advance tax | Income tax paid in instalments during the year. |
| Aging | Grouping unpaid amounts by how old they are (0–30, 31–60… days). *Kitne din se baaki.* |
| AIS / 26AS | Income-tax statements showing TDS and financial transactions reported against your PAN. |
| Asset | Something the business owns or is owed: cash, bank, equipment, receivables, deposits. *Sampatti.* |
| AWB / Docket / Waybill / CN | Unique shipment tracking number/document, like a train PNR. *Parchi number.* |
| B2B / B2C | Business to business (customer has GSTIN) / business to consumer (walk-in person). |
| Bad debt | Money a customer will never pay, written off as an expense. *Doobi hui rakam.* |
| Balance sheet | A "photo" on one date of assets, liabilities and capital. |
| Bank book | Record of all bank transactions, including UPI and card. |
| Bank reconciliation | Matching your bank book with the bank statement and explaining differences. |
| Bill of supply | Bill without GST, used by composition taxpayers or for exempt supplies. |
| Capital | The owner's share in the business: assets minus liabilities. *Poonji.* |
| Carrier | The courier company that actually carries the parcel. You pay it. |
| Cash book | Record of all physical cash transactions. *Rokad bahi.* |
| Cash booking | Booking paid on the spot (cash, UPI or card). *Nakad.* |
| Cash memo | A simple bill. For a registered business it must have tax-invoice details. |
| Cash short/excess | Account where daily cash differences are recorded honestly. |
| CGST / SGST / IGST | Central GST and State GST (within a state, 9% + 9% for courier) / Integrated GST (between states, 18%). |
| Chargeable weight | Higher of actual and volumetric weight, rounded up to the slab. |
| COD | Cash on Delivery: goods value collected from the consignee and passed on to the consignor. |
| COD remittance | Carrier's payout of collected COD amounts. |
| Consignee | Receiver of the shipment. *Paane wala.* |
| Consignor | Sender of the shipment; your customer. *Bhejne wala.* |
| Consignment | The goods/parcel sent under one booking. |
| Contra entry | Transfer between cash and bank (e.g., cash deposit). Not income or expense. |
| Credit (Cr) | The right side of an entry: income, what goes out, whom you owe. |
| Credit note / Debit note | Document reducing / increasing an earlier invoice value. |
| Creditor / Payable | Someone you owe money to (e.g., a carrier). *Dena.* |
| Day book | Time-ordered record of all transactions. *Rojmel.* |
| Debit (Dr) | The left side of an entry: what comes in, who owes you, expenses. |
| Debtor / Receivable | Someone who owes you money (e.g., an account customer). *Lena.* |
| Denomination count | Counting cash note by note (₹500 × 1, ₹200 × 2 …). |
| Depreciation | Spreading the cost of equipment over its useful life. *Ghisaai.* |
| Docket charge | Fixed per-AWB paperwork fee. |
| Double entry | Every transaction written on two sides (debit and credit) with equal amounts. |
| Drawings | Money the owner takes for personal use. Not an expense. *Maalik ne nikala.* |
| DSO | Days Sales Outstanding: average days to collect receivables. |
| E-invoice | GST invoice registered on the IRP with an IRN/QR code (above ₹5 crore turnover). |
| E-way bill | Electronic document for moving goods worth above ₹50,000. |
| Financial year (FY) | 1 April to 31 March. |
| First mile | Pickup to origin hub. |
| FOV / Freight on Value / Insurance | Value-based liability/insurance charge on declared value. |
| Franchise commission | Share of booking value paid by the franchisor to the franchisee. |
| FSC (Fuel surcharge) | % added to freight for fuel cost changes. |
| Gross profit | Income minus direct costs (carrier charges etc.). |
| GST | Goods and Services Tax; 18% on courier services. |
| GSTIN | 15-character GST registration number. |
| GSTR-1 / GSTR-3B / GSTR-9 | GST sales return / summary return & payment / annual return. |
| GSTR-2B | Auto-generated statement of ITC available from suppliers' filings. |
| GTA | Goods Transport Agency: road transporter issuing consignment notes (SAC 9965). |
| Hub | Sorting/collection centre. |
| Imprest | Petty cash system with a fixed float topped up by the voucher total. |
| Input GST / ITC | Input Tax Credit: GST paid on purchases, set off against GST on sales. *GST ki wapsi.* |
| Invoice / Tax invoice | Numbered bill showing the service, amount and GST. *Pakka bill.* |
| Journal entry | One transaction written with its debit and credit sides. |
| Last mile | Destination hub to doorstep. |
| Ledger | Account-wise record (per party or per income/expense head). *Khata.* |
| Liability | Something the business owes: carrier dues, GST payable, COD payable, loans. *Dena.* |
| Linehaul | Long-distance hub-to-hub movement. |
| Manifest | List of shipments handed over to a carrier in a batch. *Challan / list.* |
| Margin per shipment | Customer charge minus carrier cost for one shipment. *Munafa.* |
| NDR | Non-Delivery Report: failed delivery attempt. |
| Net profit | Gross profit minus operating expenses. |
| ODA | Out of Delivery Area: remote location with extra charge. |
| Output GST | GST collected on your sales, owed to the government. |
| P&L | Profit & Loss statement: income minus expenses for a period. *Nafa-nuksaan.* |
| PAN | Permanent Account Number for income tax. |
| Petty cash | Small cash float for minor expenses (imprest). *Chhota kharcha.* |
| Place of supply | GST rule deciding whether IGST or CGST+SGST applies. |
| POD | Proof of Delivery. |
| Presumptive taxation | Declaring profit at a fixed % of turnover (old 44AD/44ADA/44AE → new s.58). |
| Proprietor | Single owner of a business. |
| QRMP | Quarterly Return Monthly Payment GST scheme for small taxpayers. |
| Rate card | Price table by service × zone × weight slab. *Bhaav ki list.* |
| RCM | Reverse Charge Mechanism: the recipient pays GST (applies to GTA, not courier). |
| Receipt voucher | Slip given when money is received. |
| Reconciliation | Matching two records that should agree. *Milaan.* |
| Round off | Small difference when a bill is rounded to the nearest rupee. |
| RTO | Return to Origin. |
| SAC | Services Accounting Code (courier: 9968 / 996812). |
| Slab | Weight step used for pricing (e.g., each 0.5 kg). |
| TAT | Turn-around (delivery) time. |
| TBB | To Be Billed: credit billing to the consignor. |
| TDS | Tax Deducted at Source (old 194C/194H → new s.393 table entries). |
| To-Pay | Freight collected from the consignee at delivery. |
| Trial balance | List of all ledger balances; debits must equal credits. |
| Turnover | Total sales in a period. |
| UTR | Unique reference number of a bank transfer (NEFT/RTGS/IMPS). |
| Volumetric weight | L × B × H ÷ divisor (often 5000). |
| Voucher | Small document proving a payment or receipt. |
| Weight discrepancy | Carrier's re-measured weight higher than declared, causing an extra charge. |
| Zone | Destination group used for pricing. |

---

# Part 12: Sources and items to confirm with a CA

## 12.1 Sources

Links accessed October 2026. Secondary sources (tax portals and blogs) are useful summaries but **not** authoritative. Confirm with CBIC, the Income Tax Department, or your CA.

1. SAC 996812 courier services, 18% (IncorpX): https://www.incorpx.io/sac-codes/code-996812
2. GST on courier services, courier vs GTA, ITC (TaxClue): https://taxclue.in/guide/gst-on-courier
3. GST on freight & GTA (5% RCM / forward charge) (TaxClue): https://taxclue.in/guide/gst-on-freight
4. GST rate for courier & goods transport (IndiaFilings): https://www.indiafilings.com/learn/gst-rate-for-courier-and-goods-transport-services
   4b. Books of account: Rule 46 (2026 Rules) vs Rule 6F (TaxHeal): https://www.taxheal.com/key-difference-for-books-of-accounts-maintenance-in-new-income-tax-act-2025-rule-46-vs-rule-6f.html
5. Section 393, Income-tax Act 2025 (TDS table) (Income Tax Department): https://www.incometaxindia.gov.in/w/section-393-6
6. Tax payments help (new section codes from 1 Apr 2026; 194C → 393(1) Table Sl. No. 6(i)) (Income Tax e-filing portal): https://www.incometax.gov.in/iec/foportal/help/all-topics/e-filing-services/tax-payments
7. TDS payment under the new Act (TaxGuru): https://taxguru.in/income-tax/making-tds-payment-income-tax-act-2025.html ; TaxHeal: https://www.taxheal.com/tds-on-contractors-w-e-f-01-4-2026.html
8. Section 44AD text (Income Tax Department): https://www.incometaxindia.gov.in/w/section-44ad-9 ; 8b. India Code (44AD incl. exclusions): https://indiacode.ecourtsindia.com/income-tax-1961-to-2025/44AD/
9. 44AD/44ADA explained (₹2 cr/₹3 cr, 8%/6%, exclusions, 44AB limits) (TaxSphere): https://taxnotice.vittsphere.com/caselaw/concept/presumptive-taxation-44ad-44ada/
10. Section 269ST (and correspondence to s.186 of the 2025 Act) (India Code): https://indiacode.ecourtsindia.com/income-tax-1961-to-2025/269ST/ ; 10b. Nexus crosswalk: https://nexusca.ai/crosswalk/1961/269st ; 10c. Section 271DA penalty: https://indiacode.ecourtsindia.com/income-tax-act/section/271DA/
11. Cash transaction limits (40A(3), 269SS, 269ST, 269T) (TaxGuru): https://taxguru.in/income-tax/prohibited-transaction-cash-limit-cash-transactions.html
12. Old vs new section mapping (40A(3)→36, 44AD→58, 44AA→62, 44AB→63) (AskCASanjay): https://askcasanjay.com/blog/income-tax-act-2025-glossary
13. Section mapper (Lexvio): https://www.lexvio.ai/tools/income-tax-section-mapper
14. CGST Rule 138 (e-way bill) (CBIC): https://taxinformation.cbic.gov.in/content/html/tax_repository/gst/rules/cgst_rules/active/chapter16/rule138_v1.00.html
15. E-way bill flyer (GST Council): https://gstcouncil.gov.in/sites/default/files/e-version-gst-flyers/51_GST_Flyer_Chapter18.pdf
16. CGST Act section 36 (retention 72 months) (CBIC): https://taxinformation.cbic.gov.in/content/html/tax_repository/gst/acts/2017_CGST_act/active/chapter8/section36_v1.00.html
17. CGST Rule 56 (accounts and records) (CBIC): https://taxinformation.cbic.gov.in/content/html/tax_repository/gst/rules/cgst_rules/active/chapter7/rule56_v1.00.html
18. Volumetric weight calculation (÷5000) (Delhivery Help): https://help.delhivery.com/docs/calculating-volumetric-weight
19. 194H → s.393(1) Table Sl. No. 1(ii) (Audcrix): https://audcrix.com/income-tax-act-2025/section-194h
20. CGST Rule 46 (tax invoice contents; 16-character serial number unique per FY; details for unregistered recipients at ₹50,000; end-of-day consolidated invoice proviso) (CBIC): https://taxinformation.cbic.gov.in/content/html/tax_repository/gst/rules/cgst_rules/active/chapter6/rule46_v1.00.html
21. CGST Act section 34 (credit and debit notes; credit note declaration by 30 November following the FY or annual return date, whichever is earlier) (CBIC): https://taxinformation.cbic.gov.in/content/html/tax_repository/gst/acts/2017_CGST_act/active/chapter7/section34_v1.00.html

## 12.2 Items to confirm with a CA (summary)

1. Who is the GST supplier in your franchise agreement (you or the franchisor), and GST on your commission.
2. Place of supply / IGST vs CGST+SGST for your customer mix.
3. GST registration threshold, QRMP eligibility, e-invoicing applicability, GSTR-9 requirement.
4. Whether any part of your business is GTA rather than courier.
5. Blocked ITC on delivery vehicles.
6. Whether you are a "designated person" required to deduct TDS; new-Act section/table codes for contractor, commission and rent.
7. Presumptive taxation (new s.58) eligibility, especially the commission/agency exclusion.
8. New-Act section numbers and penalties for cash limits (s.186, s.36, loan/deposit rules).
9. Record-retention period under the Income-tax Rules, 2026.
10. E-way bill responsibility and state-specific limits for high-value parcels.
11. GST invoicing for To-Pay parcels with your franchisor.
12. Periodic (weekly/monthly) invoice timing for account customers, and the end-of-day consolidated invoice for small B2C bookings.
13. Credit note time limits and conditions; how to correct invoices already reported.
14. Your rounding method on invoices; GST receipt vouchers for advances.
15. Depreciation rates; PF/ESI/professional tax/shop licence rules for your state and staff size; composition scheme suitability.
