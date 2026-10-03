# Kumkaj — User experience

This document describes how the product should feel for end users. It is not a coding guide.

Kumkaj is one product that scales by **capability**: a tea stall or clothes counter stays fast and uncluttered; a food shop can turn on tables and kitchen; an owner with many outlets manages an **organization** of branches. Progressive complexity hides what you do not need — it does not withhold it behind a paywall. Book-like flexibility (amend, undo, credit by name, round a bill) matters at every scale.

---

## First open

Asked **once**, then remembered:

1. **This device is the shop** — this phone or computer holds the books (device role `server`). Extra tills stay **off** so LAN does not start. A future cloud drain syncs these shop books; there is no separate standalone role.
2. **This is an extra till** — pair to the shop device (device role `client`).

| Later, in Backup & settings | Meaning |
|---|---|
| **Allow extra tills** | Start LAN and show a pairing code. Other phones join as extra tills. |
| **Stop extra tills** | Stop accepting joins. This device is still the shop. |

LAN pairing uses the outlet Wi‑Fi: the shop shows a **6-digit pairing code**. The extra till enters the shop IP + code. Trust model = same local network; there is no cloud requirement for ordinary billing.

---

## Shop — first open (Owner)

1. Only **Owner** exists. Owner sets a PIN (4–12 digits).
2. Onboarding: optional name of this place (and the organization if you run more than one outlet), then either:
   - **Start fresh** — empty catalogue, or optional example items; or
   - **Bring my book** — type products with prices and customers with opening credit balances (explicit “opening balance” ledger entries, not silent fudge fields).
3. Tables & kitchen stay **off**: sales do **not** create kitchen tickets. Floor, Kitchen, order types, and Send KOT stay **hidden** until Owner turns on **Tables & kitchen** in Settings.
4. A **Main** branch is created automatically. **Branches & org** stays out of More until a second branch exists (open it from Settings).

Owner is not a job title. The same person may sell and own the business. Solo use with no staff must stay excellent.

---

## Selling (every scale)

1. Tap items (sold-out and out-of-stock tracked items are blocked; Owner/manager can mark **Sold out / Restock** or manage stock under Inventory).
2. When **Tables & kitchen** is on, choose **Counter**, **Dine-in**, or **Takeaway** if it matters.
3. Adjust qty, `@` unit price, line Applied amount, line note, or bill **Rounded Total**.
4. Pay **Cash / Card**, the local instant rail, **Split Cash + rail**, or **Credit** with a customer name (no CRM required). The rail button is named for your country — UPI in India, Pix in Brazil, PayNow in Singapore, FPS in Hong Kong, Interac in Canada, and so on.
5. See confirmation + optional **Undo** for a few seconds.
6. Day strip shows cash / rail / card / credit today and **who still owes**.
7. Later: settle credit without inventing a fake sale. **Day close** (summary + **Record day close** audit), **Reports & expenses**, and (for multi-outlet owners) **Branches & org** consolidated roll-up under More.

Where prices include tax, the totals strip is what the customer pays. Where tax is added at the till, a line under the strip shows the tax and the real amount to collect before you take payment.

Receipts: full text after pay (strip + Sales → **Copy receipt** / **Print / open**), including a sequential bill number (`INV-MAIN-000042`). A split bill lists each tender and its amount. The tax breakout follows your country: **Net / tax / Total** when prices include tax, **Subtotal / tax / Total** when it is added, and named components (India's CGST + SGST) where a country bills one tax as halves. Your business id prints at the top once you save it. Desktop copies to clipboard and opens/prints a temp file. Android **Copy receipt** uses the system clipboard, and **Print / share** opens the system share sheet. A photo picked from the gallery is copied into the app before it is saved.

---

## Menu (Items)

- Name, price, optional photo.
- Optional **category** (created on the fly).
- Optional **kitchen station** (Hot / Cold seeded; add more as needed) — shown when Tables & kitchen is on; tickets split per station.
- Optional **tax rate** — type a rate from your country pack (`Standard`, `GST 18%`, `Reduced`) or a bare percent (`0`, `5`, `12`, `20`). A percent you have not used before becomes a named rate you can reuse.
- Optional **modifiers** (`Extra cheese=20, Less sugar=0`). On the cart line, tap a chip to add its price and send the name to the kitchen. Tap again to take it off. A free-text **note** is still there for anything that is not on the list.

---

## Country & tax

Under More → Backup & settings → **Country & tax**:

- Type a country code (`IN`, `GB`, `US`, `AE`, `BR`, `SG`, `ZA`…) — or walk the list with **‹ Browse** / **Browse ›**, which fills the code and previews what the pack changes — then tap **Apply country pack**. That sets the tax name and how it is calculated, seeds the usual rates, names the instant-pay rail, labels your business id (GSTIN / VAT number / ABN / EIN / TRN / RFC / CNPJ…), and moves currency and language to the country's defaults — all still editable afterwards.
- Applying a pack keeps any rate an item still uses, so switching country never strands a product without its tax.
- **Tax inside price** vs **Tax added at till** can be overridden without leaving the pack, for shops that price against the local norm.
- Money is written your country's way: `₹1,234.50` in India, `1.234,50 €`-style commas across most of Europe and Latin America, `1'234.50` in Switzerland, spaces in France and Sweden. Typing is forgiving in both directions — `13,5` and `13.5` mean the same thing wherever you are.
- Save your **business tax id** once and it prints on every bill.
- Optional **service %** (blank or `0` = off). When set, it is added on top of the lines, shown on the Sell strip before you charge, and printed as its own **Service** line on the bill. No country pack turns it on for you.

Kumkaj does the arithmetic and the wording. It does **not** file returns, produce government e-invoices, or drive fiscal printers anywhere in the world — the rates you enter and your filings stay yours.

---

## Staff and credentials

- Owner adds people with a name, role title, permission preset (**Cashier** / **Manager** by default; **Waiter** / **Kitchen** when Tables & kitchen is on), and a starting PIN.
- Cashiers who cannot void/refund need a **Manager PIN** on the Sales screen to approve; large bill round-offs (>10%) need the same.
- Owner may also set an optional login id + starting password **once**. After that, only that person can change their own PIN or password.
- Unlock with PIN **or** login id + password.
- Owner can toggle feature access (Sell, Items, Sales, Credit, Floor, Kitchen, Inventory, Branches, Team, Backup) per staff member.
- Tabs and More rows the person cannot use are hidden or blocked.

---

## Extra till

After pairing:

- Unlock as **Staff** (preferred) or Owner (Owner PIN matches the shop after sync), or login id + password.
- Catalog, categories, stations, customers, credit balances, recent sales, open kitchen tickets, and tables come from the shop device.
- Sales, amend, Send KOT, credit settlements, undo, void, refund, and kitchen Advance are recorded on the **shop** (LAN). The extra till never owns the books.
- Items, inventory, branches, team, and fiscal settings are shop-only (the extra till shows a clear message if you try).
- Split Cash + rail is recorded on the shop, including from a paired extra till. Both tenders are written with the sale.
- Inventory / Branches rows stay hidden on an extra till.

---

## Tables & kitchen

In Settings → **Tables & kitchen**:

- Sales create kitchen tickets (with table name when seated), split by kitchen station when items are assigned.
- **Send KOT** can fire the kitchen before payment.
- Tables hold a parked cart when you switch tables (open-tab draft). Paying clears the table.
- Kitchen → Advance moves ticket status; cycle **station filter** for a busy multi-station pass (also works from a paired extra till against the shop).

Tables & kitchen stays off until a food business needs it.

---

## Inventory & stock

Under More → **Inventory & stock** (Owner / Manager by default):

- Start tracking an item, adjust qty with a reason, log purchases (supplier + unit cost).
- Low-stock list and recent movements stay visible.
- Tracked stock decrements on sale; untracked items (`−1`) behave like a pure menu (sold-out flag only).

Recipe / BOM production and forecasting remain deeper roadmap; foundational stock, purchases, adjustments and wastage-as-adjustment are in the local product.

---

## Branches & organization

Under More → **Branches & org**:

- Name the **organization**; add branch codes (e.g. MAIN, INDIRANAGAR, WHITEFIELD).
- **Switch** the active branch on this device (menu, tables, sales and stock are branch-scoped).
- **Export** a daily snapshot JSON (clipboard + `backups/branch_snapshot_{CODE}_{date}.json`).
- **Import** another outlet’s file as `backups/import_branch_snapshot.json` to register / refresh a remote branch.
- **Consolidated** report rolls up this device’s live day with imported branch snapshots (marks stale / missing branches).

Each physical outlet still runs its own free local Server. File snapshots coordinate the group without Gotigin Cloud. A durable **sync outbox** is written on every sale so optional WAN / Kumkaj Cloud sync can attach later without rewriting the books. Hosted cloud dashboard, automatic offsite backup, and metered WAN sync remain optional paid infrastructure — never required to bill a guest.

---

## What we refuse to become

- A cloud-only till that dies offline.
- Software that silently rewrites history or blocks tiny book-like corrections.
- A free product that withholds core local POS, restaurant, inventory or multi-branch **local** capabilities to force a subscription.
- An enterprise ERP forced onto a tiny shop on day one.

Local use stays complete at every scale described above. Gotigin Cloud (when used) is optional and usage-based. Statutory **e-invoicing** (Indian GST IRN, EU/LatAm clearance, fiscal printers), raw **ESC/POS** thermal drivers, and hosted WAN dashboards are called out honestly where not yet claimed — country tax packs and system print/share are available now.
