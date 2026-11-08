# Tabiyat — pharmacy market research

> **Scope:** discovery for Tabiyat only. Competitor names stay here —
> never on the public site. Product plan: [`TABIYAT.md`](TABIYAT.md) §4b.
> **Updated:** 2026-11-08 (India chemist POS feature lists).
> **Deep dive notes:** [`../../../research_notes/Kirano Tabiyat market features/pharmacy_pos.md`](../../../research_notes/Kirano Tabiyat market features/pharmacy_pos.md)

## 1. Why

Chemist counters need the kirana books spine plus **batch/expiry**,
**barcode**, and **salt/composition search**, with Schedule H / Rx treated
as **owner bookkeeping**, not fake statutory software.

## 2. Evidence (mechanics)

| Source class | Mechanics taken |
|---|---|
| India pharmacy POS listings (Alfa Decode, Shopper Pro, KOSH, MyMBill, PharmoTech-class pages) | Batch + expiry on purchase; 30/60/90 near-expiry alerts; FEFO or “sell older first” hints; barcode; salt/generic search; offline billing |
| Local chemist loop | Walk-in flood, Rx note on some sales, khata regulars, wholesaler restock / RTD near-expiry |
| Kumkaj doctrine | Offline bill, GST presentation, UUID identity, **no compliance theatre** |

Representative public pages (research only):

- https://alfadecode.com/pharmacy-software
- https://ingensoftware.in/pharmacy
- https://getkosh.co.in/billing-software-for-pharmacy
- https://www.mymbill.com/products/mymbill
- https://pharmotech.in/services/pharmacy-management

## 3. Lessons learned

| Lesson | Implication for Tabiyat |
|---|---|
| Salt search is as important as brand | `composition` is searchable; Sell status + catalogue search |
| Batch without expiry alerts is half a feature | Buy batch/expiry + Books strip + Sell near-expiry hint |
| Auto “Schedule H1 register” claims are compliance theatre unless lawyered | Owner flags + free-text Rx note only — never claim Drugs Act |
| FEFO auto-pick is expected in premium tools | Soft hint first (**Have**); hard FEFO batch pick = **Later** |
| Drug master DB of 200k SKUs is a vendor lock-in play | Empty catalogue; owner enters what they stock |
| Blocking expired stock at bill is valuable | Later (needs line-level batch on every sale) |
| Offline is table stakes | Already: local store + LAN Client |

## 4. Decision summary

- Copy Kirano counter face; pharmacy deltas only in this crate.
- Never claim Drugs Act compliance or auto Form registers.
- FEFO at sell and drug DB import stay **Later**; soft near-expiry is **Have**.

## 5. Module map

| Need | Call |
|---|---|
| Counter sell + khata + GST | **In** |
| Barcode / salt search | **In** / **Have** |
| Schedule H / Rx owner flags + note | **Have** |
| Buy batch + expiry + Books alerts | **Have** |
| Soft near-expiry / FEFO practice hint on Sell | **Have** |
| Hard FEFO batch pick / block expired | **Later** |
| Generic substitute graph | **Later** |
| Statutory Form registers | **Out** |
