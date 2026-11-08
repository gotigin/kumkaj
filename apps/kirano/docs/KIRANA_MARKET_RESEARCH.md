# Kirano — kirana / grocery market research

> **Scope:** discovery for Kirano only. Competitor **brand names stay in
> this research file** — never on the public site
> ([`docs/WEBSITES.md`](../../../docs/WEBSITES.md) §1).
> **Turned into plan:** [`KIRANO.md`](KIRANO.md) §4b.
> **Updated:** 2026-11-08 (India kirana POS buyers’ guides & feature lists).
> **Deep dive notes:** [`../../../research_notes/Kirano Tabiyat market features/grocery_pos.md`](../../../research_notes/Kirano Tabiyat market features/grocery_pos.md)

## 1. Why this exists

Neighbourhood kirana and small grocery counters need the same books spine
as other Kumkaj products (money in/out, parties, day book) with a **dense
SKU counter** face — not a restaurant floor and not a courier desk.
India-first: GST presentation, UPI, khata credit, supplier dues, offline
billing when the internet drops.

## 2. Evidence used (mechanics only)

Sources consulted (mechanics extracted; brands not for UI copy):

| Source class | What it gave us |
|---|---|
| India kirana / grocery POS buyers’ guides (Fyond, Inzoo, retail ERP blogs) | Sub-2‑minute bills; barcode + **weight (kg)** on one ticket; mixed pack + loose; cash / UPI / khata without leaving sell |
| SMB billing apps (Vyapar-class, ReadyBooks-class, VyapRise-class listings) | Short codes, MRP vs sell, split tender, digital khata integrated with the bill (not a separate diary), shift / Z close, low-stock |
| Local owner loop (field pattern from sibling Kumkaj products) | Walk-in flood, regulars on khata, evening cash count, restock from wholesaler |
| [`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md) grocery row | Dense catalogue, AR + supplier AP, cash + bank, margin/stock/owed day ask |

Representative public pages (research only):

- https://www.fyond.com/blog/best-kirana-grocery-software-india-buyers-guide
- https://inzoo.in/guides/pos-software-for-kirana-store
- https://readybooks.ai/features/pos-billing
- https://vyaprise.com/kirana-store-billing-software/
- https://www.logicerp.com/blog/top-10-best-features-of-kirana-retail-billing-software-solution/

## 3. Owner loop (target)

1. Unlock → **Books** (today’s cash / UPI / owed).
2. **Sell**: search, short code, or barcode → qty (kg/pc) → pay cash/UPI or khata.
3. Regular settles later on **Credit** / party ledger.
4. **Buy** from wholesaler → stock up, optional AP + batch/expiry when useful.
5. **Day close**: tenders vs drawer; who still owes.
6. Extra phone/tablet on LAN as Client till (same books).

## 4. Lessons learned (what good software does / fails at)

| Lesson | Implication for Kirano |
|---|---|
| Khata that is **not** wired into Sell falls behind | Credit settle must be the same bill path as cash/UPI |
| MRP on shelf must match bill line when GST is inside price | Keep MRP optional; show off-MRP on add; fiscal inclusive maths already in books |
| Loose + packed on one bill is non-negotiable | Units (kg/g/pc) + short code + barcode already first-class |
| Cloud-only tills die when the line drops | Local-first + LAN Client is the product promise |
| Enterprise multi-warehouse / e-invoice IRN is noise for one counter | Defer; do not fake GSP filing |
| WhatsApp statements are loved but optional | Later — never block billing |
| Weighing-scale USB capture is common in guides | Later hardware; qty keypad is enough for v1 |

## 5. Module map (must / later / out)

| Module | Must-have (v1) | Later | Out |
|---|---|---|---|
| Sell desk | Search, short code, barcode Enter-add, qty units, split pay, khata | Camera barcode, scale | Restaurant order types |
| Parties | Customer AR + Supplier AP | Credit limit warn, WhatsApp reminder | CRM theatre |
| Buy / stock | Purchase → on-hand, low stock, optional batch/expiry | PO, FIFO valuation | Kitchen raw recipes as primary |
| Bills | Tax invoice / Cash memo, print, UPI QR | e-invoice IRN, HSN packs | — |
| Reports | Day / month, aging, margin vs MRP | GSTR CSV for CA | Auto filing |
| Team / LAN | PIN roles, Server+Client tills | — | Floor/Kitchen peers |

## 6. Decision summary

- Copy-then-diverge from a **retail counter** sibling spine; lock **Counter**.
- Empty catalogue — owner adds SKUs (no restaurant photo shelf).
- Ship MRP + off-MRP hint, barcode, Buy batch optional, Books expiry strip.
- Never claim to be Khaynjo/Moklanjo; never show Floor/Kitchen/KOT as product.
