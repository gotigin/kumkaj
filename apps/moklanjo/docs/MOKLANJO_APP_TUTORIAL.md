# Moklanjo — learn the courier business by running the app

Learn enough courier **business** and Moklanjo **practice** to run a counter
and demo the product offline (one Server, then LAN if you need a second till).
Deeper tax and month-of-books examples:
[`grokbot/MOKLANJO_BUSINESS_TUTORIAL.md`](grokbot/MOKLANJO_BUSINESS_TUTORIAL.md).
Product brief: [`MOKLANJO.md`](MOKLANJO.md). Books vocabulary:
[`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md).

**Order**

1. Parts 0–1 — idea and first open  
2. Parts 2–3 — parties, rates, then the booking desk with people in line  
3. Parts 4–7 — Day book, corrections, Pay out / Reports, Ops  
4. Parts 8–9 — talking points and demo script  
5. Part 10 — checklist  

Use practice names only. Owners type their own party names; do not seed sample
party lists into a demo build. Solidify on Linux before Windows packaging.

Field names below match the English chrome of the shipped app (tabs, empty
text, buttons). Do not invent a second path.

------------------------------------------------------------------------

## Part 0 — What kind of business this is

A courier **agency / franchise counter** is a travel agent for parcels:

1. People bring parcels (or you pick them up).
2. You weigh, price, take money or put it on account, give a receipt / AWB.
3. You hand bags to **carriers** and/or send them with **your own riders**.
4. Your edge is roughly: what the customer paid you − what the carrier / rider
   cost you. Rent and salaries come out of that.

| Role | Meaning | Money |
|------|---------|-------|
| **Consignor** | Sender — your customer | Money in (they pay you / owe you) |
| **Carrier** | Who moves the bags | Money out (you pay them / owe them) |
| **Consignee** | Receiver in the other city | Usually not your customer (except COD / to-pay) |

Moklanjo is books **and** booking on one device, offline-first. Learn one
Server before pairing extra tills on office Wi‑Fi.

------------------------------------------------------------------------

## Part 1 — First open

1. Launch Moklanjo → language → **Server Mode** → create a server → set an
   **Owner PIN** (there is no default).
2. You land on **Day book**. ₹0 is correct until you book. Full reading of
   this screen is in Part 4, after you have bookings.
3. Tabs you will use first (scroll the bar / rail; drag a tab onto another to
   rearrange — order is saved in the books):

| Tab | Role |
|-----|------|
| **Day book** | Today’s position |
| **Booking** | Counter desk |
| **Pay out** | Goods received, expense vouchers, carrier-style AP |
| **Rates & services** | Rate cards (name, price/unit, slab, zone, …) |
| **Parties** | Consignors and carriers |
| **Bookings** | History, amend, void, refund |
| **Reports** | Opens as **Bank book & P&L** — day / month / outstanding / bank |
| **Ops** | Find / Scan AWB, label, status, dispatch |
| **Lock** | Lock the desk |

Also on the bar when you need them: **COD**, **Riders**, **Day close**,
**Team**, **Settings**, and others (full list in Part 11).

------------------------------------------------------------------------

## Part 2 — Parties and rates (before the desk works)

Booking needs at least one rate. Named customers and account / credit need
parties. Set those up before the rush.

### 2.1 Parties

Open **Parties**. One list for consignors, carriers, suppliers, or **Both**.
Debtor balance (they owe us) and creditor balance (we owe them) stay separate
on the same party.

**Form (always visible, top → bottom):**

| Control | Label / empty text |
|---------|-------------------|
| Name | `Party name` |
| Phone | `Phone` |
| Address | `Address` |
| Logo | Hint (e.g. initials) · **Clear logo** · **Pick logo** |
| Kind heading | **Account kind (tap one)** |
| Kind chips | **Consignor** · **Carrier** · **Both** · **Supplier** |
| Custom role | `Custom role (optional) — or tap a chip above` |
| Primary | **Add party** (no row selected) → **Save changes** after **Pick** |
| Always | **New party (clear form)** |
| After Pick | **Delete selected party** |
| List | **Saved parties** / `Saved parties (N)` · each row **Pick** |

**After Pick only** (`party_detail`):

| Block | Controls |
|-------|----------|
| Custom fields | Field name · value · **Add / update field** · **Remove field** |
| Credit | Credit limit · **Set credit limit** · **Period bill** |
| They pay us (AR) | `Amount received` · **Settle cash** · **Pay UPI** (digital name) · **Settle card** · `Against bill #…` · write-off |
| We pay them (AP) | Pay AP cash / UPI / card · write-off AP |
| Notes | Notes & attachments |

**Practice:** add two carriers and a few consignors with **Add party**. Pick one
consignor → **Set credit limit** so account booking can refuse over-limit later.
Optional: add a custom field after Pick.

### 2.2 Rates & services

Open **Rates & services**.

| Chrome | Text |
|--------|------|
| Heading | **Your rates** |
| Help | Each row is a rate (e.g. Surface Zone A). Unit is usually kg. Tap to edit. |
| Mode | **Adding a new rate** · or **Editing:** + name |
| While editing | **New rate** clears the form to add another |

**Form (top → bottom):**

| Control | Empty text / button | Use |
|---------|---------------------|-----|
| Name | `Item name — type it in your language` | Name of this rate card |
| Aliases | `Aliases — other names, comma separated` | Optional |
| Price · Unit | Price placeholder `0` · `Unit (kg or blank)` | Price = flat ₹/kg (or piece) when slab fields are empty. For courier slabs set Unit to `kg`; Price may stay `0` |
| First kg · First amount | `First kg (slab)` · `First amount` | First weight in the first charge, and that charge. When First kg > 0 and First amount or Extra / kg is set, Booking uses the slab instead of flat Price |
| Extra / kg · Zone name | `Extra / kg` · `Zone name` | Per kg after the first slab. Zone name creates the zone if new — type the **same** name in Booking’s Zone field |
| Carrier · Party | `Carrier (blank = our sell rate)` · `Party (blank = all customers)` | Optional. Party / carrier must already exist on **Parties** |
| Primary | **Add rate** / **Save** | Create or save edits |
| Tax | `Tax rate name or % — Standard, Reduced, 0, 5, 12, 18, 20` | Optional |
| Category | `Category (optional, created if new)` | Optional |
| Options | `Options (optional) — Large=20, Gift wrap=10` | Optional |
| Photo | **Photo** · **Clear photo** | Optional |

List under the form. Empty: *No rates yet. Open Rates & services…* (Booking
shows the same idea when the catalogue is empty).

**Minimum practice:**

1. Name e.g. `Surface Local` · Unit `kg` · Price `0` if using the slab.  
2. First kg `1` · First amount (practice) · Extra / kg (practice).  
3. Zone name e.g. `Local` — remember it for Booking.  
4. Carrier and Party blank · **Add rate**.

------------------------------------------------------------------------

## Part 3 — Working the counter with people in line

Several people can stand at the desk. Moklanjo handles that as **one finished
booking at a time**, then a clean desk for the next person.

### 3.1 How entries get onto a booking

```text
Fill destination, zone, weight (and customer if account / credit)
  → tap ADD on rate card(s)     ← lines enter the cart
  → Account booking or Cash booking (bill lane)
  → type Cash / UPI / Card / Credit amounts (must add up to the bill)
  → Take payment
  → note AWB → Next booking     ← clear desk for the next person
```

| Situation | What you do |
|-----------|-------------|
| One person, one parcel | One **ADD**, then pay with **Take payment** |
| One person, several charge lines | Several **ADD**s on the **same** cart, then **Take payment** |
| Several people in line | Finish person 1 → **Next booking** → person 2 |
| Named account / credit | Customer on Active ledger · **Account booking** · put the bill (or part) in **Credit / khata amount** · **Take payment** |

### 3.2 Booking layout (as shipped)

Help line: *Walk-in: destination → weigh → pick rate → Account/Cash booking →
pay. AWB on receipt. Next booking clears the desk.*

**Desk (catalogue column), top → bottom:**

| Control | Empty text / label |
|---------|-------------------|
| Destination | `Destination city / zone` |
| Zone | `Zone (name from Rates)` |
| Scope | **Domestic** · **International** + volumetric hint (÷5000 / ÷6000) |
| Consignee | `Consignee (receiver)` |
| Weight | `Actual kg` · **Weight in** kg / lb / g / oz |
| Size | `L cm` · `B cm` · `H cm` · **Size in** cm / in / ft / m |
| COD · Carrier cost | `COD amount` · `Carrier cost` |
| Compare | Compare line (when rates allow) · `Carrier for cost (from compare)` · **Use carrier cost** |
| Declared · Contents | `Declared value` · `Contents` |
| E-way · Insurance | `E-way bill no.` · `Insurance (yes/no)` |
| Fuel · Docket | `Fuel` · `Docket charge` |
| Notify | `Notify phone (WhatsApp/SMS hook)` |
| Bulk | Bulk CSV field · **Load bulk row → desk** |
| Chargeable | Chargeable kg line |
| After save | AWB line · **Next booking** |
| Rates | Rate cards with **ADD** (or empty-catalogue message) |

**Cart column:**

| Control | Label |
|---------|-------|
| Header | **Active ledger** · Ready · **Clear** |
| Cart | Lines · empty hint: *Desk empty — set destination & weight, tap ADD on a rate, then pay.* |
| Totals | ACTUAL TOTAL · APPLIED TOTAL · ROUNDED (editable) |
| Mix hint | Type how much in each (cash / UPI / card / credit). Must add up to the bill. Credit needs a customer above. |
| Bill lane | **Account booking** · **Cash booking** |
| Amounts | `Cash amount` · `UPI amount` · `Card amount` · `Credit / khata amount` |
| Pay | **Take payment** |
| Customer | Chip · `Customer name` · **Add / use this name** · **Clear customer** · list with **Pick** |

**Domestic** uses the shop volumetric divisor (usually ÷5000).
**International** uses ÷6000 for that booking only; Settings keep the shop
default.

### 3.3 Cash walk-in

1. Destination and Zone (same zone name as on the rate).  
2. Domestic or International as needed.  
3. Consignee.  
4. Actual weight (and size if bulky). Check Chargeable kg.  
5. **ADD** on a rate (more lines if needed).  
6. Select **Cash booking**.  
7. Put the full bill in **Cash amount** and/or **UPI amount** / **Card amount**
   (they must add up to the bill).  
8. **Take payment**. Note the AWB; print if a printer is set.  
9. **Next booking**.

### 3.4 Account / credit walk-in

1. Same desk fill as §3.3.  
2. **Pick** or **Add / use this name** so **Active ledger** has the consignor.  
3. Select **Account booking**.  
4. Put the amount on **Credit / khata amount** (or mix with cash/UPI/card).  
5. **Take payment**. Cash strip does not rise for the credit part; **They owe
   us** rises. Over credit limit, booking is refused.

### 3.5 Rush drill

Stay on **Booking**. Three parcels back to back (cash, UPI, account/credit),
**Next booking** after each. Then open **Day book** (Part 4).

Optional later: bulk CSV · second paired till · Ops scan after the rush.

------------------------------------------------------------------------

## Part 4 — Day book

Day book answers: where does the business stand **today**? It is a mirror —
fix wrong numbers where you entered them. There is no Refresh.

Help: *Today’s position. Booking = parcels in. Pay out = carrier bills &
expenses. Parties = consignors & carriers.*

**Shortcuts (top of screen):**

| Button | Goes to |
|--------|---------|
| **Book a parcel** | Booking |
| **Expense / carrier bill** | Pay out |
| **Day close** | Day close |
| **Bank book & P&L** | Reports |
| **Parties (debtors & creditors)** | Parties |

**Cards:**

| Card | Lines |
|------|-------|
| TODAY | **Money in (sales)** · **Purchases (COGS)** · **Expenses** · **Gross** · **Net** |
| BOOKINGS | **Account booking** · **Cash booking** |
| PAYMENTS TODAY | **Cash** · **UPI** · **Card** |
| OUTSTANDING | **They owe us** · **We owe them** |
| BANK | **Bank balance** · **Cash in hand (till)** · **Bank net today** |

A thin strip on other screens also shows today’s count / tenders when useful.

After a rush you should see today’s money in, tenders, any account booking
line, and **They owe us** for credit. Check Day book before locking up; use
**Day close** to count the drawer against the books.

------------------------------------------------------------------------

## Part 5 — Corrections

| Situation | Where |
|-----------|--------|
| Not saved yet | Fix fields, **Clear**, or **Next booking** |
| Last action | **Undo** when offered |
| Saved booking wrong | **Bookings** → select the sale → adjust · **Amend** + reason |
| Should not exist | **Void** + reason (rights / manager PIN may apply) |
| Money returned | **Refund** on that sale |
| Party details wrong | **Parties** → **Pick** → **Save changes** |
| Who may amend / void | **Team** rights |

On **Bookings** you also get copy/print receipt paths and day-close summary
helpers beside amend / void / refund.

------------------------------------------------------------------------

## Part 6 — Collecting, Pay out, and Reports

### 6.1 Collect from a consignor

**Parties** → **Pick** → `Amount received` → **Settle cash** / **Pay UPI** /
**Settle card**. Optional `Against bill #…`.

### 6.2 Pay out

Open **Pay out**. Two blocks:

**Goods received** — `Item name` · qty · `Unit cost` · supplier / party kind ·
note · **Paid now** / **On account (AP)** · **Record purchase**.

**Expense** — `Amount` · note · **Paid by** Cash / Bank / UPI / Card · optional
second way · **Expense voucher**.

Also **Inventory & stock adjust** when you need stock.

### 6.3 Day close

Tab **Day close** (also from Day book shortcut) — count drawer vs books.

### 6.4 Reports (screen title: Bank book & P&L)

| Need | Chip / control |
|------|----------------|
| Today | **Day book (today)** |
| Month P&L | **Profit & loss (month)** |
| Balance sheet | **Balance sheet** (operating snapshot) |
| Outstanding / Dr–Cr | **Outstanding / Dr-Cr** |
| Bank | **Bank book** — till↔bank, bank in/out, reconciliation |
| Expense from Reports | **Expense voucher** |
| Spreadsheet | Spreadsheet (CSV) section |

Operating balance sheet = bank + debtors vs creditors + net — not a full
Schedule III statutory statement. Filing portals and auto SMS are later
connectors; the counter runs offline without them.

------------------------------------------------------------------------

## Part 7 — Ops after the rush

**Ops** → `AWB / scan barcode` → **Find / Scan AWB**.

From the desk card (as needed): **Print AWB label**, status (out for delivery /
delivered / failed / reattempt), NDR, dispatch / handover, run sheet / DRS,
Collect / Remit COD, period bill, rider settle.

Start Ops only after Booking has produced AWBs. Own fleet also has **Riders**;
COD remittance also on **COD**.

------------------------------------------------------------------------

## Part 8 — Words and short answers

| Word | Meaning |
|------|---------|
| AWB / CN / docket | Parcel id on receipt and label |
| Chargeable kg | Max(actual, volumetric), rounded to slab |
| Cash booking | Bill lane — paid now (cash / UPI / card mix) |
| Account booking | Bill lane — ledger / tax-invoice style; often with credit |
| Take payment | Commits the mix amounts on the cart |
| They owe us | Consignor AR |
| We owe them | Carrier / AP |
| Day book | Today’s money story |
| Expense voucher | Pay out / Reports expense CTA |
| Settle cash / card | Parties — collect AR |
| COD / POD / NDR / RTO | Delivery money / proof / failed / return |
| Manifest / DRS | Handover or run sheet |

**Cash vs account:** cash/UPI/card in the mix fill the drawer (or bank via UPI)
today; credit / account raises *they owe us* until the party settles; carrier
bills / AP raise *we owe them* until you pay.

**Many people in line** — ADD their rate lines, **Take payment**, **Next
booking**.  
**Mistakes** — Amend or void with a reason from **Bookings**.

------------------------------------------------------------------------

## Part 9 — Demo script (~12–15 minutes)

Practice names. Aim for “this is my counter” plus clear hisab.

| Min | Do | Say |
|-----|----|-----|
| 0–1 | Unlock → Day book ₹0 | Today’s position; fills as we book |
| 1–3 | Parties: carriers + consignors | Customers and carriers; AR and AP separate |
| 3–4 | Rates & services: one slab + zone | Name, unit kg, First kg / amount, Extra, Zone |
| 4–9 | Booking: ADD · lane · mix amounts · **Take payment** · Next booking ×3 | One person finished, then the next |
| 9–10 | Day book cards | Money in, payments today, they owe us |
| 10–12 | Reports: Outstanding · Profit & loss (month) · Balance sheet | Party-wise and monthly picture |
| 12–13 | Account invoice / amend if asked | Invoice vs receipt; corrections with reason |
| 13–14 | Pay out → **Expense voucher** | So net today is honest |
| 14–15 | Ops → **Find / Scan AWB**; optional second till | After the rush; same books on Wi‑Fi |

If he gives a real shop or carrier name, add it live under Parties.

------------------------------------------------------------------------

## Part 10 — Checklist

**Setup**

- [ ] Server Mode + Owner PIN  
- [ ] Carriers and consignors on Parties (**Add party**)  
- [ ] One rate on Rates & services (unit kg, First kg / amount, Extra, Zone)  
- [ ] One consignor with **Set credit limit**  

**Counter**

- [ ] Several bookings: ADD → Cash/UPI/Card amounts → **Take payment** → Next booking  
- [ ] One booking with more than one cart line  
- [ ] Account booking with customer + Credit / khata (or mix)  
- [ ] Credit-limit block once  
- [ ] Domestic / International once when size matters  

**Books**

- [ ] Amend or void once on **Bookings**  
- [ ] Party collect via **Settle cash** / **Pay UPI** / **Settle card**  
- [ ] **Expense voucher** on Pay out  
- [ ] Bank post on Reports → Bank book  
- [ ] Profit & loss (month) and Balance sheet once  
- [ ] Account invoice / receipt copy once  

**Ops and demo**

- [ ] Ops **Find / Scan AWB** on a booked AWB  
- [ ] Explain cash vs account and they-owe-us vs we-owe-them  
- [ ] Run Part 9 twice in ≤15 minutes  

**Second device (after the above)**

- [ ] Settings → Extra tills allowed; client paired; one booking syncs  
- [ ] Optional: offline queue then reconnect  

------------------------------------------------------------------------

## Part 11 — Tab map

| Tab | Meaning |
|-----|---------|
| Day book | Today’s position |
| Booking | Walk-in / account parcels |
| Pay out | Goods received · Expense voucher · stock adjust |
| Rates & services | Rate cards, slabs, zones |
| Ops | Find / Scan AWB, status, labels, dispatch |
| COD | COD register / remit |
| Riders | Own-fleet riders / runs |
| Bookings | History · amend / void / refund |
| Day close | Drawer vs books |
| Reports | Bank book & P&L · day / month / outstanding / CSV |
| Parties | Consignors and carriers |
| Team | PINs / rights |
| Settings | Language, printers, extra tills, backup, booking measures |
| Lock | Lock the desk |

------------------------------------------------------------------------

## Related docs

| Doc | Job |
|-----|-----|
| This file | Hands-on business + app + demo practice (matches live chrome) |
| `MOKLANJO_BUSINESS_TUTORIAL.md` | Full ops day, hisab, tax depth |
| `MOKLANJO.md` | Product brief |
| `MOKLANJO_CLIENTS_REQUIREMENTS.md` | Discovery checklist (separate from this tutorial) |
| `BUSINESS_BOOKS.md` | Shared books vocabulary |
| `PRODUCTION_STATE_FOR_OFFLINE_USAGE.md` | Offline / LAN readiness |
