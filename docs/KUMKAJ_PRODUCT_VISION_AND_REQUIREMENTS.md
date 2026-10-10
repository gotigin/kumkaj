# Kumkaj --- Product Vision & Requirements

> **Product name:** Kumkaj (*business*)\
> **Brand / company:** Gotigin\
> **Document purpose:** Consolidate the important product ideas,
> requirements, principles, scope, and scaling model for Kumkaj so
> future development stays focused on the actual product rather than
> development tooling.\
> **Decision rule:** When earlier edition or pricing plans conflict with
> later finalized decisions in this document, **the later decision
> wins**. In particular, Community / Professional trial / annual
> entitlement pricing is superseded by free-forever local use and
> usage-based cloud billing.

------------------------------------------------------------------------

## 1. What Kumkaj Is

Kumkaj is intended to be much more than a billing screen. It is a
**business operating system** for **any entity** that sells or keeps
books — a business, a personal or household set of books, a service
trade, an agency, or a multi-branch organization. The same fundamental
product serves them all. Examples of *users of one OS* (not product
SKUs or vertical forks):

-   a roadside **chai wala / tea seller** who may only need extremely
    fast item selection, payment and a simple daily total;
-   a **clothes seller**, **kirana / grocery**, or other retail counter
    with a catalogue, sizes, and stock;
-   a **service** counter (barber, repair, salon) with appointments or
    walk-in sales and party credit;
-   a **courier / logistics agency** or similar intermediary with named
    parties, quantity units (e.g. kg), day books and outstanding;
-   a small takeaway, snack shop, bakery or café;
-   a single restaurant with tables, waiters, kitchen tickets, stock and
    expenses;
-   a busy large restaurant with multiple counters, kitchen stations,
    managers and many employees;
-   **personal / household** books where the entity is a person or
    family rather than a shop front;
-   an owner operating **multiple shops or branches**, who needs
    centralized visibility, controls, reporting, synchronization and
    administration.

The app must **not encode the user's trade**. Implementation stays
generic (parties, lines, units, charges, ledgers, day books, documents).
Naming, configuration, and which capabilities the entity enables make
Kumkaj *feel* concrete to that user.

The central idea is therefore:

> **One Kumkaj, progressively adapting to the complexity of the
> entity instead of forcing every user into enterprise-level
> complexity — generic in the core, concrete in effect.**

A tea seller or clothes counter should not have to understand kitchen
routing just to sell. At the other end, a large restaurant group must
not outgrow Kumkaj simply because it started with a simple till.
Food-only tools (tables, KOT, kitchen stations) stay available as
**Tables & kitchen** — an optional capability pack, not the identity of
Kumkaj. Extra tills are the same shop device with LAN on — not a
different product type for cloud.

------------------------------------------------------------------------

## 2. Product Philosophy: Simple at the Bottom, Powerful at the Top

Kumkaj should scale by **capability**, not by turning the
smallest interface into a miniature enterprise ERP, and not by
shipping a separate product per trade.

The smallest entity may effectively see:

**Items → Sell → Payment → Receipt / Done**

A food business may additionally enable:

**Tables → orders → KOT/KDS → Kitchen → Billing → Inventory →
Expenses → Reports**

A multi-branch owner may additionally enable:

**Organization → Branches → Users/Devices → Central Dashboard →
Cross-Branch Reports → Sync/Backup → Remote Monitoring**

Features that are irrelevant to a particular entity should remain out
of the way.

This is a key product principle: **progressive complexity**. Kumkaj
should feel small when the entity is small and become powerful as the
entity grows.

**Client signal → shared capability:** when a client describes something
in vertical language (courier parties, kg credit, salon chairs, household
expense buckets), treat that as a signal for a **shared primitive**
(party, unit on a line, ledger, day book, charge, document). Implement
the primitive; let the entity's labels and defaults provide the concrete
feel. Do not fork a trade-specific schema or “mode” that duplicates the
core.

------------------------------------------------------------------------

## 3. The Chai Wala Test

Every major product decision should be checked against a very small
seller.

Imagine a tea seller with a handful of products:

-   Tea
-   Special tea
-   Coffee
-   Water
-   Snacks

The seller may not need tables, inventory recipes, employees, kitchen
screens or cloud dashboards. They **do** often need a very small form of
customer memory: who bought on **credit**, how much is
outstanding, and when that person paid later.

The useful cash workflow is simply:

1.  Tap an item.
2.  Change quantity if needed.
3.  See line and bill totals immediately (**Actual** / **Applied** on
    each line; **Actual** / **Applied** / **Rounded** on the bill).
4.  Optionally adjust a line's `@` unit price, that line's **Applied**
    amount, or the bill **Rounded Total** before paying (catalogue price
    stays the default; `@` is never rewritten by a line Applied edit).
5.  Record cash/UPI/other payment **or** put the sale on a customer's
    credit.
6.  Finish the sale — the **recorded history** of that moment is kept
    forever; the owner may later amend the live order with a tracked
    revision (who / when / why / previous values).
7.  Optionally print a receipt.
8.  At the end of the day, see sales, payment totals, and **who still
    owes money**.
9.  When a credit customer pays later, record the settlement against
    that balance without inventing a fake new sale.

This mode must remain **fast and understandable without training**.
Credit must not force a full CRM setup: a name (and maybe a phone note)
should be enough to start a credit account.

The system should not require an internet connection for an ordinary
sale. It should not make a tiny seller configure enterprise concepts
before the first bill.

------------------------------------------------------------------------

## 4. The Large Restaurant Test

The same underlying product must support a substantially more complex
operation.

A large restaurant may need:

-   dine-in and takeaway orders;
-   table management;
-   waiter/cashier workflows;
-   menu categories, variants/modifiers and pricing;
-   KOT generation;
-   kitchen display/status workflows;
-   different kitchen stations;
-   order status tracking;
-   discounts with permissions;
-   cash, card, UPI and split-payment recording;
-   receipts and invoices;
-   staff roles;
-   inventory and purchases;
-   stock movements and adjustments;
-   expenses;
-   daily closing and operational reports;
-   manager approvals and audit history;
-   backup and recovery.

The interface should expose these capabilities when they are needed
without damaging the speed of the POS workflow.

------------------------------------------------------------------------

## 5. The Multi-Restaurant / Multi-Branch Test

Kumkaj must have an architecture that can grow beyond one physical
location.

A restaurant owner with several branches should eventually be able to
manage an **organization** containing multiple branches/outlets.

Important capabilities include:

-   branch selector;
-   branch-specific menu/pricing where required;
-   users and devices associated with appropriate branches;
-   centralized owner dashboard;
-   branch-level sales;
-   consolidated sales across branches;
-   cross-branch operational reporting;
-   synchronization status;
-   backup status;
-   remote read-only operational visibility;
-   device management/revocation;
-   role and permission management;
-   branch health/alerts;
-   centralized API access where appropriate.

WAN coordination, remote dashboards, and automatic offsite backup are
**Kumkaj Cloud** capabilities and are billed by cloud usage. Each
branch must still be able to run its full local POS without the cloud.
A cloud outage must **not stop a restaurant from billing locally**. Each
operational location should remain capable of continuing service and
synchronizing safely later.

------------------------------------------------------------------------

## 6. Local-First and Offline-First Operation

Reliable local operation is one of Kumkaj's strongest requirements.

Normal restaurant service must continue without the cloud or internet. A
network failure must not prevent:

-   creating an order;
-   modifying an active order according to permissions;
-   sending/maintaining kitchen work locally;
-   recording payment;
-   creating the final invoice/receipt;
-   accessing the local menu and required operational data;
-   viewing relevant local reports.

Completed orders and invoices must survive:

-   application restart;
-   machine restart;
-   temporary internet failure;
-   synchronization failure;
-   recoverable migration failure;
-   power interruption scenarios for which the local database has
    already durably committed the transaction.

Cloud functionality should enhance Kumkaj, not become a dependency for
serving a customer standing at the counter.

------------------------------------------------------------------------

## 7. Core POS / Ordering

The POS is the operational heart of Kumkaj.

It should support a fast flow for common operations:

-   start order;
-   choose dine-in/takeaway as applicable;
-   choose table where applicable;
-   search or tap menu/product items;
-   quantities;
-   **optional per-item notes** during the sale (e.g. "less sugar",
    "extra spicy") — never required;
-   **per-sale unit price edits** on cart lines (catalog price is only the
    default — the seller may charge a different `@` for this bill);
-   **per-line Actual vs Applied** — Actual is always `@ × qty`
    (read-only); Applied is editable and defaults to Actual; editing
    Applied never rewrites `@`;
-   **three bill totals** — Actual, Applied, and Rounded (see below);
-   modifiers/options;
-   order-level notes where useful;
-   taxes/configurable charges;
-   discounts subject to permissions;
-   order state;
-   payment (cash / UPI / card / **Credit**);
-   invoice/receipt.

### Lenient cart, durable history (not silent rewrites)

Small sellers routinely negotiate or round at the counter. Example:

-   4 × tea at ₹13 → line **Actual** ₹52;
-   seller may change `@` for this sale, or set this line's **Applied**
    (e.g. ₹50) without changing `@`;
-   seller may also set bill **Rounded Total** (e.g. ₹50) before
    recording payment.

These edits apply only to the **open cart**. The catalogue/menu price is
not rewritten by a one-off cart change unless the seller edits the item
in Items.

### Line totals (two values per cart line)

Always visible on each cart row:

-   **Actual** — `@ × qty` (true item math; read-only). Changing `@` or
    qty recalculates Actual and clears any line Applied override.
-   **Applied** — what this line contributes to the bill. Defaults to
    Actual. The seller may edit Applied (e.g. round this line to ₹50).
    **`@` is never rewritten** when Applied is edited.

Line-level difference (for transparency; not a separate DB column today):

> `line Applied − line Actual` (negative = round down on that line)

### Bill totals (three values on the open cart)

Always visible at the bottom of the cart:

-   **Actual Total** — sum of every line's **Actual** (`@ × qty`).
    Read-only.
-   **Applied Total** — sum of every line's **Applied**. Read-only on the
    bill (edit lines to change it). This is the bill before seller
    round-off.
-   **Rounded Total** — amount collected / written to the sale record.
    Editable. Defaults to Applied Total. The seller may edit it to round
    the whole bill (e.g. ₹52 → ₹50).

Bill-level rounding is stored explicitly so books stay reconcilable:

> `Applied Total (lines_total) + rounding = Rounded Total (total recorded)`

When the seller rounds ₹52 → ₹50 at bill level, Kumkaj must **not**
silently rewrite tea to ₹12.50. It must keep the true line amounts and
store an explicit **rounding adjustment** of −₹2, with **Rounded Total**
₹50.

A completed sale therefore retains at least:

-   each line's unit price (`@`), qty, **Applied** line total, and
    optional note;
-   `lines_total` — sum of line Applied (= bill Applied Total);
-   `rounding` — bill-level adjustment (`Rounded − Applied`; may be
    negative or positive);
-   `total` — amount actually collected (= **Rounded Total** =
    `lines_total + rounding`).

Day/owner visibility should be able to show money collected (Rounded),
gross from lines (Applied), Actual where useful, and rounding separately
so accounts can reconcile.

Window chrome (Wayland/Windows client-side decorations) uses cream
caption controls on the navy UI so min/max/close stay visible.

### What “immutable” means

**Immutable** means prior records and revision history are never erased or
silently overwritten. It does **not** mean the owner is locked out of
fixing mistakes.

After a sale is recorded:

-   the **current** order may be amended by the owner (and, where
    permitted, a manager) — lines, `@` prices, line Applied, bill
    Rounded Total / rounding, payment method, etc.;
-   every amendment must append a **revision history** entry with at
    least: previous snapshot, new values (or a full before-state),
    **who** changed it, **when**, and a **reason**;
-   void/refund remain available as stronger corrections and also leave
    an attributable trail;
-   there is no silent in-place rewrite without history.

So: history is append-only and trustworthy; the live order stays
fixable under permission.

A trained cashier should be able to begin an order, add common items,
tweak price/total if needed, record payment and produce a receipt
**without navigating through settings**.

Touch, keyboard and mouse operation should all be considered.

Repeated actions should feel immediate. The POS should prioritize
operational speed over decorative complexity.

------------------------------------------------------------------------

## 8. Tables and Dine-In

For restaurants that use tables, Kumkaj should support a complete
table/order lifecycle.

Conceptually:

-   available table;
-   occupied/active table;
-   current order;
-   additions to an existing table order;
-   send new items to kitchen;
-   settlement;
-   table becomes available again.

Table functionality should be optional. A chai seller or takeaway shop
should never have to interact with it.

------------------------------------------------------------------------

## 9. KOT and Kitchen Display System (KDS)

Kitchen operation is a major restaurant capability.

Kumkaj should support kitchen tickets/KOT and a KDS-style workflow so
orders can move from counter/waiter to preparation without relying
entirely on paper.

Kitchen users should see the information needed to prepare the order
rather than unnecessary financial information.

Typical states may include concepts such as:

**New → Preparing → Ready → Completed**

The exact workflow can evolve, but kitchen status must remain
operationally clear.

Future growth can include intelligent routing to different kitchen
stations/printers/displays.

------------------------------------------------------------------------

## 10. Menu and Product Management

Menu management should cover the needs of simple and complex businesses.

Potential concepts include:

-   products/items;
-   categories;
-   price;
-   tax configuration;
-   variants/modifiers;
-   combos;
-   availability;
-   archive/deactivate instead of destructive deletion where history
    depends on an item.

A small seller should be able to create a tiny menu quickly. A large
restaurant should be able to organize a large catalog without changing
the fundamental product.

------------------------------------------------------------------------

## 11. Payments and Billing

Initial payment scope is intentionally practical:

-   cash;
-   card as a recorded payment method;
-   UPI as a recorded payment method;
-   **credit** against a customer account;
-   split payments (including partly cash/UPI and partly credit).

Kumkaj does **not need to store card data** merely to record that a
bill was paid by card.

Putting a sale on credit is **not** the same as a completed cash-like
settlement: it records that goods/service were given and that money is
still owed. Later settlement of that debt must update the customer's
outstanding balance and remain auditable.

The first system should not make live payment-gateway settlement a
dependency for normal billing.

Invoices and finalized financial records must be durable and traceable.
A finalized invoice/payment should not simply disappear because someone
edited a database-backed screen.

Corrections, voids and refunds should be represented explicitly, with
permissions/reasons/audit history as appropriate. Credit balance
adjustments (settlements, write-offs) must be equally explicit.

------------------------------------------------------------------------

## 12. Receipts and Printing

Receipts are part of the real-world POS workflow.

The initial strategy discussed is:

-   support a known/tested thermal printer path;
-   provide normal system printing/PDF fallback;
-   use a controlled branded receipt/invoice layout;
-   expand hardware-specific integrations progressively.

Hardware already considered for the local POS environment includes an
**80 mm thermal receipt printer** (Rugtek RP-82 Compact UE, USB +
Ethernet) and 80 mm thermal paper.

Native raw ESC/POS support, cash-drawer integration and broader hardware
support can be added after the core workflow is stable rather than
blocking the first usable product.

------------------------------------------------------------------------

## 13. Barcode, Labels and Other Hardware

The broader POS direction includes hardware such as:

-   barcode/QR scanner;
-   receipt printer;
-   label printer;
-   KOT printer/display;
-   eventually cash drawer;
-   eventually weighing scale;
-   eventually customer-facing display.

Hardware should remain modular. A restaurant should not require hardware
that its business model does not use.

Devices previously considered for demonstrations/local deployment
include a PX40 wireless 1D/2D scanner and 4×6 label-printer options.

------------------------------------------------------------------------

## 14. Customers, Credit and CRM

For many of India's smallest food sellers, **customer credit is not a
nice-to-have CRM feature** --- it is everyday business. A tea seller
often has regulars who take tea/snacks on credit and settle later
(daily, weekly, or when they can). Kumkaj must support this as a
**core local POS capability**, not only as a later restaurant CRM
module.

### Minimum credit capability (chai-wala scale)

Even without a rich customer module, the seller should be able to:

-   keep a short list of credit customers (name is enough; phone/notes
    optional);
-   finish a sale as **credit** against a chosen customer;
-   see that customer's **outstanding balance**;
-   see a simple list of **who owes how much** (today / all open);
-   record a later **settlement** (cash/UPI/card) that reduces the
    balance;
-   keep an attributable history of credit sales and settlements so
    totals remain trustworthy.

This must stay inside the **Chai Wala Test**: few taps, no training, no
mandatory cloud, no enterprise customer-master setup.

Credit sales still count as real sales for operational visibility, but
**cash/UPI/card day totals and credit outstanding must not be confused**.
The seller needs both: what came in today, and what is still owed.

### Broader customer / CRM (later / larger businesses)

As the business grows, customer management can additionally include:

-   richer profile/contact information where the business chooses to
    collect it;
-   order history beyond credit;
-   notes;
-   future loyalty/reward capabilities;
-   offers/feedback as later capabilities.

Kumkaj should collect only information actually required by the
business and should not make customer registration mandatory for an
ordinary **paid** counter sale. Credit, however, inherently needs a
customer identity --- keep that identity as light as possible.

------------------------------------------------------------------------

## 15. Inventory, Purchases and Expenses

A business operating system should eventually connect sales to the
broader operation.

Core operational areas include:

-   inventory items;
-   stock movements;
-   purchases;
-   suppliers/workflows as the product grows;
-   low-stock visibility;
-   stock adjustments;
-   expenses;
-   wastage;
-   recipe/ingredient-based deduction as a deeper restaurant capability.

Stock corrections should be traceable. More advanced recipe production,
forecasting and supplier automation can evolve after the foundational
inventory model is trustworthy.

------------------------------------------------------------------------

## 16. Reports and Owner Visibility

Useful reporting should answer operational questions rather than merely
display charts.

Local reports can include:

-   daily sales;
-   payment-method totals (cash / UPI / card / credit);
-   **lines Actual / Applied vs Rounded totals vs rounding adjustments**
    (so line overrides and bill round-offs remain reconcilable);
-   open credit outstanding (by customer and total);
-   credit settlements collected today;
-   tax totals;
-   item/product sales;
-   order counts;
-   average bill;
-   expenses;
-   stock/adjustments;
-   cash-session/day-close information.

For multi-branch businesses, the owner should additionally be able to
view branch-specific and consolidated information.

The dashboard should adapt to the scale of the user. A chai seller may
only care about today's sales, cash/UPI totals, and **who still owes on
credit**; a restaurant-group owner may need branch comparisons,
device/sync health and consolidated operational data.

------------------------------------------------------------------------

## 17. Staff, Roles and Permissions

Initial role concepts discussed include:

-   **Owner** --- organization, branches, finance, users, settings and
    exports;
-   **Manager** --- controlled discounts, voids, stock adjustments and
    reports;
-   **Cashier** --- orders, payments and permitted customer actions;
-   **Kitchen** --- kitchen-order/status operations without unnecessary
    financial access.

Staff PINs are useful for fast local POS access.

Sensitive actions such as high discounts, refunds, voids or stock
corrections should be permission-controlled and recorded with a
reason/audit event where appropriate.

------------------------------------------------------------------------

## 18. Auditability and Business Trust

Kumkaj handles money and operational records, so silent destructive
mutation is dangerous.

Important principles:

-   completed financial transactions are durable;
-   prior versions must never be silently deleted or rewritten without a
    trail;
-   cart leniency (edited `@`, line Applied, bill Rounded Total) is
    allowed on the open cart before finalize;
-   after finalize, the **owner may amend** the live order, but every
    amendment appends history (who, when, reason, previous snapshot);
-   void/refund/credit settlement remain explicit, attributable paths;
-   corrections should have an explicit history;
-   sensitive operational mutations should be attributable;
-   manager/owner approval should be meaningful, not merely a cosmetic
    UI flag;
-   the owner should be able to understand why totals changed and who
    changed them.

This is particularly important as Kumkaj moves from a one-person shop
to a business with cashiers, managers and multiple branches.

------------------------------------------------------------------------

## 19. Backup, Restore and Recovery

A backup is not useful merely because a file exists. Kumkaj should have
a **verified restore path**.

Requirements include:

-   safe local backup;
-   restoration onto a clean installation;
-   migration discipline;
-   data-integrity checking;
-   recovery after interrupted operations;
-   no dependence on cloud availability for local operation.

When a business uses Kumkaj Cloud, automatic encrypted cloud backups can
supplement---not replace---the local reliability model. Cloud backup is
part of paid cloud usage, never a requirement for local operation.

------------------------------------------------------------------------

## 20. Cloud and Synchronization

Cloud capabilities are optional enhancements for businesses that need
remote visibility, automatic offsite backup, multi-location
coordination over WAN, or other Gotigin-hosted infrastructure.

Potential cloud responsibilities include:

-   organization/branch identity;
-   user/device management;
-   synchronization;
-   owner dashboard;
-   cross-branch reports;
-   backup;
-   remote health monitoring;
-   authenticated APIs;
-   usage metering and billing for the resources above.

Local multi-device coordination over a restaurant's own LAN does **not**
require Gotigin Cloud and must remain available as part of free local
operation.

Synchronization must be designed for unreliable networks:

-   local commits happen first;
-   changes are queued durably;
-   retry is safe;
-   duplicate retries must not create duplicate financial transactions;
-   sync state should be visible;
-   unresolved conflicts should not silently corrupt financial history.

------------------------------------------------------------------------

## 21. Commercial Model --- Free Local, Usage-Based Cloud

### Final commercial decision (overrides earlier edition plans)

Earlier planning used **Community / Professional Evaluation
(14-day trial) / Professional Paid / Enterprise** entitlement tiers.
**That model is superseded.**

The finalized commercial identity for Kumkaj is:

> **Kumkaj is free to run locally, forever. Use the local product fully.
> If you want Gotigin Cloud, pay only for the cloud resources you
> actually consume. Same rates for everyone.**

Core principle:

> **Software capability is not the thing being sold. Infrastructure
> consumption is.**

Gotigin does not decide who is “small” or “enterprise” for pricing.
A chai seller and a large restaurant group see the **same price sheet**.
The large group simply consumes more cloud resources and therefore pays
a larger total bill.

There is **no 14-day Professional trial**, no annual Pro/Enterprise
entitlement as the primary commercial product, and no artificially
crippled free tier.

### Kumkaj Local --- Free Forever

Anyone may install and run Kumkaj offline / on their own devices and
local network at **₹0 recurring software or cloud cost**.

Local use must be **genuinely complete**, not a demo. Free local
operation must not withhold core restaurant capabilities merely because
the customer is not paying. Progressive complexity still hides
irrelevant UI, but **functionality is not gated by payment**.

Free local capabilities include (as implemented over time):

-   POS, menu, **customer credit**, customers, tables
    where needed;
-   orders, invoices, receipts;
-   employees / local roles and permissions;
-   inventory, purchases, expenses;
-   reports;
-   KOT / KDS;
-   local backup / restore;
-   multi-device operation over the restaurant's own LAN;
-   other local-only capabilities that do not consume Gotigin
    infrastructure.

Do **not** charge for things that cost Gotigin essentially nothing to
host, such as number of employees, menu items, tables, local orders,
local devices on a LAN, inventory features, KOT, or local reports.

A restaurant may run locally for years, never pay Gotigin, and still
have a functioning system. Local data remains owned by the restaurant.

### Kumkaj Cloud --- Paid by Usage, Same Rates for All

Cloud is optional. Customers pay when they use Gotigin-hosted
infrastructure, for example:

-   remote access / owner dashboard;
-   cloud synchronization across locations over WAN;
-   automatic encrypted cloud backup;
-   cross-branch cloud reporting and remote device/sync monitoring;
-   authenticated cloud APIs and similar hosted services;
-   later: genuinely expensive compute such as AI or heavy analytics.

Initial metering should stay simple and customer-facing, for example:

-   **Storage** (database, backups, images/documents);
-   **Network / data transfer** (especially outbound);
-   **Requests / operations** (sync/API), preferably with a large
    included allowance so owners are not nickel-and-dimed per request;
-   **Compute-heavy services** only when those services exist and truly
    cost money.

Internally Gotigin may meter many resources. Externally the message
stays simple: **free local product; pay for cloud resources used; same
rates for everyone.** Exact unit prices are commercial configuration,
not a product claim in this document.

Optional later services (SSO, on-premise managed deployments, custom
integrations) may exist as contracted cloud/ops offerings. They must
**not** be advertised as already implemented until evidence exists, and
they must not reintroduce feature-tier gating of the free local
product.

### What this replaces

| Earlier plan | Current decision |
| --- | --- |
| Community Edition as a limited free tier | Full local Kumkaj free forever |
| 14-day Professional evaluation | Removed --- no trial of local software |
| Annual Professional / Enterprise entitlement | Replaced by usage-based Kumkaj Cloud |
| Feature unlock by edition | Local features free; cloud infrastructure billed |
| Branch ceilings as the main paid limit | Scale via cloud consumption, not artificial software ceilings |

------------------------------------------------------------------------

## 22. Safe Commercial / Cloud Continuity Principle

Billing and cloud account state must never hold a restaurant's
operational data hostage.

Non-negotiable rule:

> Stopping, pausing, or failing to pay for Kumkaj Cloud must not delete
> restaurant data or make the restaurant unable to access, operate, or
> export its own local information.

When cloud service is unavailable, unpaid, or deliberately stopped:

-   full local / offline operation continues;
-   cloud sync, remote dashboard, and other cloud-only controls pause;
-   unsynchronized local events remain preserved for later reconnect;
-   local backups and exports remain available;
-   restoring cloud service resumes cloud features without destructive
    migration or data loss.

There is no “trial expiry that locks the POS.” Local Kumkaj does not
expire. Cloud billing affects cloud infrastructure only.

------------------------------------------------------------------------

## 23. India-First, Without Locking the Architecture to India

The initial market is India and the product can be seeded with:

-   INR;
-   common cash/card/UPI payment recording;
-   India-friendly restaurant workflows.

However, currency and fiscal configuration should remain
per-business/configurable so the domain model is not unnecessarily
India-only.

This is now implemented as **country packs** (`apps/kumkaj/src/fiscal.rs`).
A pack carries the tax system and its name (VAT, GST, sales tax, IVA,
BTW, SST, consumption tax), whether prices include tax or have it added
at the till, the usual rates, any receipt split such as India's
CGST/SGST, the instant-pay rail label (UPI, Pix, PayNow, FPS, Interac),
the business-id label (GSTIN, VAT number, ABN, EIN, TRN) and the date
order. India remains the default pack, so seeding the product for its
first market costs an existing shop nothing.

A country pack is seller-facing wording plus arithmetic defaults. It is
**not** a statutory filing engine. Kumkaj still does not claim
GST/e-invoicing, clearance e-invoicing, fiscal-printer support or any
other statutory compliance — in India or anywhere else — until those
exact workflows have been implemented, tested and appropriately
reviewed. The rates in a pack are starting points a seller edits, and
the seller remains responsible for their own filings.

------------------------------------------------------------------------

## 24. UX Direction

An early capability map included a desktop dashboard/sidebar with areas
such as:

-   Dashboard
-   POS
-   Orders
-   Kitchen / KDS
-   Menu
-   Inventory
-   Billing
-   Customers
-   Reports
-   Settings
-   Users
-   Plugins

This should be treated as a **capability map**, not a requirement that
every user sees every module all the time.

For a chai seller, the best home screen may essentially be the POS
itself.

For a restaurant cashier, POS/orders/tables may dominate.

For a kitchen operator, KDS may dominate.

For an owner, Dashboard/Reports may dominate.

For a multi-branch owner, organization and branch controls become
important.

The interface should therefore become **role-aware and
business-scale-aware**.

------------------------------------------------------------------------

## 25. Current Technical Direction

The current Kumkaj implementation is being developed as a **native Rust
application using Makepad**.

Current project structure includes:

``` text
workspace/
    └── apps/
        └── kumkaj/
            └── src/
                └── main.rs
```

### Makepad version and source of truth

Kumkaj targets **Makepad version 2** from the official repository
**`main` branch** (`https://github.com/makepad/makepad.git`), **not** the
`dev` branch.

Important discipline:

-   Prefer APIs, widgets, DSL patterns, and Studio workflows that match
    the checked-out **Makepad 2 / `main`** revision used by this
    workspace.
-   Treat much of Makepad's older public material carefully: docs,
    tutorials, blog posts, and many examples still reflect **version 1**
    patterns and are often **deprecated or misleading** for current
    work.
-   When docs and live `main` code disagree, **the `main` source and the
    locked Cargo revision win**.
-   Do not copy v1 app structure, live macros, or widget idioms into
    Kumkaj without verifying them against the current Makepad 2 tree.
-   Pin/update Makepad deliberately; keep Kumkaj and Makepad Studio on
    compatible `main` revisions when possible.

### Recommended live-dev workflow (Cursor)

Primary coding stays in **Cursor**. For source ↔ UI live iteration, run:

```bash
cargo kumkaj-dev
```

which is an alias for:

```bash
cargo run -p kumkaj -- --hot --remote
```

Meaning of the flags:

-   `--hot` — watches `script_mod!` Splash DSL and applies UI changes on
    save without a full recompile (Rust/struct/`#(...)` placeholder
    changes still need a rebuild/restart).
-   `--remote` — enables the localhost remote control surface and the
    in-app Tweaker (**F12**) for live visual inspection/editing of the
    running UI.

Notes:

-   `--devtools` is **not** a Makepad flag on current `main`; it can be
    omitted.
-   **Makepad Studio** remains useful for Studio-centric or agent/remote
    bridge workflows, but it is a second editor. For day-to-day Kumkaj
    coding in Cursor, `cargo kumkaj-dev` is the more convenient path.
-   There is also a Cursor/VS Code task: **Kumkaj: live (--hot --remote)**.

Makepad Studio is tooling; **Kumkaj is the product**.

The product architecture should not be distorted merely to accommodate a
development-tool issue.

------------------------------------------------------------------------

## 26. Data Architecture Principles

Regardless of the final implementation details, the business domain
should clearly distinguish concepts such as:

-   organization;
-   branch/outlet;
-   device;
-   user/staff member;
-   role/permission;
-   menu/category/item/modifier;
-   customer;
-   customer credit balance and ledger entries;
-   table;
-   order/order item;
-   kitchen ticket;
-   payment (including credit settlement);
-   invoice;
-   sale Applied Total (`lines_total`) / rounding / Rounded Total
    (`total`) accounting identity;
-   sale revision history (append-only previous snapshots with actor,
    timestamp and reason);
-   inventory item;
-   stock movement;
-   purchase;
-   expense;
-   audit event;
-   synchronization/outbox event.

Even the smallest deployment can use a subset of this model. Designing
branch and organization boundaries properly early prevents multi-branch
support from becoming a rewrite later.

------------------------------------------------------------------------

## 27. Reliability Rules

Kumkaj is dealing with real businesses and money. The following should
be treated as non-negotiable engineering principles:

1.  A completed sale must not disappear after restart.
2.  Internet loss must not stop ordinary local billing.
3.  Cloud failure must not destroy or block local POS data.
4.  Duplicate clicks/retries must not accidentally duplicate financial
    transactions.
5.  Migrations must be versioned and recoverable.
6.  Backup restoration must actually be tested.
7.  Financial and important operational corrections must remain
    traceable.
8.  Error handling should preserve the cashier's work whenever possible.
9.  Data from one organization/branch must not leak into another.
10. Features can be delayed; data integrity should not be traded away to
    meet a deadline.

------------------------------------------------------------------------

## 28. Security and Privacy Baseline

Important previously established principles include:

-   least-privilege access;
-   secure password hashing;
-   owner authentication with stronger options such as passkeys where
    appropriate;
-   staff PINs designed for fast local access but protected against
    trivial abuse;
-   TLS for cloud communication;
-   secrets kept out of source code/logs;
-   tenant/organization isolation;
-   safe handling of customer data;
-   no unnecessary card data;
-   encrypted backup/cloud transport;
-   structured audit/security events;
-   owner-controlled export.

Production restaurant data should not automatically become AI training
data.

------------------------------------------------------------------------

## 29. Extensibility

Kumkaj should be capable of growing through optional
modules/integrations rather than accumulating hard dependencies.

Longer-term areas include:

-   raw ESC/POS;
-   cash drawers;
-   scanners;
-   weighing scales;
-   customer displays;
-   advanced label printing;
-   delivery aggregators;
-   payment-provider integrations;
-   advanced recipe/production inventory;
-   wastage and forecasting;
-   loyalty;
-   advanced approvals;
-   fraud/anomaly detection;
-   GST/e-invoicing modules;
-   custom APIs;
-   enterprise SSO;
-   on-premise deployments;
-   organization-level analytics.

These are **roadmap directions**, not claims that they already exist.

------------------------------------------------------------------------

## 30. A Practical Scaling Example

### Seller A --- roadside chai stall

Enabled:

-   POS
-   tiny menu
-   cash/UPI recording
-   **simple customer credit** (names + balances +
    settlements)
-   simple receipt if wanted
-   today's totals (including cash in vs credit outstanding)

Hidden/not required:

-   tables
-   KDS
-   inventory recipes
-   full CRM / loyalty
-   employees
-   branches
-   cloud dashboard
-   paid cloud services

### Seller B --- small café

Adds:

-   more structured menu
-   takeaway/dine-in
-   customers
-   thermal printing
-   simple stock
-   expenses
-   reports

### Business C --- established restaurant

Adds:

-   tables
-   waiter/cashier roles
-   KOT/KDS
-   modifiers
-   inventory/purchases
-   manager approvals
-   detailed reports
-   backup/recovery

### Business D --- large restaurant

Adds:

-   multiple POS devices
-   multiple kitchen workflows/stations
-   larger staff/permission structure
-   operational monitoring
-   deeper inventory
-   stronger audit/control workflows

### Business E --- restaurant group

Adds optional **Kumkaj Cloud** usage (same rates as any other
customer; higher total bill only because consumption is higher):

-   organization
-   multiple branches coordinated over WAN
-   centralized owner account / dashboard
-   branch switching
-   cloud synchronization
-   automatic cloud backups
-   consolidated reports
-   remote health/visibility
-   device/user administration
-   APIs/integrations

Each branch still runs free local Kumkaj. Cloud is additive, never
required for counter service.

**All five are Kumkaj.** They should not require five unrelated
products.

------------------------------------------------------------------------

## 31. What Kumkaj Should Avoid Becoming

Kumkaj should not become:

-   a cloud-only POS that stops when the internet stops;
-   an enterprise ERP forced onto a tiny shop or personal books;
-   a simple cash register that large restaurants immediately outgrow;
-   a collection of unrelated modules with inconsistent workflows;
-   a stack of **vertical forks** or trade-specific schemas that
    duplicate the core (a separate “courier product”, “barber mode”,
    “personal finance app”, and so on);
-   a system where cloud billing failure or unpaid cloud usage threatens
    local data or local POS;
-   a crippled “free” product that withholds core local capabilities to
    force a subscription;
-   a system where financial history can be silently edited without
    retaining who/when/previous values;
-   a product that claims hardware/regulatory integrations before they
    are actually tested;
-   a UI designed around feature count instead of service speed.

------------------------------------------------------------------------

## 32. Product North Star

A useful way to judge Kumkaj is:

> **Can the smallest entity — a tea seller, a kirana counter, or someone
> keeping personal books — begin using it quickly, while the most
> complex entity can keep using the same platform as it grows into
> multiple busy branches?**

(Tea stall and restaurant group remain useful examples of those poles;
they are not the only poles.)

If the answer remains yes, the architecture and UX are moving in the
intended direction.

The smallest entity should receive simplicity, speed and reliability.

The largest entity should receive scale, control, visibility and
integration.

Neither should have to sacrifice ownership of their data or the ability
to operate locally. Neither should have to pay for software capability
they can run on their own machines; payment applies to Gotigin Cloud
resources they actually consume.

------------------------------------------------------------------------

## 33. Immediate Development Focus

Now that the Makepad development environment can run Kumkaj in Studio,
development should return to the product itself.

A sensible implementation sequence is:

1.  Establish the real Kumkaj application shell and design language.
2.  Build the smallest excellent POS/order flow.
3.  Persist menu/orders/payments locally and make restart recovery
    trustworthy.
4.  Add **customer credit** (lightweight customers,
    credit sales, outstanding balances, settlements) --- this belongs
    with the core POS, not after restaurant CRM.
5.  Add receipts/printing.
6.  Add optional restaurant capabilities: tables and KOT/KDS.
7.  Add operational modules: inventory, purchases, expenses and reports.
8.  Add users/roles/audit controls.
9.  Prove backup/restore.
10. Only then layer optional Kumkaj Cloud (sync, remote visibility,
    usage metering) onto the reliable free local system.

**Local implementation status (shipping code):** steps 1–9 are in the
Makepad app (**schema v13**): POS with three totals, credit, stock
enforcement, invoice numbers, country fiscal packs; receipts
(print/share on desktop; clipboard share on Android); tables +
multi-station KOT; inventory/purchases/expenses/reports; staff roles
(Owner/Manager/Cashier; Waiter/Kitchen when Tables & kitchen is on) with feature flags; shop-only
write authority with optional LAN extra-till sell (including split tenders)/amend/KOT/void/refund/settle;
pay debounce; PIN reset after migrate + unlock backoff; verified
`restore_from` path; organization + branch switch + snapshot
export/import + consolidated roll-up + write-only sync outbox scaffold.
Step 10 (hosted WAN cloud, metering, remote dashboard), raw ESC/POS,
and statutory e-invoicing remain optional / roadmap — not required for
local multi-branch coordination.

At every stage, keep both extremes in view:

**Would this still feel effortless to the smallest entity (e.g. chai
wala or personal books)?**\
**Would the underlying model still work for the most complex entity
(e.g. multi-branch restaurant or agency group)?**\
**Is the implementation still generic, with concrete feel from
labels/config rather than a vertical fork?**

That is the central design discipline for Kumkaj.
