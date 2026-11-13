# Kirano — Grocery / Kirana Shop Operating System

> **Brand / domain:** Kirano · kirano.shop (kirana counter — grocery &
> neighbourhood shop)
> **What ships:** the Kumkaj grocery product — a concrete business
> operating system for kirana / grocery / general stores, under the
> Kumkaj brand alongside Moklanjo, Khaynjo and Tabiyat.
> **Platform ids:** production `shop.kirano` · development
> `shop.kirano.debug`
> **Naming:** domain **kirano.shop** (not a path under kumkaj.com).
> Platform id is the reverse DNS of that domain: `shop.kirano` / 
> `shop.kirano.debug` — never `com.kirano` and never `com.kumkaj.*`.
> Data dirs, Android application id, iOS/macOS bundle id (when signed),
> and the desktop packager identifier all use this root.
> **Doctrine:** [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](../../../docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
> (concrete products in a common workspace — same Makepad 2, no Kumkaj
> core / shared product code; local-first; free local, usage-based
> cloud). This brief only says what is grocery-specific.

Companions: [`KIRANA_MARKET_RESEARCH.md`](KIRANA_MARKET_RESEARCH.md),
[`KIRANO_WEBSITE.md`](KIRANO_WEBSITE.md),
[`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md), vision §1–2.

**Build rule:** implement this product concretely, in Kirano's own code
only. Never promote logic into shared crates. When another product needs
the same thing, it copies Kirano and diverges.

------------------------------------------------------------------------

## 1. What Kirano is

A complete **kirana / grocery counter** operating system: fast catalogue
sell, khata credit, supplier purchases, stock, day book and GST-facing
bills — without restaurant floor or courier desk chrome.

```text
Sell + credit + day totals
  → categories / short codes / stock / expenses / reports / printing
  → team, permissions, audit, backup
  → extra tills on the shop LAN
  → optional Kumkaj Cloud (later, usage-billed)
```

Venue is **Counter**: no restaurant floor, kitchen tickets, or dining tables. The shop operating system itself stays complete — sell, buy, stock, khata, GST bills, expenses, reports, team, backup, and extra tills on the LAN. Small, medium, and large groceries use the same product; depth is in the books, not in a dining floor.

------------------------------------------------------------------------

## 2. Principles

Kirano follows Kumkaj-wide principles (edit them in root docs once):

- Local-first / free forever locally — vision §§6, 20–21.
- One Server (sole writer) + LAN Clients — vision §20;
  [`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md).
- Triple timestamps / `user_at` day book — `BUSINESS_BOOKS.md`.
- Actual / Applied / Rounded; split tenders; party credit — vision §18.
- India-first (GST, UPI, khata); units —
  [`UNITS_AND_CONVERSIONS.md`](../../../docs/UNITS_AND_CONVERSIONS.md).
- Auditable history — vision §§18–19.
- Identity by UUID — [`IDENTITY_AND_NAMES.md`](../../../docs/IDENTITY_AND_NAMES.md).

Kirano-specific: **dense SKU catalogue**, **kg / piece / litre**,
**supplier AP + customer AR**, margin-leaning day picture.

------------------------------------------------------------------------

## 3. Platform spine (engineering — not the product face)

Kirano owns a full local BOS spine in this crate (store, LAN
Server/Client, fiscal, staff, Makepad shell). That code was started by
**copy-then-diverge** from a sibling app and now evolves only here.
Do not extract a shared core. Do not ship sibling restaurant chrome:

| Area | Where |
|---|---|
| Local store, schema, backup/restore | `src/store.rs` |
| LAN server/client | `src/net.rs`, `src/local_servers.rs` |
| Money, Actual/Applied/Rounded | `src/domain.rs` |
| Country/fiscal packs | `src/fiscal.rs` |
| Staff PIN / roles / audit | `src/store.rs` |
| Makepad 2 shell, nav, locales | `src/main.rs`, `src/locale*.rs` |
| Buy / inventory / UPI QR | `src/store_ops.rs`, `src/upi_qr.rs` |

Venue is `VenueMode::Counter`. Floor / Kitchen / table seed are gated
off and must not appear in owner-facing copy.

------------------------------------------------------------------------

## 4. Kirano-specific state

- Locked product face: `kirano_kirana` (`src/entity_template.rs`,
  `VenueMode::Counter`, Tax invoice / Cash memo). A new shop stays empty until the owner imports the neighbourhood shelf (`assets/items`, `assets/templates/neighbourhood`).
- Parties: **Customer** (khata/AR) and **Supplier** / wholesaler (AP) —
  role labels on one party UUID.
- Shell: **Books · Sell · Buy · Items · Sales · Inventory · Credit ·
  Reports · …** (Floor/Kitchen gated off by Counter venue).
- Brand art: `assets/kirano-app-icon.png` (orange cart + green leaf);
  theme tokens in `docs/DESIGN.md` § Color tokens — Kirano.

------------------------------------------------------------------------

## 4b. What kirana people need — feature map

| User need | Kirano call | Where |
|---|---|---|
| 3-tap billing: item → qty → pay | **In** — vision §3 | Sell desk |
| Search by name / short code / alias | **In** | Catalogue |
| Qty in kg / g / pc / L / ml | **In** | Units |
| Khata credit + later settle | **In** | Credit / Parties |
| Buy from supplier → stock + AP | **In** | Buy |
| Day book: cash vs UPI, who owes | **In** | Books / Day close |
| GST Tax invoice / Cash memo | **In** | Bill lanes |
| UPI QR on bill | **In** | Settle |
| Low-stock visibility | **In** | Inventory |
| Margin-leaning period reports | **In** / deepen | Reports |
| Frequent items chip | **Have** | Sell |
| Counter-only chrome (no KOT / order types) | **Have** | Sell paint |
| Buy creates resale stock (not kitchen raw) | **Have** | Buy / books_op |
| Buy batch + expiry (optional) | **Have** | Buy |
| Barcode / short-code Enter add | **Have** | Sell search |
| MRP + off-MRP hint on Sell add | **Have** | Items form + Sell status |
| HSN on item master (optional) | **Have** | Items form |
| Expiry alerts on Books | **Have** | Books |
| Starter shelf (500+ WebP items, Hindi aliases) | **Have** | Items → Import neighbourhood grocery |
| Barcode wedge scan-to-add | **Later** | Sell |
| Wholesale rate tiers / schemes | **Later** | Items |
| WhatsApp / SMS khata reminder | **Later** | Notify |
| E-invoice / GSTR filing | **Deferred** | Cloud / GSP |

------------------------------------------------------------------------

## 5. Brand & packaging

| | Production | Development |
|---|---|---|
| App id | `shop.kirano` | `shop.kirano.debug` |
| Domain | kirano.shop | — |
| MSI | `apps/kirano/scripts/msi.sh` | `msi.sh dev` |
| Dev | `cargo kirano-dev` | |

Footer line for sites: *Kirano is a Kumkaj product by Gotigin.*
