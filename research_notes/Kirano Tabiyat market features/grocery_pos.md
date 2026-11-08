# India-first neighbourhood kirana / grocery POS — research for Kirano

> **Product:** Kirano · [kirano.shop](https://kirano.shop) · offline-first Makepad counter app  
> **Scope:** neighbourhood kirana, grocery, general store — not restaurant, not pharmacy depth, not multi-outlet enterprise ERP.  
> **UI rule:** Competitor brand names appear only in this research file. Never put them in product chrome, marketing copy that names rivals, or “works like X” UI strings.  
> **Related product docs:** [`apps/kirano/docs/KIRANO.md`](../../apps/kirano/docs/KIRANO.md), [`apps/kirano/docs/KIRANA_MARKET_RESEARCH.md`](../../apps/kirano/docs/KIRANA_MARKET_RESEARCH.md), [`docs/BUSINESS_BOOKS.md`](../../docs/BUSINESS_BOOKS.md).  
> **Researched:** 2026-11-08 (web + in-repo Kirano map).

---

## 0. Class of tools studied (mechanics only)

| Class | What the market actually sells at the counter | Sources |
|---|---|---|
| Mobile/desktop SMB billing | Fast GST/non-GST bill, barcode, stock auto-update, udhaar/khata, UPI on bill, WhatsApp share, GSTR export | [Vyapar retail billing](https://vyaparapp.in/free/billing-software-for-retail-shop), [Vyapar feature list](https://vyaparapp.in/blog/vyapar-app-features/) |
| Desktop retail / supermarket POS | 2s checkout, weigh-scale interface, multi-counter LAN, MRP billing, batch/expiry, schemes, loyalty | [Marg supermarket / kirana POS](http://www.margsoftwares.com/), [Marg key features](https://margcompusoft.com/key_features.html) |
| On-prem / hybrid retail POS | Server + LAN clients, offline counter + cloud sync story, multi-outlet later | [GoFrugal retail](https://www.gofrugal.com/retail/), [cloud vs offline](https://www.gofrugal.com/blog/cloud-vs-offline-billing-software/), [GoBill intranet mode](https://community.gofrugal.com/portal/en/kb/gobill-mobile-sellquick/setup/articles/internet-mode-and-intranet-mode) |
| Accounting + retail POS | Barcode POS, split pay, carton↔pc units, GSTR dashboards, MRP-wise stock | [BUSY retail shop](https://busy.in/accounting-software/retail-shop/), [BUSY features](https://busy.in/busy-features-and-faqs/) |
| Accountant ledger used as till | POS voucher type, multi-mode pay, barcode as alias, MRP display | [TallyPrime POS](https://help.tallysolutions.com/point-of-sale-tally/), [Tally barcodes](https://help.tallysolutions.com/barcodes/), [Tally small retailers](https://tallysolutions.com/tally/tallyprime-for-small-retailers-pos-inventory/) |
| Lightweight / Android POS | Camera barcode, WhatsApp bills, low-stock/expiry, payment reminders | [BharatPOS (Magicstep listing)](https://magicstep.andro.io/), [BharatPOS site](https://www.bharatpos.xyz/) |
| Kirana buyer guides / niche POS | Offline-first, weigh loose items, khata, thermal, LAN multi-counter checklists | [BillingKo kirana buyer guide](https://retail.billingko.in/guides/how-to-choose-kirana-billing-software), [KOSH kirana](https://getkosh.co.in/billing-software-for-kirana-store), [BillClap offline kirana](https://www.billclap.com/offline-billing-software/kirana), [Khatadar kirana](https://khatadar.com/for/kirana-store), [BillBasket kirana](https://www.billbasket.app/pos-billing-software/kirana-grocery) |

**Takeaway for Kirano:** win the **evening queue + khata + buy→stock + day cash/UPI close** loop offline on one Server + LAN Clients. Do not chase full ERP filing theatre in v1.

---

## 1. Owner daily loop (open → sell → credit/khata → buy/restock → day close)

Composite of buyer guides and retail day-end practice ([BillClap day run](https://www.billclap.com/offline-billing-software/kirana), [BUSY day-end notes](https://busysoftware.net/busy-software-for-retailers), [BillingKo offline reliability](https://retail.billingko.in/guides/how-to-choose-kirana-billing-software)):

| Phase | What the owner actually does | What the counter app must make fast |
|---|---|---|
| **Open** | Unlock till; glance yesterday’s open khata / cash expected; optional float | Books: today’s cash / UPI / card / credit outstanding; staff PIN |
| **Sell (flood)** | Scan or short-code FMCG; weigh/type kg for loose rice/dal/oil; print thermal; accept cash + UPI split; put regulars on khata | ≤3 taps after item known: item → qty → settle; barcode/short-code Enter; units kg/g/pc/L; split tender; credit party |
| **Credit / khata** | “Put on his book”; evening settle part cash/UPI; dispute “what did I take?” | Party ledger with bill lines; part payment; balance statement; phone on party |
| **Buy / restock** | Wholesaler van / godown trip; bill arrives with MRP/HSN; stock goes on shelf before typing | Purchase entry that **increases sellable stock** + optional AP; pack units (carton→pc); optional batch/expiry |
| **Midday hygiene** | Returns, damaged packet, price change on new MRP batch | Return against bill; stock adjust with reason; MRP vs sell rate distinct |
| **Day close** | Count drawer vs software cash; check UPI; who still owes; backup | Tender totals vs float; short/excess reason; aging khata; local backup prompt |
| **Extra till** | Second phone/tablet during festival rush | Same books via LAN Client; one Server writer |

**Kirano product face already aimed here:** Unlock → Books → Sell / Buy / Credit → day picture ([`KIRANO.md`](../../apps/kirano/docs/KIRANO.md) §4b). Keep restaurant Floor/Kitchen out.

---

## 2. Must-have vs nice-to-have vs compliance theatre

### Must-have (neighbourhood counter dies without these)

| Need | Why |
|---|---|
| Offline bill + stock + print with no WAN | Broadband/hotspot failure is normal ([BillingKo](https://retail.billingko.in/guides/how-to-choose-kirana-billing-software), [GoFrugal offline mode](https://www.gofrugal.com/blog/cloud-vs-offline-billing-software/), [KOSH offline FAQ](https://getkosh.co.in/billing-software-for-kirana-store)) |
| Fast item find: name, short code, barcode | Queue speed; wrong-name billing causes stock drift ([stock mismatch guide](https://richdaddy.in/guides/stock-not-matching-billing-software-fix)) |
| Units: kg/g, pc, L/ml + pack conversion | Loose staples + carton purchase ([KOSH loose weighing](https://getkosh.co.in/billing-software-for-kirana-store), [BUSY multi-unit](https://busy.in/accounting-software/retail-shop/)) |
| Cash / UPI / card + **split tender** + change | How people actually pay ([Tally multi-mode](https://help.tallysolutions.com/point-of-sale-tally/), [BUSY receipts](https://busy.in/accounting-software/retail-shop/)) |
| Customer khata AR + supplier AP | Relationship credit is the moat ([BillingKo khata](https://retail.billingko.in/guides/how-to-choose-kirana-billing-software), [BillBasket khata](https://www.billbasket.app/pos-billing-software/kirana-grocery)) |
| Purchase → stock up automatically | Delayed purchase entry → negative stock ([catchus stock drift](https://catchus.in/articles/why-the-stock-on-your-screen-stops-matching-the-shelf.html)) |
| MRP field ≠ sell rate; warn sell above MRP | Legal Metrology: sell ≤ printed MRP; sell below OK ([LM FAQ MoCA](https://consumeraffairs.nic.in/sites/default/files/file-uploads/latestnews/LM_FAQs.pdf)) |
| Tax invoice **and** cash memo / non-GST bill lanes | Registered vs unregistered / composition realities ([GST Council flyer](https://www.gstcouncil.gov.in/sites/default/files/e-version-gst-flyers/Tax_Invoice_and_other_new.pdf), [DukanUp bill types](https://www.dukanup.com/guides/gst-bill-format-for-shops)) |
| Day book: tender split + who owes | Owner’s bedtime question ([BUSINESS_BOOKS](../../docs/BUSINESS_BOOKS.md) grocery row) |
| Thermal print + local backup | Faded slips + disk death are routine ([backup plan](https://richdaddy.in/blog/shop-billing-data-backup-india)) |
| PIN roles / audit on voids & rate edits | Staff leakage narrative is ubiquitous in SMB POS marketing ([BharatPOS “sale leakages”](https://www.bharatpos.xyz/)) |
| One Server + LAN Client tills | Festival second counter ([GoFrugal LAN/SQL clients](https://www.gofrugal.com/system-requirement.html), [KOSH multi-counter checklist](https://getkosh.co.in/billing-software-for-kirana-store)) |

### Nice-to-have (after the loop is boring-fast)

| Need | Notes |
|---|---|
| Hardware weigh-scale import | Huge for loose-only shops; type-kg is enough for many ([BillingKo scales](https://retail.billingko.in/guides/how-to-choose-kirana-billing-software)) |
| Self barcode / label print | Unbranded / re-pack ([Marg barcode](https://margcompusoft.com/key_features.html), [Vyapar generate barcode](https://vyaparapp.in/free/billing-software-for-retail-shop)) |
| WhatsApp bill / khata reminder | Conversion feature, not books spine ([Khatadar](https://khatadar.com/for/kirana-store), [Vyapar WhatsApp](https://vyaparapp.in/blog/vyapar-app-features/)) |
| Schemes: B1G1, slab discounts, loyalty points | Supermarket depth ([Marg](http://www.margsoftwares.com/), [BUSY schemes](https://busy.in/accounting-software/retail-shop/)) |
| Hold/recall bill, multi-window bills | Busy counters ([Vyapar multi-window](https://vyaparapp.in/free/billing-software-for-retail-shop)) |
| Excel bulk item upload | Catalogue bootstrap ([BillClap](https://www.billclap.com/offline-billing-software/kirana)) |
| Rack / godown / multi-MRP batches | Shrinkage and MRP change reality ([BUSY MRP stock](https://busy.in/saffron/), [catchus dual MRP](https://catchus.in/articles/why-the-stock-on-your-screen-stops-matching-the-shelf.html)) |
| GSTR-1 / 3B JSON export for CA | Useful; filing UX is not the till job ([KOSH GSTR](https://getkosh.co.in/billing-software-for-kirana-store)) |
| Tally/XML export | Accountant handoff, optional ([KOSH Tally XML](https://getkosh.co.in/billing-software-for-kirana-store)) |
| Voice search Hindi/English | Niche speed claim ([Khatadar](https://khatadar.com/for/kirana-store)) |
| Online storefront / marketplace | Out of neighbourhood counter core ([BharatPOS storefront](https://magicstep.andro.io/)) |

### Compliance theatre to avoid (do not block Sell)

| Theatre | Why skip early |
|---|---|
| In-app GSTR filing + IMS + notice downloads as v1 | Accounting suite feature ([BUSY GST dashboard list](https://busy.in/accounting-software/retail-shop/)); owners still use CA |
| Forced e-invoice IRN / e-way on every B2C kirana bill | Threshold / B2B-driven; most walk-ins are B2C ([B2C bill rules](https://richdaddy.in/guides/b2c-bill-rules-gst-retail-shop-india)) |
| Fake “100% GST compliant” badge without serial + GSTIN fields | Real need is correct document type + rates + HSN when registered ([GST Council](https://www.gstcouncil.gov.in/sites/default/files/e-version-gst-flyers/Tax_Invoice_and_other_new.pdf)) |
| Enterprise multi-godown manufacturing BOM | Marg Gold territory ([Marg editions](http://www.margsoftwares.com/)) |
| Cloud-only till with no offline path | Explicit buyer reject criterion ([BillingKo](https://retail.billingko.in/guides/how-to-choose-kirana-billing-software)) |
| Competitor-named “modes” or clone UI labels | Brand pollution; teach **Sell / Buy / Credit / Books** in Kirano words |

---

## 3. Catalogue: short codes, barcode, MRP vs sell, units, packs, low stock

### Item identity

| Mechanism | Practice | Kirano implication |
|---|---|---|
| **Short code** | 2–6 char keyboard entry for staples (atta, oil brands) | First-class search key; Enter adds line |
| **Barcode** | USB/BT wedge or camera; alias on item ([Tally alias barcode](https://help.tallysolutions.com/barcodes/), [Vyapar scan](https://vyaparapp.in/free/barcode-billing-software)) | Scan-to-add on Sell; tolerate missing barcodes on loose goods |
| **Name + aliases** | Hindi/English / brand / local name | Alias list; fuzzy search |
| **Category chips** | Grocery / personal care / snacks | Browse when barcode fails |

### Price: MRP vs sell

- Packaged goods: printed **MRP is a ceiling**; selling **below** is allowed; selling **above** is prohibited; printed MRP must not be altered ([MoCA LM FAQs Q23–Q25](https://consumeraffairs.nic.in/sites/default/files/file-uploads/latestnews/LM_FAQs.pdf), [Kerala LMD FAQ](https://lmd.kerala.gov.in/faq-frequently-asked-questions/)).
- Same EAN often arrives at a **new MRP**; one sell rate without batch/MRP awareness corrupts margin ([catchus](https://catchus.in/articles/why-the-stock-on-your-screen-stops-matching-the-shelf.html), [BUSY multiple MRP](https://busysoftware.net/busy-software-for-retailers)).
- **Actionable model:** store `mrp`, `sell_rate` (and later `cost` from last purchase). On Sell: default sell_rate; show MRP; block or hard-warn if sell_rate > mrp for packaged SKUs.

### Units & packs

| Pattern | Failure if ignored |
|---|---|
| Sell in g/kg, buy in kg sack | Loose vs packed drift ([catchus](https://catchus.in/articles/why-the-stock-on-your-screen-stops-matching-the-shelf.html)) |
| Buy carton (12), sell piece | Differences of 6/10/12/24 ([stock mismatch](https://richdaddy.in/guides/stock-not-matching-billing-software-fix)) |
| Tax-inclusive MRP billing | Retail B2C expectation ([BUSY tax inclusive](https://busy.in/accounting-software/retail-shop/)) |

**Kirano:** keep unit conversion on item (base unit + pack factor). Buy in pack, sell in pc/kg; stock always in base.

### Low stock

- Per-item reorder level + list on Inventory/Books ([Vyapar low stock](https://vyaparapp.in/free/inventory-management-software/retail), [Marg min/max alerts](https://margcompusoft.com/key_features.html)).
- Negative stock should **alarm**, not silently invent quantity ([stock mismatch](https://richdaddy.in/guides/stock-not-matching-billing-software-fix)) — usually missing purchase.

### Optional catalogue depth (Later)

Batch + expiry (dairy, biscuits), variants (size), self-printed barcodes, Excel import, weighing-scale qty inject.

---

## 4. Money: tenders, khata, AP, GST document types

### Tenders

| Mode | Counter reality |
|---|---|
| Cash | Still majority evenings; need tendered → change ([BUSY auto change](https://busy.in/accounting-software/retail-shop/)) |
| UPI | Show shop QR on settle / print QR on bill ([BUSY UPI QR on bill](https://busy.in/accounting-software/retail-shop/), Kirano `upi_qr`) |
| Card | Often “bank” tender bucket; reconcile later at day close ([BUSY day-end](https://busysoftware.net/busy-software-for-retailers)) |
| **Split** | ₹400 cash + ₹300 UPI on one bill ([Tally Alt+I multi-mode](https://help.tallysolutions.com/point-of-sale-tally/)) |
| Credit / khata | Zero cash now; party AR increases |

### Khata AR

Loop: create/find customer → credit sale → balance → part payment → history/statement → optional reminder ([KOSH digital khata](https://getkosh.co.in/billing-software-for-kirana-store)).  
Keep **line-level bill history** — disputes are “what did I take last week?”, not just a number.

### Supplier AP

Purchase bill → stock + payable; pay later from Books/Buy; supplier aging. Without AP, owners still invent paper khatas for distributors.

### GST document lanes (product words, not rival names)

| Shop status | Document to customers | Source |
|---|---|---|
| Regular GST registered | **Tax invoice** (GST shown); B2C rules for name/address/HSN | [GST Council flyer](https://www.gstcouncil.gov.in/sites/default/files/e-version-gst-flyers/Tax_Invoice_and_other_new.pdf), [B2C rules](https://richdaddy.in/guides/b2c-bill-rules-gst-retail-shop-india) |
| Composition | **Bill of supply** (cannot charge GST) | [DukanUp](https://www.dukanup.com/guides/gst-bill-format-for-shops) |
| Unregistered | **Cash memo** / ordinary bill (no GST) | [mysoundbox invoice vs memo](https://mysoundbox.in/blog/invoice-bill-receipt-small-shop) |
| Small B2C &lt; ₹200, unregistered buyer, no request | May skip individual invoice; need **day-end consolidated** invoice | [CGST FAQ via BIS PDF](https://www.bis.gov.in/other/GSTFAQ200717.pdf), [Rule 46 discussion](https://richdaddy.in/guides/b2c-bill-rules-gst-retail-shop-india) |

**Kirano v1:** Tax invoice + Cash memo lanes (already in product brief). Composition “bill of supply” label can follow. Do **not** force e-invoice IRN for walk-in B2C. Prefer correct rates + serial + GSTIN fields over filing UI.

Rates on packaged FMCG move (e.g. post–Sep 2025 examples on [BillClap GST table](https://www.billclap.com/offline-billing-software/kirana)) — keep rate on item/HSN editable; never hardcode a blog table as law.

---

## 5. Multi-device / LAN till patterns for small shops

| Pattern | How market does it | Kirano fit |
|---|---|---|
| **One Server PC + Client phones/PCs on shop Wi‑Fi** | On-prem retail: static LAN IP, SQL/server process, N billing clients ([GoFrugal system requirements](https://www.gofrugal.com/system-requirement.html), [GoBill intranet vs internet](https://community.gofrugal.com/portal/en/kb/gobill-mobile-sellquick/setup/articles/internet-mode-and-intranet-mode)) | Kumkaj vision: sole-writer Server + LAN Clients — already Kirano lineage |
| **Offline-first, cloud optional** | Bill locally; sync when WAN returns ([GoFrugal hybrid](https://www.gofrugal.com/blog/cloud-vs-offline-billing-software/), [BillBasket offline](https://www.billbasket.app/pos-billing-software/kirana-grocery)) | Local free forever; cloud later usage-billed |
| **Mobile as second till** | Tablet/phone POS for space-constrained shops ([GoFrugal mobile POS](https://www.gofrugal.com/retail/)) | Android Client on same LAN |
| **Multi-outlet head office** | Central MIS, transfer stock ([GoFrugal multi-outlet](https://www.gofrugal.com/retail/), Marg Gold) | **Out** for neighbourhood Kirano v1 |

**Operational rules that prevent pain:**

1. Only Server mutates books; Clients submit ops.  
2. Catalogue/price edits on Server (or owner role) — avoid dual masters.  
3. Prefer shop LAN (intranet) over phone-data VPN for clients ([GoFrugal warns against data-card/VPN](https://community.gofrugal.com/portal/en/kb/gofrugalretaileasy/configurations/barcode-printer/articles/system-requirements-to-use-gofrugal-software-in-retail-business)).  
4. Backup lives off the till disk ([backup guide](https://richdaddy.in/blog/shop-billing-data-backup-india)).

---

## 6. Lessons learned / failure modes of typical India retail POS

| Failure | Mechanism | Design antidote for Kirano |
|---|---|---|
| **Shelf ≠ screen** | Purchases entered late; wrong item name; carton/pc mix-up; unrecorded returns/damage/theft; bad opening stock | ([richdaddy 8 causes](https://richdaddy.in/guides/stock-not-matching-billing-software-fix), [catchus](https://catchus.in/articles/why-the-stock-on-your-screen-stops-matching-the-shelf.html)) → Buy must be faster than “do it Sunday”; barcode over guess; pack conversions; adjust-with-reason; negative stock alarm |
| **Negative stock “fixed” by hand** | Staff force qty up; truth dies | Alarm → open pending purchase / stock take; don’t hide negatives |
| **MRP change same barcode** | Margin averages two worlds | Optional batch/MRP; at least editable MRP + sell with warn |
| **Cloud till dies with internet** | Hotspot flap freezes queue | Offline-first local DB ([BillingKo](https://retail.billingko.in/guides/how-to-choose-kirana-billing-software)) |
| **No backup / only copy on PC** | Disk theft/fire; festival day skipped backup | Auto local backup + restore tested ([KamaiPlus-style cadence](https://kamaiplus.com/help/backup-restore), [backup plan](https://richdaddy.in/blog/shop-billing-data-backup-india)) |
| **Khata as single number** | Disputes; forgotten dues | Bill-linked ledger + part pay |
| **GST theatre, bad serials** | CA cannot file; thermal fades | Stable bill numbers; Tax invoice vs memo; digital copy retained |
| **Slow correct billing → approximate billing** | Staff pick “similar” item | Search/short-code/barcode must beat guessing ([catchus](https://catchus.in/articles/why-the-stock-on-your-screen-stops-matching-the-shelf.html)) |
| **Omnichannel split brains** | App stock ≠ store POS | Out of scope; one books spine if ever added ([commmerce mismatch](https://commmerce.com/blog/how-indian-grocery-chains-fix-stock-mismatch-store-app/)) |
| **Heavy ERP on day one** | Owner abandons after setup hell | Empty catalogue OK; progressive depth; hide unused |

---

## 7. Concrete Kirano backlog (map: In / Have / Later / Out)

Legend (aligned with [`KIRANO.md`](../../apps/kirano/docs/KIRANO.md) §4b):

- **In** — committed product promise; ship / finish for counter-ready.  
- **Have** — already in Kirano (Khaynjo lineage / prior grocery divergence).  
- **Later** — after loop is solid.  
- **Out** — not for neighbourhood Kirano (or not as UI theatre).

| # | Feature | Priority | Notes |
|---|---|---|---|
| 1 | 3-tap Sell: item → qty → pay/credit | **In** | Vision §3 |
| 2 | Search name / short code / alias | **In** | |
| 3 | Qty kg / g / pc / L / ml | **In** | |
| 4 | Khata AR + settle later | **In** | Credit / Parties |
| 5 | Buy → resale stock + supplier AP | **In** / deepen AP UX | |
| 6 | Day book: cash vs UPI vs card vs credit | **In** | |
| 7 | Tax invoice + Cash memo lanes | **In** | |
| 8 | UPI QR on settle / bill | **In** | `upi_qr` |
| 9 | Low-stock visibility | **In** | |
| 10 | Margin-leaning period reports | **In** (deepen) | |
| 11 | Counter-only chrome (no Floor/KOT) | **Have** | |
| 12 | Frequent-items chip | **Have** | |
| 13 | Buy creates resale stock (not kitchen raw) | **Have** | |
| 14 | Optional buy batch + expiry | **Have** | |
| 15 | Barcode / short-code Enter add | **Have** | |
| 16 | MRP + off-MRP hint on Sell | **Have** | Enforce ≤ MRP warn/block as polish **In** |
| 17 | Expiry alerts on Books | **Have** | |
| 18 | LAN Server + Client tills | **Have** (lineage) | Harden shop Wi‑Fi onboarding **In** |
| 19 | Split tender + change | **In** / verify | Books money model exists; grocery UX must be obvious |
| 20 | Pack factor (carton ↔ pc) | **In** | Prevents classic mismatch |
| 21 | Negative-stock alarm (no silent invent) | **In** | |
| 22 | Stock adjust with reason (damage/expiry) | **In** | |
| 23 | Return / credit note against bill | **In** | |
| 24 | Thermal print path (58/80mm) | **Have** / polish **In** | |
| 25 | Local backup + restore | **Have** / polish **In** | Off-disk copy guidance |
| 26 | Staff PIN / roles / audit | **Have** | |
| 27 | Barcode wedge scan-to-add reliability | **Later** | Dedicated wedge path |
| 28 | Hardware weigh-scale inject | **Later** | |
| 29 | Self barcode / shelf label print | **Later** | |
| 30 | WhatsApp/SMS bill + khata reminder | **Later** | |
| 31 | Wholesale rate tiers / schemes / B1G1 | **Later** | |
| 32 | Hold/recall bill | **Later** | |
| 33 | Excel bulk catalogue import | **Later** | |
| 34 | Multi-MRP / batch-wise sell pick | **Later** | |
| 35 | GSTR-1/3B JSON export for CA | **Later** | Export ≠ filing |
| 36 | Accountant XML handoff | **Later** | Optional |
| 37 | Composition bill-of-supply label | **Later** | |
| 38 | Day-end consolidated small-B2C invoice helper | **Later** | Rule 46 edge |
| 39 | E-invoice IRN / e-way in till | **Out** (defer cloud/GSP) | Not B2C kirana v1 |
| 40 | In-app GST portal filing / IMS / notices | **Out** | Compliance theatre |
| 41 | Multi-outlet HO / godown manufacturing | **Out** | |
| 42 | Online marketplace / storefront | **Out** | |
| 43 | Loyalty points / gift vouchers | **Out** / far Later | Supermarket |
| 44 | Voice billing | **Out** unless proven | |
| 45 | Competitor brand names in UI | **Out** | Hard rule |
| 46 | Cloud sync backup | **Later** (usage-billed) | After local loop |

---

## Kirano ship checklist

Use as a release gate for “neighbourhood counter ready” (offline-first Makepad app). Product UI must use Kirano vocabulary only.

### Daily loop

- [ ] Unlock → Books shows today cash / UPI / card / khata owed at a glance  
- [ ] Sell: find item by name, short code, or barcode in one field  
- [ ] Sell: set qty in kg/g/pc/L/ml without leaving the line  
- [ ] Sell: settle cash, UPI (QR), card, **split**, or **khata**  
- [ ] Credit: party ledger with bill lines + part payment  
- [ ] Buy: supplier bill lines raise **sellable** stock; optional AP balance  
- [ ] Day close: tender totals vs drawer; short/excess note; open dues list  

### Catalogue & stock

- [ ] Item has short code, barcode, aliases, category  
- [ ] Distinct **MRP** and **sell rate**; warn/block sell above MRP for packaged goods  
- [ ] Pack conversion (e.g. 1 carton = N pc) on buy/sell  
- [ ] Low-stock list from reorder level  
- [ ] Negative stock surfaces as alarm, not silent success  
- [ ] Stock adjust with reason; return against prior bill  

### Money & documents

- [ ] Tax invoice lane (GSTIN, serial, rates/HSN when configured)  
- [ ] Cash memo / non-GST lane for unregistered or plain memo need  
- [ ] UPI QR available at settle without leaving the bill  
- [ ] Actual / Applied / Rounded money behaviour preserved  

### Devices & resilience

- [ ] Bills continue with WAN down  
- [ ] Second device on shop LAN as Client; one Server writer  
- [ ] Thermal receipt path works on target Windows/Android hardware  
- [ ] Backup + restore documented and one-click from owner role  

### Explicitly not required for first ship

- [ ] No in-app GSTR filing portal  
- [ ] No e-invoice IRN gate on walk-in B2C  
- [ ] No multi-outlet HO, loyalty engine, or online storefront  
- [ ] No competitor names anywhere in product UI  

---

## Source index (URLs)

1. https://retail.billingko.in/guides/how-to-choose-kirana-billing-software  
2. https://getkosh.co.in/billing-software-for-kirana-store  
3. https://khatadar.com/for/kirana-store  
4. https://www.billclap.com/offline-billing-software/kirana  
5. https://www.billbasket.app/pos-billing-software/kirana-grocery  
6. https://vyaparapp.in/free/billing-software-for-retail-shop  
7. https://vyaparapp.in/blog/vyapar-app-features/  
8. https://vyaparapp.in/free/inventory-management-software/retail  
9. https://vyaparapp.in/free/barcode-billing-software  
10. http://www.margsoftwares.com/  
11. https://margcompusoft.com/key_features.html  
12. https://www.gofrugal.com/retail/  
13. https://www.gofrugal.com/blog/cloud-vs-offline-billing-software/  
14. https://www.gofrugal.com/system-requirement.html  
15. https://community.gofrugal.com/portal/en/kb/gobill-mobile-sellquick/setup/articles/internet-mode-and-intranet-mode  
16. https://busy.in/accounting-software/retail-shop/  
17. https://busy.in/busy-features-and-faqs/  
18. https://busysoftware.net/busy-software-for-retailers  
19. https://help.tallysolutions.com/point-of-sale-tally/  
20. https://help.tallysolutions.com/barcodes/  
21. https://tallysolutions.com/tally/tallyprime-for-small-retailers-pos-inventory/  
22. https://www.bharatpos.xyz/  
23. https://magicstep.andro.io/  
24. https://www.gstcouncil.gov.in/sites/default/files/e-version-gst-flyers/Tax_Invoice_and_other_new.pdf  
25. https://www.bis.gov.in/other/GSTFAQ200717.pdf  
26. https://richdaddy.in/guides/b2c-bill-rules-gst-retail-shop-india  
27. https://www.dukanup.com/guides/gst-bill-format-for-shops  
28. https://mysoundbox.in/blog/invoice-bill-receipt-small-shop  
29. https://consumeraffairs.nic.in/sites/default/files/file-uploads/latestnews/LM_FAQs.pdf  
30. https://lmd.kerala.gov.in/faq-frequently-asked-questions/  
31. https://richdaddy.in/guides/stock-not-matching-billing-software-fix  
32. https://catchus.in/articles/why-the-stock-on-your-screen-stops-matching-the-shelf.html  
33. https://richdaddy.in/blog/shop-billing-data-backup-india  
34. https://kamaiplus.com/help/backup-restore  
35. https://commmerce.com/blog/how-indian-grocery-chains-fix-stock-mismatch-store-app/  
36. https://billerpos.com/features (generic offline POS feature checklist cross-check)  
