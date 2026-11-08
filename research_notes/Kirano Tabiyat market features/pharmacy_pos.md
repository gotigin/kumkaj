# India-first chemist / pharmacy POS — market features for Tabiyat

> **Audience:** Tabiyat product (tabiyat.shop), offline-first Makepad counter.  
> **Companion:** `apps/tabiyat/docs/TABIYAT.md`, `apps/tabiyat/docs/PHARMACY_MARKET_RESEARCH.md`.  
> **Hard rules:** Do **not** invent Drugs & Cosmetics Act compliance features. Competitor brand names stay in this research note — **never** in product UI copy, site claims, or in-app strings.  
> **Researched:** 2026-11-08 (web).

---

## 0. What “India chemist software” actually sells

Independent medical stores and small chains need a **counter operating system**, not a hospital formulary and not a drug-inspector simulator. Across Marg-class ERPs, GoFrugal pharmacy, BitMed/kNODE-style offline POS, and lighter medical-store tools, the same mechanics recur:

| Mechanic | Why the counter cares |
|---|---|
| Keyboard/barcode-fast sell | Peak queues; 5-line bill in ~30s is the sales pitch |
| Batch + expiry on every purchase line | Stock is not a single qty; waste and returns are batch-shaped |
| FEFO / near-expiry alerts | Money (returns to distributor) more than “compliance” |
| Salt / composition / substitute search | Brand out of stock; customer wants the salt |
| Strip + loose tablet | Cut strips are normal retail |
| Scheme / free goods on buy (10+1) | Landed cost wrong ⇒ margin lies |
| Customer khata + supplier payable | Notebook replacement |
| GST invoice with MRP-inclusive tax | Pack price is the ceiling; tax is inside MRP |
| Offline till | Broadband drops; queue cannot wait |

**Sources (product pages / guides):**  
https://margcompusoft.com/retail/pharmacy_software.html · https://www.gofrugal.com/retail/pharmacy-software/ · https://www.bitmed.in/features · https://knode.co.in/pharmacy/ · https://nivasik.com/industries/medical-stores/ · https://smartchemist.in/ · https://www.mymbill.com/ · https://pharmhouse.co.in/ · https://medmanthra.com/features · https://bilzy.in/medical-store-billing-software

**Note on “Bitscape-class”:** web search does not show a widely marketed India retail-pharmacy product under the Bitscape IT brand. Treat the ask as **BitMed / kNODE / modern offline pharmacy POS class** (keyboard bill, batch/expiry, GST, offline sync). Bitscape as a general SI/custom-software firm is out of scope.

**MedPlus-style counter needs (organized retail ops, not Tabiyat’s first customer):** on-shelf availability, inventory turns, ageing/liquidation, expiry control, barcode GST POS, replenishment discipline — see MedPlus annual-report risk language on inventory/expiry and LFS job metrics (OSA, GMROI, aging). Independent chemists feel the same losses without HO analytics.  
https://nsearchives.nseindia.com/annual_reports/AR_27560_MEDPLUS_2024_2025_A_21082025222353.pdf · MedPlus-related ops case study on FEFO/expiry waste: https://gomeds.com/case-studies/medplus-pharmacy-mumbai

---

## 1. Chemist daily loop

Concrete day (synthesized from Nivasik “chemist’s day”, Bilzy morning→close, BitMed “morning rush / distributor delivery / close”, SmartChemist counter copy):

1. **Open / shelf glance** — yesterday’s near-expiry, low stock, cash float. Expired stock should already be blocked or written off.
2. **Morning rush** — walk-ins with Rx slips or verbal brand names. Search by barcode, brand short code, or salt. Hold a bill when a customer steps aside (F-keys / park bill is table stakes).
3. **Distributor van** — multi-line invoice with batches, expiry, MRP, PTR, schemes (10+1), free goods. One purchase entry must become stock; second “stock entry” is a failure mode.
4. **Regulars / chronic** — monthly packs on **khata** (udhaar); part-pay; mobile-number lookup.
5. **Returns** — customer return against bill; **supplier return** of short-dated stock with debit note against payable.
6. **Rx-flagged lines** — owner may want doctor/patient note on the sale (bookkeeping). Paper H1 book remains the chemist’s statutory tool unless they separately choose software registers (see §5).
7. **Close** — day sales, cash vs UPI vs card vs khata, stock variance, near-expiry list for return planning, GST summary for CA.

**Action for Tabiyat:** keep Sell → Buy → Books as the spine (Kirano copy). Pharmacy deltas live on item + purchase batch + sell search — not a second product.

---

## 2. Batch + expiry + FEFO / near-expiry alerts

### Bookkeeping (product should do)

| Need | Behaviour |
|---|---|
| Purchase line identity | `batch_code` + `expires_on` + qty + cost + MRP (per batch) |
| Stock quantity | Sum of live batches; never a single undifferentiated qty for medicines |
| Near-expiry alerts | Buckets (e.g. 7 / 30 / 60 / 90 days) with supplier for return |
| Soft FEFO hint | Prefer earliest in-date batch on sell; allow override |
| Hard block (owner policy) | Refuse or warn on **expired** batch |
| Write-off | Expired/damaged leaves books with a reason |
| Supplier return | Debit note linked to batch + payable |

Vendor evidence: FEFO auto-pick and 30/60/90 alerts are universal marketing (GoFrugal expiry blog, PharmHouse, MyMBill, MedManthra, BitMed, Nivasik).  
https://www.gofrugal.com/blog/pharmacy-expiry-management/ · https://pharmhouse.co.in/ · https://nivasik.com/industries/medical-stores/

### Statutory vs shop practice (do not conflate)

- **Selling expired medicine** is a safety/licence problem for the **pharmacist and licensee**, not something Tabiyat should market as “Drugs Act compliant blocking.”
- **Batch + expiry on the bill** is trade practice and traceability; inspectors and customers often expect batch/expiry on the line. That is still **document content**, not a software “compliance certificate.”
- **Near-expiry alerts** are primarily **working-capital / return-to-distributor** tools. Distributors push short dating during schemes; FEFO + return lists reduce write-off (ops narrative in GoMeds MedPlus case; GoFrugal short-expiry purchase warnings).

**Tabiyat posture (aligned with `TABIYAT.md` §2):** batch/expiry = **shop bookkeeping and alerts**. Soft FEFO hint = Have/Later. Auto statutory “inspection readiness” dashboards = **Out**.

---

## 3. Barcode, salt/composition, Schedule H / Rx as owner notes

### Search & identity

| Capability | Counter reality |
|---|---|
| Barcode / wedge | Primary speed path; strip/bottle scan |
| Brand / short code | “pcm”, first letters |
| Salt / composition | Find substitutes when brand OOS |
| Company / rack | Secondary; rack helps new staff |
| Preloaded drug DB | Competitors advertise 50k–3.6L SKUs; licensing and staleness are traps |

Marg lists barcode/salt/substitute/rack search and Sch-H indication: https://margcompusoft.com/retail/pharmacy_software.html  
BitMed: name, salt, company, barcode, short code: https://www.bitmed.in/features  
Nivasik explicitly: **no licensed medicine DB** — salts on **your** items only: https://nivasik.com/industries/medical-stores/

**Tabiyat:** barcode + composition search = **In/Have**. Curated national drug DB import = **Later** (optional reference, never legal authority — `TABIYAT.md` §2).

### Schedule H / Rx (owner notes — not compliance)

Lok Sabha / CDSCO framing for Schedule H1: retail supply of H1 drugs must be recorded in a **separate register** at supply with **prescriber name+address, patient name, drug name, quantity**, kept **three years**, open to inspection.  
https://sansad.in/getFile/loksabhaquestions/annex/179/AU3371.pdf

Independent analysis (cite carefully; not legal advice): Rule **65(3)(h)** lists those **four fields**; paper register is fully legal; Rules are **silent** on electronic registers; vendor “tamper-proof digital H1” extras are **features, not mandated text**. Fake penalty ranges circulate.  
https://cybiqon.in/blog/pharmacy-billing-software-schedule-h1-register

**Therefore for Tabiyat UI/product:**

- `schedule_h` / `rx_required` = **owner toggles**
- Rx / doctor / patient note = **owner wording** on sale or item
- Soft reminder on sell when flagged = OK as practice aid
- **Do not** claim: Drugs Act compliance, auto Form 20/21 registers, “inspection ready,” blocked sale = legal compliance, validated prescriptions, medical advice

Competitor pages that sell “registers fill themselves / can’t take payment until H1 fields” are **workflow products** the owner may want later as **exportable notes**, not something Tabiyat should ship as statutory theatre.

---

## 4. Khata, supplier buys, GST billing

### Khata (customer credit)

- Named account (often mobile), credit limit, part-pay, statement, advance as credit.
- Split tender: cash + UPI + card + khata on one bill.
- SmartChemist / BitMed / MyMBill / Nivasik all treat udhaar as core, not an add-on.  
  https://smartchemist.in/ · https://www.bitmed.in/features · https://www.mymbill.com/

### Supplier buys

| Must-have | Detail |
|---|---|
| Purchase = stock | Batch/expiry/cost/MRP from the same entry |
| Schemes | 10+1 / free goods → true landed cost per strip/tablet |
| Rate history | Last deals vs this invoice (Marg/GoFrugal pitch) |
| Payables | Bill-wise outstanding, ageing |
| Returns | Debit note reduces what you owe **that** distributor |
| Short-expiry on GRN | Warn owner before accepting dumped stock |

Purchase OCR / PDF import is a competitive differentiator (MyMBill, MedLens, BitMed) — high value, not day-one for offline Makepad.  
https://www.mymbill.com/pharmacy-purchase-management · https://medlens.in/

### GST + MRP

- Most formulations: HSN **3003/3004**, commonly **5%** GST after 2025 rate reform (confirm against current notifications when coding rates — do not hardcode forever). Overview: https://richdaddy.in/blog/gst-on-medicines-india-2026
- **MRP is GST-inclusive.** Bill **at or below MRP**; back-calculate taxable value + CGST/SGST (or IGST). Adding 5% on top of pack MRP is a classic bug.
- DPCO: sell ≤ labelled MRP / Form V list; NPPA ceiling context: https://www.pib.gov.in/PressReleseDetailm.aspx?PRID=2200935&lang=2&reg=48 · retailer GST-revision FAQs: https://www.arogyacompliance.com/articles/answering-faqs-by-retailers-about-the-sale-of-medicines-with-revised-mrp-due-to/
- GSTR-1 / 3B **exports for CA** matter more than in-app filing for single shops.
- e-Invoice / e-Way: threshold businesses — **Later**, not counter MVP.

**Tabiyat:** reuse Kirano GST presentation + khata + Buy; pharmacy-specific = MRP discipline + batch on purchase + scheme costing (**Later** if not already).

---

## 5. What good pharmacy software does vs compliance theatre

### Good (ship / aspire)

1. **Never stop the till** — true offline bill + stock deduct + print; real invoice numbers, not temp IDs that rewrite on sync.  
   https://www.bitmed.in/docs/offline-and-sync/off-01-billing-while-offline · https://knode.co.in/pharmacy/ · https://www.bitmed.in/blogs/offline-pharmacy-billing
2. **Purchase entry creates batches** — one typing job.
3. **FEFO + near-expiry + supplier return path** — protect cash.
4. **Search that matches how staff think** — barcode, brand, salt.
5. **Strip/loose** — pack size math that keeps stock and margin honest.
6. **Khata both ways** — customer receivable + supplier payable.
7. **GST that matches the pack** — MRP-inclusive, HSN on lines, CA exports.
8. **Day book the owner trusts** — cash drawer vs UPI vs credit columns.
9. **Optional owner reminders** for Rx-flagged items — no legal claim.
10. **Honest “out of scope” list** (Nivasik model: no H1 register product, no licensed DB) — builds trust.

### Compliance theatre (avoid in Tabiyat)

| Theatre | Why it is theatre / dangerous claim |
|---|---|
| “Inspection ready” checklist with green ticks for Form 20/21 + H1 | Licences live with the licensee; software ticks ≠ statutory compliance |
| Auto “Drug register filled itself” as legal compliance | Rules ask the **licensee** to record supply; software can **help**, cannot certify |
| Inflated H1 field sets sold as “the law” | Statutory list is four fields (prescriber name+address, patient name, drug, qty) per Rule 65(3)(h) analysis — extras are product choices |
| “Mandatory software H1 register” | Paper remains legal; Rules silent on e-registers (Cybiqon) |
| Fake fine ranges / fear marketing | Misquoted penalties circulate |
| Blocking sale marketed as Drugs Act compliance | Owner policy / workflow ≠ legal validation of Rx |
| DPCO “enforcement” without accurate ceiling data | Wrong ceilings worse than none; leave to Later with clear non-authority |

**Good middle path for Tabiyat Later:** export a **date-ordered CSV/PDF of owner-entered Rx notes** (if fields filled) labelled explicitly as **owner records / for your paper register**, never “statutory H1 register.”

---

## 6. Lessons / failure modes

| Failure | Consequence | Design lesson |
|---|---|---|
| Cloud-only till | Peak-hour hang → walkaways, 1★ reviews (“slow at evening rush”) | Offline-first local books (Kumkaj doctrine) |
| Offline that can’t purchase | Delivery arrives during outage; stock diverges | Prefer offline Buy/GRN too, or clear “purchase needs net” with queue |
| Temp bill numbers that change on sync | GST/reprint chaos | Stable local invoice identity |
| Held bills lost on crash | GoFrugal release note: abrupt close → held bills unrecoverable | Persist holds to disk |
| Sync conflict on multi-till stock | Negative stock / double sell | LAN truth + conflict UX (Kirano/Tabiyat LAN path) |
| Browser clear while offline pending | Lost queue (BitMed docs warn) | Native local DB > browser cache |
| Purchase typing hell | Staff skip batches → FEFO useless | OCR Later; until then fast keyboard GRN |
| Scheme ignored | Margin reports lie | Cost after free goods |
| GST added on MRP | Overcharge / distrust | Inclusive tax math |
| Giant drug DB as authority | Wrong schedule flags, stale salts | Owner-owned composition; DB optional reference |
| Compliance-first onboarding | Slow go-live; owners want to bill today | Bill first; flags optional |
| Selling “registers” without print/sign workflow | Inspector still wants a book/artefact | If ever export notes, print+sign guidance — not “digital = done” |

Offline partial mode (bill yes, purchases/void/credit no) is a common compromise — document it honestly: https://www.bitmed.in/docs/offline-and-sync/off-01-billing-while-offline

---

## 7. Concrete prioritized backlog — Tabiyat (offline-first Makepad)

Legend: **In** = product promise / must be true · **Have** = already in crate or explicitly shipped · **Later** · **Out**. Aligns with `TABIYAT.md` §4b; refined from market evidence above.

### In (promise — chemist OS)

- Counter sell with GST presentation (MRP-aware when medicines priced at pack)
- Customer **khata** + split tender
- Day totals / books spine (cash, UPI, card, credit)
- Item fields: barcode, composition/salt, Schedule H / Rx **owner flags**, Rx note text
- Sell search ranking: barcode + name + composition
- Buy with **batch_code** + **expires_on** → stock
- Near-expiry **alerts** (books strip / status)
- Offline-first local books + LAN tills (workspace pattern)
- No restaurant Floor/Kitchen chrome (Venue = Counter)

### Have (shipped / in code today — verify on build)

- Template `tabiyat_pharmacy`, counter-only chrome
- Barcode / composition search ranks
- Schedule H / Rx flags + soft Sell reminder (status, not blocker-as-law)
- Buy batch + expiry fields; Books expiry strip
- Soft near-expiry / FEFO **hint** on Sell add
- `barcode_scan` module (wedge primary; image decode fallback)
- Kirano-copied books, staff, backup, fiscal spine

### Later (earn; do not market as compliance)

| Item | Why |
|---|---|
| Hard FEFO batch pick at sell (default earliest in-date; one-key override) | Cuts expiry waste; market table stakes |
| Strip ↔ loose tablet pack math | Cut-strip retail |
| Scheme / free goods on purchase → landed cost | Margin truth |
| Supplier return + debit note against payable | Closes the expiry money loop |
| Near-expiry buckets 30/60/90 + “return this week” list | Ops |
| Bill hold / park / recall | Peak counter |
| Substitute list = other own SKUs with same composition | No licensed DB required |
| GSTR-1 / 3B / HSN export for CA | Accountant workflow |
| MRP ceiling soft-block (owner setting) | DPCO/MRP practice aid — **not** NPPA authority |
| Purchase import (CSV first; OCR photo later) | Beats typing 40-line invoices |
| Optional export of owner Rx-note log (CSV/PDF) clearly labelled non-statutory | Helps paper H1 book; no legal claim |
| Thermal reprint “DUPLICATE” | Audit hygiene |
| Multi-batch on one SKU valuation reports | Owner clarity |
| Refill / chronic reminders (WhatsApp optional) | Loyalty; not MVP |

### Out (do not build / do not claim)

- Drugs & Cosmetics Act / Schedule H1 **statutory register product**
- Auto Form 20 / 21 / narcotic Form workflows as compliance
- “Inspection ready” licence dashboard
- Prescription **validation**, doctor registration verification, medical advice
- Claiming software replaces the licensed pharmacist
- Licensed national drug DB as **legal authority** for schedule flags
- e-pharmacy marketplace / teleconsult stack
- Hospital ward formulary / inpatient MAR
- Fake DPCO ceiling enforcement without maintained official data
- Competitor brand names anywhere in UI or site copy

---

## Checklist — what Tabiyat should ship

Use as a go/no-go for the next pharmacy-facing release. Competitor names never appear in UI.

### Counter must-work

- [ ] Bill medicines offline with stable bill identity and local print
- [ ] Find item by barcode, name, and composition/salt
- [ ] Add line fast; hold/park bill if implemented (or document as Later)
- [ ] Split pay: cash / UPI / card / khata
- [ ] Soft status when Schedule H / Rx owner flag is set (no legal wording)

### Stock must-work

- [ ] Purchase entry records batch + expiry (required for medicine SKUs)
- [ ] Sell prefers / hints earliest in-date batch
- [ ] Expired batch warned or blocked by **owner policy**, copy ≠ “Drugs Act”
- [ ] Books / report lists near-expiry with days left
- [ ] Write-off path for expired/damaged with reason (or clear Later)

### Money must-work

- [ ] Customer khata balance, part-pay, statement
- [ ] Supplier payable from buys; payments clear it
- [ ] GST lines with HSN; **tax inside MRP** when selling at pack MRP
- [ ] Day close: sales by tender + khata column

### Honesty must-work

- [ ] In-app / site copy: Schedule H & Rx are **owner notes**, not compliance
- [ ] No “statutory register,” “inspection ready,” or Form-auto claims
- [ ] No competitor names in product strings
- [ ] README/docs list **Out** items so sales/support don’t invent them

### Ship-next (Later, prioritized)

1. Hard FEFO pick + override  
2. Strip/loose pack math  
3. Scheme/free goods costing on Buy  
4. Supplier return + debit note  
5. CA GST export  
6. Own-SKU substitute by composition  
7. Optional Rx-note export labelled non-statutory  

---

## Source index (URLs)

| Topic | URL |
|---|---|
| Marg pharmacy features | https://margcompusoft.com/retail/pharmacy_software.html |
| Marg chemist / H1 report docs | https://margcompusoft.com/retail/chemist_software.html · https://care.margcompusoft.com/margerp/departments-reports/210/1/null |
| GoFrugal pharmacy / expiry | https://www.gofrugal.com/retail/pharmacy-software/ · https://www.gofrugal.com/blog/pharmacy-expiry-management/ |
| BitMed features / offline | https://www.bitmed.in/features · https://www.bitmed.in/docs/offline-and-sync/off-01-billing-while-offline |
| kNODE offline pharmacy | https://knode.co.in/pharmacy/ |
| MedManthra FEFO/GST | https://medmanthra.com/features |
| PharmHouse | https://pharmhouse.co.in/ |
| MyMBill / purchase OCR | https://www.mymbill.com/ · https://www.mymbill.com/pharmacy-purchase-management |
| SmartChemist | https://smartchemist.in/ |
| Nivasik medical store (honest outs) | https://nivasik.com/industries/medical-stores/ |
| Bilzy strip/loose + purchase | https://bilzy.in/medical-store-billing-software |
| MedLens purchase photo | https://medlens.in/ |
| H1 rule / software honesty | https://cybiqon.in/blog/pharmacy-billing-software-schedule-h1-register |
| Lok Sabha H1 conditions | https://sansad.in/getFile/loksabhaquestions/annex/179/AU3371.pdf |
| GST medicines / MRP-inclusive | https://richdaddy.in/blog/gst-on-medicines-india-2026 |
| DPCO / MRP PIB | https://www.pib.gov.in/PressReleseDetailm.aspx?PRID=2200935&lang=2&reg=48 |
| GST rate change retailer FAQs | https://www.arogyacompliance.com/articles/answering-faqs-by-retailers-about-the-sale-of-medicines-with-revised-mrp-due-to/ |
| MedPlus ops / inventory risk | https://nsearchives.nseindia.com/annual_reports/AR_27560_MEDPLUS_2024_2025_A_21082025222353.pdf |
| FEFO / expiry waste case (ops) | https://gomeds.com/case-studies/medplus-pharmacy-mumbai |

---

*End of note. Product plan of record remains `apps/tabiyat/docs/TABIYAT.md`.*
