# Kumkaj — identity: UUID, not names (all apps)

Canonical Kumkaj-wide doctrine for identity. This applies to **every**
Kumkaj app (Courier, Khaynjo, future products) and to every persisted
entity in them. It is a shared *idea*, not shared code: each app
implements this doc in its own code (vision §1 — no Kumkaj core).

Companions: [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) § Dual-role
parties (the party application of this rule);
[`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md)
§26 (data architecture).

## Rule in one line

**Uniqueness is decided by the UUID (`id`), never by name. A name is
only a display label — an alias a human uses to refer to the entity.**

## The three kinds of handle (do not confuse them)

| Handle | What it is | Unique? | Decides identity? |
|--------|------------|---------|-------------------|
| **UUID (`id`)** | Assigned once at creation (UUIDv4); primary key of the row | Yes — globally | **Yes. The only identity.** |
| **Name / display label** | Human text on the entity (party name, item name, branch name, …) | No — may repeat, may be renamed | No. It is an alias. |
| **Aliases** | Extra names a seller adds (e.g. item `aliases_json`: spelling variants, short names) | No | No. Search handles only. |
| **Human reference numbers** | Numbers people quote: AWB, invoice/bill no, receipt no | Scoped — e.g. invoice no unique per series/FY, AWB unique in the books | No. References *attached to* the UUID record; lookups resolve them to the `id`. |

## Rules

1. **Every persisted entity gets a UUID `id` at creation** and keeps
   it for life. All joins, ledgers, attachments, audit events, sync
   rows and operations key off `id` — never off name, phone, or any
   other display text.
2. **Names may repeat.** Two different firms, people, or items may
   share a display name; they stay distinct by UUID. Never add a
   `UNIQUE(name)` constraint, a lookup-by-name that assumes one
   result, or a merge/dedupe keyed by name alone.
3. **Names may change.** Rename / re-phone / relabel edits display
   text only — ledgers, balances, bookings and history on that `id`
   are unaffected. Display language likewise never touches identity.
4. **One real-world entity = one UUID.** A party that is both someone
   you bill and someone who bills you carries both roles on the
   **same** `id` (multi-role `kind_label`; AR and AP ledgers stay
   separate — see `BUSINESS_BOOKS.md` § Dual-role parties). Do not
   create two rows that differ only by role/kind. Conversely, never
   collapse two real entities into one row because their names match.
5. **Search by name/alias returns candidates, not identity.** Name
   and alias matching is case-insensitive and language-independent,
   and may match several entities; the operator picks the right one
   using context (phone, balance, address, photo). An exact name
   match is a convenience for finding, never proof of identity.
6. **Duplicate detection is a hint, not a merge.** Imports and
   “possible duplicate” warnings may use name + phone (or similar) as
   a heuristic to *warn*, but must not silently skip, merge, or
   re-key records by it. Any merge is an explicit operator decision
   between two named UUIDs.
7. **Aliases are first-class search handles for catalogue items:**
   a seller may add extra names to an item, and lookups match the
   display name plus its aliases. Adding or removing an alias changes
   findability only — never the item's `id`, price history, or stock.

## Why

Shops are messy in exactly this way: families and firms share names,
names get re-spelled, a business renames itself, and one firm is both
customer and supplier. If name were identity, every one of those
events would fork or corrupt the books. Keying everything on UUID
makes renames free, duplicates safe, and dual roles natural — the
books follow the real entity, not its current spelling.

## Per-app implementation (own code, same doctrine)

| App | Where its own implementation lives |
|-----|-------------------------------------|
| Kumkaj Courier | `apps/courier/src/store.rs` (`customers.id`, `products.id`, …), `src/domain.rs` (`new_id()`, product `aliases()` / `matches_label()`) |
| Khaynjo | `apps/khaynjo/src/store.rs`, `src/domain.rs` (same concepts, copied then diverged) |
| Future apps | Their own `apps/<app>/src/` — copy the best-fitting app's code, then diverge; never a shared identity crate |

## Agent rule

1. Design every new entity UUID-first: `id` primary key, name as
   plain display text, no uniqueness on names. If you find yourself
   joining, merging, or looking up “the” row by name, stop — that is
   the bug this doc exists to prevent.
2. Change the doctrine here, once — then update each app's own code
   and app-specific doc separately, app by app. App docs state only
   their trade's role labels on the party UUID and link here.
