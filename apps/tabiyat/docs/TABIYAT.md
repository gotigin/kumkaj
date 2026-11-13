# Tabiyat — Pharmacy Operating System

> **Brand / domain:** Tabiyat · tabiyat.shop ("tabiyat" ≈ nature / wellness)
> **What ships:** the Kumkaj pharmacy product — a concrete business
> operating system for chemist / pharmacy counters, under the Kumkaj
> brand alongside Moklanjo, Khaynjo and Kirano.
> **Platform ids:** production `shop.tabiyat` · development
> `shop.tabiyat.debug`
> **Naming:** domain **tabiyat.shop**. Platform id is the reverse DNS
> of that domain: `shop.tabiyat` / `shop.tabiyat.debug` — never
> `com.tabiyat` and never `com.kumkaj.*`.
> **Doctrine:** [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](../../../docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
> §1–2. No Kumkaj core; this crate owns all pharmacy code.

Companions: [`PHARMACY_MARKET_RESEARCH.md`](PHARMACY_MARKET_RESEARCH.md),
[`TABIYAT_WEBSITE.md`](TABIYAT_WEBSITE.md),
[`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md).

------------------------------------------------------------------------

## 1. What Tabiyat is

A **pharmacy counter** BOS: fast medicine sell, barcode / salt search,
batch & expiry bookkeeping, khata, supplier buys, GST bills — offline
first. Not a restaurant floor. Not a regulatory compliance product.

```text
Sell + credit + day totals
  → medicines (barcode, composition, Schedule H / Rx flags)
  → Buy with batch + expiry → stock alerts
  → team, backup, LAN tills
```

Venue is **Counter**: no restaurant floor or kitchen tickets. The pharmacy operating system stays complete — medicine sell, buy with batch and expiry, stock alerts, khata, GST bills, reports, team, backup, and extra tills. Small, medium, and large pharmacies use the same product.

------------------------------------------------------------------------

## 2. Compliance posture (hard rule)

Tabiyat records **what the owner entered**:

- Schedule H / Rx flags are **owner toggles**
- Prescription note is **owner wording** on the sale/item
- Batch / expiry are **shop bookkeeping** and alerts

Tabiyat does **not** claim Drugs & Cosmetics Act compliance, does not
validate prescriptions, does not auto-generate statutory registers, and
does not give medical advice. Drug databases (if ever added) are
optional references, never legal authority.

------------------------------------------------------------------------

## 3. Platform spine (engineering — not the product face)

Tabiyat owns its books, LAN, fiscal, staff, and Sell/Buy spine in this
crate (copy-then-diverge; no shared core). Pharmacy-only fields and
alerts live only here. Sibling restaurant chrome must not appear in
owner-facing copy.

------------------------------------------------------------------------

## 4. State today

- Template `tabiyat_pharmacy`, `VenueMode::Counter`
- Product fields: `barcode`, `composition`, `schedule_h`, `rx_required`,
  Rx note text; search ranks barcode + composition
- Purchase `batch_code` / `expires_on` + `expiry_alert_text` (7-day)
- `barcode_scan` module (image decode fallback; wedge remains primary)
- Theme from `assets/tabiyat-app-icon.png` (teal / lime)
- Starter shelf: `assets/items` and `assets/templates/chemist-counter` (generic names, WebP photos). Not loaded until the owner imports it. Not a drug database.

------------------------------------------------------------------------

## 4b. Feature map

| User need | Tabiyat call |
|---|---|
| Counter sell + khata + GST | **In** |
| Barcode / salt search | **In** |
| Schedule H / Rx flags + note | **In** (owner-controlled) |
| Buy batch + expiry alerts | **In** |
| Scan-to-add (wedge / image) | **Have** — Enter/wedge on barcode, alias or name | Sell search |
| Buy batch UI | **Have** | Buy batch + expiry fields |
| Expiry strip on Books | **Have** | Books |
| Soft near-expiry / FEFO hint on Sell add | **Have** | Sell status (owner practice) |
| HSN on medicine master (optional) | **Have** | Medicines form |
| Expiry alerts window 30 days | **Have** | Books |
| Rx / Schedule H reminder on Sell add | **Have** | Sell status |
| Counter-only chrome | **Have** | Sell paint |
| Starter shelf (500+ generic WebP items) | **Have** | Medicines → Import chemist counter |
| FEFO batch pick at sell | **Later** |
| Generic substitute links | **Later** |
| Statutory Form registers | **Out** (no fake compliance) |

------------------------------------------------------------------------

## 5. Packaging

| | Prod | Dev |
|---|---|---|
| App id | `shop.tabiyat` | `shop.tabiyat.debug` |
| Ports | TCP 7877 / UDP 7878 | same |
| Dev | `cargo tabiyat-dev` | |
