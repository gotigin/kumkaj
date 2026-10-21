# Kumkaj --- Product Vision & Requirements

> **Brand / domain:** Kumkaj · kumkaj.com\
> **What ships:** a **family of concrete business operating system apps**
> under the Kumkaj brand (courier desk, counter shop, food floor,
> personal books, …) — each product feels purpose-built for that user.\
> **Platform ids (current flagship):** production `com.kumkaj.courier` ·
> development `com.kumkaj.courier.debug` (reverse-DNS under kumkaj.com; see
> [`DEV.md`](DEV.md)). Further concrete products may use their own
> package ids under the same brand.\
> **Document purpose:** Consolidate product ideas, requirements,
> principles, scope, and scaling so development stays focused on
> shipping concrete apps users understand — not tooling theatre.\
> **Decision rule:** When earlier edition or pricing plans conflict with
> later finalized decisions in this document, **the later decision
> wins**. In particular, Community / Professional trial / annual
> entitlement pricing is superseded by free-forever local use and
> usage-based cloud billing. When earlier “one generic app for everyone”
> wording conflicts with **concrete products + shared internals**, the
> concrete-products model wins.

------------------------------------------------------------------------

## 1. What Kumkaj Is

**Kumkaj is the brand** for a set of **concrete business operating system
apps**. Real users only adopt software that already feels like *their*
trade or household. We therefore ship (and market) distinct products —
for example a courier counter app, a retail/counter app, a food-floor
app, a personal/household books app — not one vague “works for anyone”
SKU that forces the owner to invent their own meaning.

Each concrete product is still a **business operating system** for that
entity: money in and out, parties, day/period books, receipts/invoices,
and the ops that trade needs — not a dumb cash register.

Examples of **concrete products under the brand** (illustrative):

-   **Courier / logistics desk** — parties (consignor / carrier), kg
    rates, account vs cash booking, day book, bank book, expense
    vouchers, debtors/creditors, P&L; later booking-desk ops (AWB,
    labels/scan) as that product needs them.
-   **Counter / retail shop** — fast catalogue sell, credit, stock,
    day totals.
-   **Food / café / restaurant** — sell plus optional tables, KOT/KDS,
    stations.
-   **Personal / household books** — money in/out and people on account
    without shop chrome.
-   **Multi-branch owner** — same concrete product(s) plus org/branch
    controls when that product enables them.

**How we build (internals):** focus on the concrete product the user
sees. When an implementation is clearly useful to another product, **move
it into shared reusable code**. Prefer shapes that *can* be shared later
(party, line, unit, charge, ledger, day book, document) — do not invent
throwaway vertical schemas that can never be extracted. Do not maintain
five divergent business engines.

The central idea is therefore:

> **Concrete apps under the Kumkaj brand for users; shared reusable
> code underneath for us. Users never have to learn a foreign trade to
> fit the software.**

A courier owner should not have to understand kitchen routing. A small
counter should not have to understand franchise AWB manifests. Extra
tills stay “same shop, LAN on” — not a different cloud product type.

------------------------------------------------------------------------

## 2. Product Philosophy: Simple at the Bottom, Powerful at the Top

**Standing rule — users first:**

> **Software should adapt to users — not the other way around.**

Software is for the users; users are not for the software. Each Kumkaj
product bends toward how that trade or household already works — people
should not bend their work to fit a generic UI.

**Granular control is part of that rule.** Where a person can today
cancel, cut, correct or re-total something by hand in a paper book, the
app must offer the same freedom: sensible defaults everywhere, but the
user can override units, conversions, divisors, slabs, rates, labels
and totals at the point of work. A default the user cannot change is a
restriction; restrictions need a statutory or integrity reason (e.g.
not silently rewriting posted history), never "the first market does
it this way". Design settings as *seeds the shop edits*, not as
constants the shop obeys.

**Build order:** implement the **concrete product** first (flows, labels,
packaging, capabilities that trade needs). When the same logic is needed
by another product, **extract into shared code**. Progressive complexity
still applies *inside* each product: hide unused capability; grow from
day book + booking to deeper ops without enterprise on day one.

For most products the shell still makes **money in, money out, parties,
cash, and day/period books** feel like peers — IA along the lines of
**Books · Sell/Booking · Buy/Pay out · More** with **product-specific
names** (see [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md),
[`DESIGN.md`](DESIGN.md)). Do not treat a till grid as the whole product
identity.

A food product may additionally enable:

**Tables → orders → KOT/KDS → Kitchen → Billing → Inventory →
Expenses → Reports**

A multi-branch owner may additionally enable:

**Organization → Branches → Users/Devices → Central Dashboard →
Cross-Branch Reports → Sync/Backup → Remote Monitoring**

Features that are irrelevant to that product stay out of the way.

**Client signal → shared capability:** when a client describes something
in vertical language (courier parties, kg credit, salon chairs, household
expense buckets), implement it in the concrete product; when a second
product needs the same idea, promote a **shared primitive** (party, unit
on a line, ledger, day book, charge, document). Do not fork a
trade-specific schema that duplicates ledgers already shared.

**Start templates / product defaults:** each concrete product may bake
its own default labels and feature defaults (e.g. Account booking vs Cash
booking). Templates and defaults **never pre-load sample catalogue or
parties** — the owner adds their own. A generic / unset face may still
offer **None** (bare minimal) as default (`custom`) for experimentation.

------------------------------------------------------------------------

## 3. Minimal sell and credit path

The smallest useful path must stay fast and understandable without
training. Many entities do not need tables, kitchen screens, staff
roles, or cloud dashboards on day one. They do often need party memory:
who bought on **credit**, how much is outstanding, and when that party
paid later.

Core counter workflow:

1.  Tap an item.
2.  Change quantity if needed.
3.  See line and bill totals immediately (**Actual** / **Applied** on
    each line; **Actual** / **Applied** / **Rounded** on the bill).
4.  Optionally adjust a line's `@` unit price, that line's **Applied**
    amount, or the bill **Rounded Total** before paying (catalogue price
    stays the default; `@` is never rewritten by a line Applied edit).
5.  Record cash/UPI/other payment **or** put the sale on a party's
    credit.
6.  Finish the sale — the **recorded history** of that moment is kept
    forever; the owner may later amend the live order with a tracked
    revision (who / when / why / previous values).
7.  Optionally print a receipt.
8.  At the end of the day, see sales, payment totals, and **who still
    owes money**.
9.  When a credit party pays later, record the settlement against that
    balance without inventing a fake new sale.

Credit must not force a full CRM setup: a name (and maybe a phone note)
should be enough to start a credit account. Ordinary sales must not
require an internet connection or enterprise setup before the first
bill.

------------------------------------------------------------------------

## 4. Optional depth (tables, kitchen, ops)

The same product must support fuller operations when the entity enables
them — without forcing that depth on everyone.

That may include:

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
without damaging the speed of the sell workflow.

------------------------------------------------------------------------

## 5. Multi-branch and organization

Kumkaj must have an architecture that can grow beyond one physical
location. An owner with several outlets should eventually be able to
manage an **organization** containing multiple branches.

Important capabilities include:

-   branch selector;
-   branch-specific catalogue/pricing where required;
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
branch must still be able to run its full local books without the cloud.
A cloud outage must **not stop an outlet from billing locally**. Each
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

Table functionality should be optional. Entities that do not use tables
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

**Party credit is core local capability**, not a later CRM module. Many
entities sell on account and settle later (daily, weekly, or when they
can). Kumkaj must support that on the same device as ordinary sales.

### Minimum credit capability

Even without a rich party module, the owner should be able to:

-   keep a short list of credit parties (name is enough; phone/notes
    optional);
-   finish a sale as **credit** against a chosen party;
-   see that party's **outstanding balance**;
-   see a simple list of **who owes how much** (today / all open);
-   record a later **settlement** (cash/UPI/card) that reduces the
    balance;
-   keep an attributable history of credit sales and settlements so
    totals remain trustworthy.

Few taps, no training, no mandatory cloud, no enterprise party-master
setup.

Credit sales still count as real sales for operational visibility, but
**cash/UPI/card day totals and credit outstanding must not be confused**.
The owner needs both: what came in today, and what is still owed.

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
broader operation. The **universal books spine** (what almost every entity
must track) is catalogued in [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) —
party, item/unit, sell, buy, expense, AR/AP, cash/bank, stock, tax,
day/period books — as generic primitives only.

Core operational areas include:

-   inventory items;
-   stock movements;
-   purchases (goods for resale / use — cost feeds **COGS** in period books
    as that loop closes; see `BUSINESS_BOOKS.md` backlog);
-   suppliers as the same **party** primitive (kind label), growing into
    payable balances — not a separate vertical schema;
-   low-stock visibility;
-   stock adjustments;
-   expenses (operating costs, distinct from purchases of goods);
-   wastage;
-   recipe/ingredient-based deduction as a deeper restaurant capability.

Stock corrections should be traceable. More advanced recipe production,
forecasting and supplier automation can evolve after the foundational
inventory model is trustworthy.

------------------------------------------------------------------------

## 16. Reports and Owner Visibility

Useful reporting should answer operational questions rather than merely
display charts. See also [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md).

Local reports can include:

-   daily sales and **day close** (tenders, credit taken, expenses, bank moves);
-   payment-method totals (cash / UPI / card / credit);
-   **lines Actual / Applied vs Rounded totals vs rounding adjustments**
    (so line overrides and bill round-offs remain reconcilable);
-   bill-type splits (**Tax invoice / Account booking** vs **Cash memo /
    Cash booking**) when the entity uses both tags;
-   open credit outstanding (by party and total) with **aging**;
-   credit settlements collected today;
-   tax totals;
-   item/product sales;
-   order counts;
-   average bill;
-   expenses;
-   **period books** (today / month) and operational **P&L** (revenue −
    COGS − expenses once COGS is wired; today: revenue − expenses);
-   bank / cashbook posts (till↔bank, bank in/out) and running bank balance;
-   stock/adjustments;
-   cash-session / counted-vs-expected drawer check as that capability lands.

For multi-branch businesses, the owner should additionally be able to
view branch-specific and consolidated information.

The dashboard should adapt to the scale of the user. A small counter may
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

### Ground truth: record money as it actually moved

In real trade, one obligation is often settled in a **mix**: part
through banked channels (bank transfer, cheque, UPI, card), part in
**cash in hand**; part may carry a bill, receipt, or bank trail, and
part may be settled in cash without any paper document. This is
ordinary ground reality across trades and countries — not an
exception, and not the app's business to judge.

Kumkaj's books are therefore the owner's **ground truth**:

- Record the **full amount that actually moved**, split by how it
  moved (multiple tenders on one entry). Cash that left the till is a
  payment in full — recording only the banked part makes the till
  unreconcilable, expenses understated, and profit a fiction.
- A bill, invoice, receipt, cheque, or bank reference is **evidence**
  attached to an entry (a note or a file) — never a **gate**. The app
  must not refuse, shrink, or hide an entry because no document exists
  for part of it or because a bank statement does not show it.
- Owner-facing views always show the **true position**: cash in hand,
  bank, who owes whom, expenses, and profit — computed from actual
  totals across all tenders, filterable by tender/channel, never
  silently excluding the cash-settled part. Book cash must be able to
  tally with physical cash.
- There is **one truth**; what leaves the shop is the owner's choice.
  Documents the app prints, shares, or exports are drawn from the
  record, the owner chooses what to issue, and a document makes clear
  the scope it covers. The app never fabricates a document for money
  that did not move, and never rewrites an amount or a payment method
  on paper.
- This truth stays **private**: on the owner's device, behind the
  owner's lock. Entries are described in neutral words — tender /
  channel, with or without a supporting document — never in slang or
  moral categories.
- Statutory or filing outputs, when they arrive, are **derived
  presentations with an explicit scope**, layered on the ground truth.
  They neither rewrite the owner's books nor borrow their authority:
  the management books do not masquerade as a filing, and a filing
  view does not shrink what the owner knows.

**Agent rule:** every money feature must accept multi-tender
settlement (one entry, several money movements), must not require a
bill/document reference to save, and must include cash-settled parts
in day book, P&L, aging, and till math by default. If a design only
works when every rupee is banked and billed, the design is wrong.

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
coordination over WAN, or other Kumkaj-hosted infrastructure.

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
require Kumkaj Cloud and must remain available as part of free local
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
> If you want Kumkaj Cloud, pay only for the cloud resources you
> actually consume. Same rates for everyone.**

Core principle:

> **Software capability is not the thing being sold. Infrastructure
> consumption is.**

Kumkaj does not decide who is “small” or “enterprise” for pricing.
A single-outlet entity and a multi-branch group see the **same price sheet**.
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
-   other local-only capabilities that do not consume Kumkaj
    infrastructure.

Do **not** charge for things that cost Kumkaj essentially nothing to
host, such as number of employees, menu items, tables, local orders,
local devices on a LAN, inventory features, KOT, or local reports.

A restaurant may run locally for years, never pay Kumkaj, and still
have a functioning system. Local data remains owned by the restaurant.

### Kumkaj Cloud --- Paid by Usage, Same Rates for All

Cloud is optional. Customers pay when they use Kumkaj-hosted
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

Internally Kumkaj may meter many resources. Externally the message
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

**India-first is not India-only.** India is the first market, the source
of the default seeds, and where the first users are — the audience is
the world. Every default below is a starting point a shop can change,
never a restriction baked into the domain model. Any agent or developer
working on Kumkaj must treat a hard-coded Indian convention as a bug
unless it is genuinely India-statutory (and §23's country-pack paragraph
below limits even those claims).

The initial market is India and the product can be seeded with:

-   INR;
-   common cash/card/UPI payment recording;
-   India-friendly restaurant workflows.

However, currency and fiscal configuration should remain
per-business/configurable so the domain model is not unnecessarily
India-only.

The same rule applies to **measurement**. Shops measure parcels and
goods in the units their trade and country already use. Products must
therefore:

-   let the user **choose entry/display units** (length, weight,
    quantity) instead of forcing one market's units;
-   ship **standard conversions** for the common units (cm, mm, m, in,
    ft · kg, g, t, lb, oz) with exact factors;
-   let the user define **custom units and custom conversions**
    (e.g. `1 bag = 25 kg`, `1 crate = 40 cm`) and use them anywhere a
    standard unit can be used;
-   keep **one canonical base inside the books** (kg / cm / milli-qty)
    so ledgers, rates and reports stay comparable — conversion happens
    at entry and display, never by rewriting recorded history;
-   treat trade constants (volumetric divisor, weight slab rounding,
    tax rates, pay-rail labels) as **shop settings with sensible local
    defaults**, not constants in code.

Units form **families** (weight, length, volume, count), each with one
canonical anchor (kg · cm · ml · pc). A custom unit is defined relative
to any unit of the same family — a standard one, or another custom
one — and its chain of conversions must ultimately land on the anchor
(`crate → box → cm` means 1 crate = 40 cm). Loops and cross-family
links are refused; a unit whose chain never reaches the anchor is
shown as "not linked" and cannot be used until it is. Because every
quantity can be normalised to the anchor, books store anchor units
only and reports/P&L aggregate one unit, displaying in whatever unit
the reader picks. Physical conversion factors are fixed constants of
nature, so applying them to old entries is always safe.

**Currency is not a unit.** An exchange rate is a price at a moment in
time, not a fixed conversion, so currencies never enter the unit
tables. Books are kept in the shop's currency; when foreign-currency
money is recorded, the entry must carry the amount, the currency, the
rate on that date, and the resulting shop-currency amount — history is
never re-converted at a later rate (only an explicit "value today"
view may do that).

Example (Courier, shipped): the booking desk defaults to cm + kg,
volumetric ÷ 5000, round up to 0.5 kg — the Indian domestic
convention. Settings → Booking measures lets a shop switch entry to
inches/pounds (or a custom unit), change the divisor (e.g. ÷ 6000) and
the slab; the books still record kg.

This is now implemented as **country packs** (`apps/courier/src/fiscal.rs`).
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

Home and chrome should follow what the entity enabled and who is signed
in: sell/credit for a counter; tables/orders when food floor is on; KDS
for kitchen staff; books/reports for owners; organization and branch
controls when multi-branch is in use.

The interface should therefore be **role-aware and
capability-aware**.

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
cargo courier-dev
```

which is an alias for:

```bash
cargo run -p kumkaj-courier -- --hot --remote
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
    coding in Cursor, `cargo courier-dev` is the more convenient path.
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
-   synchronization/outbox event;
-   **triple timestamps** on books entries — user/business time
    (`user_at`), device write time (`device_at`), and cloud accept time
    (`server_at`, empty until synced).

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

## 30. Progressive enablement

Complexity is additive capability **within** each concrete Kumkaj
product. Separate store listings / package ids under the brand are fine;
separate duplicated engines are not.

Typical progression (within a product; food tools only when that
product enables them):

1.  **Sell + credit + day totals** — catalogue, pay/credit, receipt,
    who still owes; tables/KDS/team/branches/cloud off.
2.  **Structured ops** — categories, stock, expenses, reports, printing
    as needed.
3.  **Tables & kitchen** — when the entity turns that pack on (floor,
    KOT/KDS, stations, waiter/kitchen roles).
4.  **Team and control** — staff, permissions, approvals, audit, backup.
5.  **Extra tills and multi-branch** — LAN clients; org + branch switch +
    local consolidated views.
6.  **Optional Kumkaj Cloud** — WAN sync, remote visibility, hosted
    backup, usage metering. Same rates for all; higher bill only from
    higher consumption. Each branch still runs free local Kumkaj. Cloud
    is additive, never required for counter service.

------------------------------------------------------------------------

## 31. What Kumkaj Should Avoid Becoming

Kumkaj should not become:

-   a cloud-only POS that stops when the internet stops;
-   an enterprise ERP forced onto a tiny shop or personal books;
-   a simple cash register that large restaurants immediately outgrow;
-   a collection of unrelated modules with inconsistent workflows;
-   **duplicated engines** or trade-specific schemas that copy the same
    ledgers five times (shipping “courier” and “personal” as forever-
    divergent codebases instead of concrete products on shared
    internals);
-   a single vague “works for everyone” SKU that forces owners to invent
    meaning before they can work;
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

> **Does each concrete product feel purpose-built on first open — and
> can shared internals still carry that owner as the entity grows
> (ops depth, branches, optional cloud)?**

If the answer remains yes, the brand and architecture are moving in the
intended direction.

The smallest entity should receive simplicity, speed and reliability in
*their* product.

The largest entity should receive scale, control, visibility and
integration without abandoning the same brand family or local ownership.

Neither should have to sacrifice ownership of their data or the ability
to operate locally. Neither should have to pay for software capability
they can run on their own machines; payment applies to Kumkaj Cloud
resources they actually consume.

------------------------------------------------------------------------

## 33. Immediate Development Focus

**Current ship focus: Kumkaj Courier** — see
[`KUMKAJ_COURIER.md`](KUMKAJ_COURIER.md) and [`USE_CASES.md`](USE_CASES.md).

Build the concrete courier BOS (books already largely present; next the
**booking desk**, AWB/barcode, rates/slabs, then tracking/manifest/COD).
Extract shared crates only when another brand product needs the same
logic. Food tables/KOT and other concrete products wait until courier
desk quality is credible.

**Existing Makepad codebase** is the starting engine (day book, booking
shell, parties, bank, expenses, P&L, Account/Cash booking tags, units).
Treat courier-named UX and desk ops as the product work — not a second
generic “any entity” SKU. Books checklist:
[`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md); IA: [`DESIGN.md`](DESIGN.md).
Cloud / ESC/POS / statutory e-invoicing remain optional later.

At every stage, keep these in view:

**Would this concrete product feel purpose-built on first open?**\
**Would shared internals still carry a complex multi-branch entity?**\
**Are we extracting shared code when a second product needs it — not
forking engines?**\
**Does the software adapt to the user — or is the user forced to adapt
to the software?** (§2)

That is the central design discipline for Kumkaj.
