# Khaynjo — restaurant market research (Petpooja and the field)

> **Scope:** discovery evidence for Khaynjo only (docs/README.md rule 3).
> Ideas here that apply to every Kumkaj app are written once in the root
> docs and only linked from here: branches as separate businesses
> (vision §5), rights with approval and reason (vision §17), cancellation
> after the kitchen (vision §18), shift and day-end cash
> (`BUSINESS_BOOKS.md` § Shift and day-end cash).
> **Collected:** 2026-10-23. Turned into Khaynjo's plan in
> [`KHAYNJO.md`](KHAYNJO.md) §4b and §5.

## 1. Why this exists

A restaurant client runs **Petpooja** today and wants the same daily
mechanics **without any cloud** and with **more control** in the owner's
hands. Khaynjo is free and offline forever (vision §21), so the job is:
**learn from** what Petpooja and the best restaurant systems do at the
counter, on the floor and in the kitchen, and **do better** — cover all
or most of their useful features (cloud-only ones aside for now),
designed for Khaynjo's own users rather than cloned (vision §24).

## 2. Evidence used

| Source | What it gave us |
|---|---|
| Client video, "Rajdhani" (`Kumkaj/videos/rajdhani/`, 3 clips, Oct 2026) | The real daily loop at a live restaurant (§3) |
| Petpooja screenshot set (`Kumkaj/petpooja-screenshots/`, 450 images + `INDEX.md`) | Desktop POS, captain app, KDS, setup, printers, user rights, cloud dashboard |
| Petpooja blog: [200+ tools across nine modules](https://blog.petpooja.com/operations-workflows/must-try-features-petpooja-pos/) | Their own module list, offline claim, build tiers |
| Petpooja blog: [user rights](https://blog.petpooja.com/industry-business-guides/restaurant-pos-user-rights-who-should-access/), [KOT cancellation](https://blog.petpooja.com/glossary/kot-cancellation/), [day-end](https://blog.petpooja.com/finance-compliance/restaurant-day-end-report-how-close-and-reconcile/), [shift change](https://blog.petpooja.com/glossary/quick-shift-change/) | Rights, cancel, day-end and shift mechanics (§5) |
| Petpooja product pages ([POSS](https://www.petpooja.com/poss), [fine dine](https://www.petpooja.com/fine-dine-restaurant-software)) and listings ([Techjockey](https://www.techjockey.com/detail/petpooja-pos), [SoftwareSuggest](https://www.softwaresuggest.com/petpooja)) | Split/merge/move, area-wise prices, BOGO/combos, menu timings |
| India comparisons ([Petpooja's own](https://blog.petpooja.com/industry-business-guides/best-restaurant-pos-software-india/), [restrofi](https://restrofi.com/blog/restaurant-pos-alternatives-india), [techarion](https://techarion.com/blog/restaurant-pos-system-india-comparison-2025)) | Restroworks/Posist (chains), DotPe (QR, weak offline), UrbanPiper (aggregator layer), GoFrugal (offline + inventory) |
| Global comparisons ([TouchBistro vs Toast](https://www.touchbistro.com/blog/touchbistro-vs-toast/), [Square/Toast/Lightspeed](https://www.expertmarket.com/pos/square-vs-toast-vs-lightspeed), [launchadvisor](https://www.launchadvisor.co/guides/restaurant-pos-system-comparison-full-service-restaurant)) | Coursing, split checks by item/seat, floor map + timers, offline modes |

The YouTube demo link (`watch?v=NXwNEEO1L7o`, "PETPOOJA BILLING SOFTWARE
DEMO") only returned its title to a fetch, so it adds nothing beyond the
screenshots, which already cover the same tutorial channel.

Most comparison pages are written by competing vendors; we only take
**mechanics** from them, never rankings or prices.

## 3. The client's loop (Rajdhani video)

- **Hardware:** a Windows laptop is the Petpooja POS (master); an
  Android tablet is the captain (waiter) app; a thermal printer in the
  kitchen prints KOTs; an ordinary **Canon inkjet** prints bills.
- **Table view:** two areas — **Dine In** (D1…D20) and **Parcel**
  (P1…P20). Takeaway parcels are run *as tables*, so a parcel can take
  KOTs and be billed later like any table.
- **Table states by colour:** blank → yellow (running KOT) → green
  (bill printed) → paid/free. Each running card shows **minutes since
  seated** and the **running amount**, with print-bill and view icons.
- **Captain tablet:** item names are typed in **Gujarati**; the waiter
  taps **+** on items grouped by category (thalis, Sosyo/Soda, …) and
  sends the KOT; it prints in the kitchen without anyone carrying it.
- **Settle & Save** on the laptop: payment type **Not Paid / Cash /
  Card / Due / Other / UPI / Part**, *Customer Paid*, *Return to
  Customer*, *Tip*, *Settlement Amount* → **Settle** frees the table.

## 4. Petpooja, module by module

| Module | What it does (mechanics) | Needs cloud? |
|---|---|---|
| **Billing screen** | Dine In / Delivery / Pick Up tabs; category list + item grid; search by name **or short code**; per-item special notes; variations (size) and add-ons dialog; Split; Complimentary; Sales Return; BOGO; payment chips Cash / Card / Due / Other / More (UPI, wallets) + Part; buttons **Save, Save & Print, Save & eBill, KOT, KOT & Print, Hold** | No (eBill = SMS link: yes) |
| **Table view** | Areas (AC / Garden / Non-AC / Family …); states Blank / Running / Printed / Paid / Running KOT; per-card time + amount; **Move KOT / Items**; move & **merge** tables; add table; Contactless; waiter-call alerts | No |
| **Settle & Save** | Payment type, customer paid, return to customer, **tip**, settlement amount; mandatory tender allocation (no "Not Paid" leftovers at day end) | No |
| **KOT** | Category/station-wise KOT routing to printers; copies per printer; KOT carries table, waiter, notes, never prices; **KOT cancel** (whole or one item) with reason code, prep status (wastage if already cooked), approver, time; modification vs cancellation logged differently | No |
| **Kitchen display (KDS)** | KOT cards with token/table, timers, Running → Dispatch; item aggregation sidebar ("all-day" counts) | No (LAN) |
| **Captain app** | Tables by area, running timers; New KOT, Running Bill, Unsuccessful KOT retry; **Find Server IP** (scans the /24 LAN for the POS); sync code to bind to the outlet; waiter name on KOT; voice ordering | Binding uses cloud sync code; ordering is LAN |
| **Orders** | Order list/summary, bill no / KOT no search, bill cancel with reason, assign order to captain / **delivery boy**, recent orders panel with saved/printed/paid legend | No |
| **Operations** | Orders, Online orders, KOTs, Customers, **Cash Flow, Expense, Withdrawal, Cash Top-Up**, Inventory, Table, Virtual Wallet, Due Payment, Delivery Boys, Feedback, LED display, Language profiles, Currency conversion | Mostly no |
| **Day end / shift** | Day End Summary; opening float; denomination count; expected vs counted; variance named before lock; shift handover slip (shift, biller, sales split, float, counted, variance); Bill Settlement report | No |
| **User rights** | Per-login switches, some with a **"with password"** level: bill/KOT reprint, special discount, after-save bill modification (add item / modify qty & delete), require modification reason, complimentary bill, **NC items**, pending-order edit, delete item first time, show Due box, ask password when deleting items from a saved bill/KOT; After Settlement Modification; bulk delete needs OTP | Dashboard UI is cloud; rules apply locally |
| **Menu** | Items, categories, variations, add-ons, combos, **area-wise prices**, multiple menus per area, **menu timings** (breakfast/lunch), item on/off, short codes | Central push is cloud |
| **Inventory** | Raw materials, recipes (multistage), consumption, low-stock + expiry alerts, purchase orders, suppliers, central kitchen transfers, daily stock closing | Local build: offline |
| **Reports** | 94 reports: item / category / order summary, biller & waiter performance, cancelled / modified / discounted, tax, hourly, counter-wise, happy-hour | Cloud dashboard; POS has a subset |
| **CRM / loyalty** | Customer phone/name/address at billing, history, favourites, **virtual wallet**, loyalty points, labels, campaigns (SMS/WhatsApp) | Campaigns: yes |
| **Printing** | Printer per document (bill / KOT / station), USB / LAN / Bluetooth, paper size, copies, customizable bill format (header, GSTIN, FSSAI, footer) | No |
| **Online orders** | Zomato/Swiggy accept/reject, prep time, auto-accept, store & item on/off, reconciliation | **Yes — out of scope** |
| **Add-ons** | Self-order kiosk, QR scan & order, website ordering, reservations calendar, waiter-calling device, token display TV, feedback app | Mixed |

**Left out on purpose (cloud-only reasons):** sync codes, the cloud
dashboard, head-office menu push, aggregator integration, SMS/WhatsApp
campaigns, Petpooja Pay, AI agents, marketplace. A Khaynjo owner who
owns several restaurants runs **one Khaynjo restaurant per branch**
(vision §5) and compares them from exported day snapshots — no cloud.

## 5. Mechanics worth having (designed our own way)

1. **The running table** — items punched → KOT(s) → more KOTs → print
   bill → settle. The table holds the order between KOTs; the counter
   bills what the waiter sent. (Khaynjo gap found 2026-10-23: KOTs kept
   only a summary and the cart was cleared on send, so the counter had
   nothing to bill.)
2. **Parcel as a table area** — takeaway that waits is a table.
3. **Colour states** that every device agrees on, with minutes + amount.
4. **Settle & Save** — tendered, change to return, tip; Due goes to the
   customer's khata.
5. **Move items / move table / merge tables** before the bill.
6. **Cancel after KOT** — reason, prep status (wastage), approver,
   never a silent delete; kitchen gets a cancellation slip.
7. **Complimentary (NC)** — item or whole bill at ₹0 with reason and
   approver; counted, not hidden.
8. **Rights with approval** — a switch per action, some "with
   password" (an Owner/Manager PIN at the moment of action).
9. **Short codes + search** for fast punching.
10. **Day end and shift** — float, expected vs counted, variance named.
11. **Find the server on the LAN** — a tablet should not need someone
    to type an IP.
12. **Bill format** — restaurant name, address, GSTIN, **FSSAI licence
    no.** (mandatory on Indian food bills), footer line.

## 6. What the best systems elsewhere add

| Idea | Seen in | Khaynjo view |
|---|---|---|
| Course firing (starters now, mains on "fire") | Toast, TouchBistro | **Have** (2026-10-27) — hold/fire per course; held never prints |
| Split check by item / by seat / evenly | Toast, Square, TouchBistro | **Have** (2026-10-27) — seat numbers + split evenly / by seat |
| Visual floor map + table timers | Toast, TouchBistro | **Have** (2026-10-27) — Grid ↔ Layout; timers already on tiles |
| Offline mode that is the *normal* mode | TouchBistro (local iPad), GoFrugal | Khaynjo is local-first by design |
| "86" list (sold out now) that every device sees | Toast | Already: item on/off (sold out) syncs |
| Kitchen all-day counts | Toast KDS, Petpooja KDS | **Have** — "To cook now" on Kitchen |
| Happy-hour / time-based price | Petpooja, Toast | **Have** (2026-10-27) — `schedule_json` / `price_now` |
| Tip pooling / tip report | Toast, Square | **Have** (2026-10-27) — tip per bill + pooling report |

## 7. Decision summary

- **Build now (§5 1–12):** **Done** — designed for Khaynjo's users, in its
  own code (no shared crate). See [`KHAYNJO.md`](KHAYNJO.md) §4d.
- **Local/LAN catch-up from market + [`RESTAURANT_RESEARCH.md`](RESTAURANT_RESEARCH.md):**
  **Done 2026-10-27** (combos/BOGO, timings, UPI QR, KDS age/recall,
  coursing, split by seat, loyalty/wallet, PO/expiry/variance, waitlist,
  token/expo/kiosk, pay-at-table, time clock, drawn floor, WhatsApp/SMS
  share, allergens/vouchers/bulk price, …). Full matrix there §6–§8.
- **Still later / Cloud:** priced variation matrix polish; live aggregator
  SaaS; public mobile-data ordering; live multi-outlet dashboard —
  unless an owner opts into Kumkaj Cloud (vision §21). Optional Net
  stubs must never gate the counter.
- **Never as a gate:** sync codes, remote-only dashboard, campaigns that
  require hosted messaging. SMS delivery is planned as a cloudless Local
  SMS Gateway (Android sibling) plus optional paid Cloud SMS —
  [`SMS_MESSAGING.md`](../../../docs/SMS_MESSAGING.md).
