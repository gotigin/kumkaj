# Khaynjo client requirements (discovery)

Sanitized checklist from restaurant discovery (Rajdhani and follow-ups).
**No personal names, phones, chat IDs, or download links** — only product asks.

Companions: [`KHAYNJO.md`](KHAYNJO.md), [`RESTAURANT_MARKET_RESEARCH.md`](RESTAURANT_MARKET_RESEARCH.md),
[`RESTAURANT_RESEARCH.md`](RESTAURANT_RESEARCH.md),
[`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md).

**Scope:** Khaynjo's discovery evidence and status tracker — stays in
Khaynjo's docs. Do not copy this checklist into Moklanjo; courier asks
live in [`MOKLANJO_CLIENTS_REQUIREMENTS.md`](../../moklanjo/docs/MOKLANJO_CLIENTS_REQUIREMENTS.md).

------------------------------------------------------------------------

## Who they are (role, not identity)

- Sit-down / mixed restaurant counter with floor tables.
- Owner wants Petpooja-class daily mechanics **without cloud**, with
  stronger owner control over what staff see and what pick-lists offer.
- Guests often sit before anyone punches dishes — the floor must show
  that table as busy without forcing an order.

------------------------------------------------------------------------

## Typed must-haves (owner control + floor)

### A. What non-owners can see (and what they cannot)

| Ask | Meaning | Status |
|-----|---------|--------|
| Owner controls staff visibility | Beyond action rights (pay / cancel / NC), the owner chooses which **screens, summaries, money figures, and reports** each non-owner role may open or read | **Done 2026-10-27** — Team feature chips include Floor/Kitchen; nav gates on `can_access_*` (Books/Buy/DayClose/Reports as before for money); Owner via `StaffFeatures::all()` |
| Hide books / day totals from waiters | A waiter should not browse Day book, P&L, bank, or other money screens the owner did not grant | **Done 2026-10-27** — Books/Buy hidden unless sales/items/inventory (or owner); DayClose/Reports need sales or owner; Waiter preset keeps Floor without money books |
| Same pattern for other sensitive surfaces | Kitchen may omit prices (already); Reports, Inventory cost, Team, Settings subsets follow the owner's matrix | **Done 2026-10-27** — same Team → feature chips + nav matrix (Reports/Inventory/Team/Settings already feature-gated) |

Today Khaynjo already gates **actions** (Take payment, Cancel, NC, Move/merge, Discount, Print again) per person — Allowed / With approval / Not allowed (`StaffRights`, Team → Rights). This ask is the **read/see** half: what appears in the shell for that person.

### B. Configurable pick-lists (units and the same pattern elsewhere)

| Ask | Meaning | Status |
|-----|---------|--------|
| Units dropdown shows only what this shop uses | Country pack may ship ~10 units; owner picks which appear in Buy / Inventory / recipe / suggest lists (e.g. only 5) | **Done 2026-10-27** — `units.picker_enabled`; Settings → Units On/Off (Owner); suggest lists filter; conversions keep disabled codes |
| Same for other crowded lists | Areas, kitchen stations, tax rates, payment methods, order-type chips, dish categories in day-to-day pickers — owner decides what the working list contains | **Done 2026-10-27** — areas / stations / tax rates On/Off in Settings (same pattern as units); suggest lists filter; pack defaults kept |

Doctrine stay: India-first packs still ship the full set ([`UNITS_AND_CONVERSIONS.md`](../../../docs/UNITS_AND_CONVERSIONS.md)); the shop **narrows the face**, not the maths.

### C. Occupy a table without an order

| Ask | Meaning | Status |
|-----|---------|--------|
| Mark table occupied when guests sit | Guests arrive and sit; staff mark the table busy **before** any dish is punched | **Done 2026-10-27** — `FloorState::Seated` + `Store::seat_table` / LAN `SeatTable`; Free act seats without a running order |
| Order starts later on that table | When ready, tap the occupied table → Order / Add dishes as today; occupy may create a light seat record or upgrade to running on first line | **Done 2026-10-27** — Seated tile tap opens order as today; first KOT / lines → In kitchen / Eating |
| Free again if they leave without ordering | Occupied with no bill can return to Free (with a clear action); once lines exist, today's cancel / pay path applies | **Done 2026-10-27** — Seated act frees empty table; open-bill path unchanged |

Today the floor moves Free → busy mainly when a running order / KOT path starts. This ask inserts an explicit **seated / occupied** step so the floor tells the truth during the wait for the first order.

------------------------------------------------------------------------

## Related already covered (do not re-ask)

See [`KHAYNJO.md`](KHAYNJO.md) §4b / §4d: running orders, KOT, settle, rights for actions, areas, reservations, LAN captain tablets, menu window, etc.

------------------------------------------------------------------------

## Gaps to build (priority for product)

All three typed must-haves (§A–C) are **Done 2026-10-27**. Wider Local/LAN
research catch-up is tracked in [`RESTAURANT_RESEARCH.md`](RESTAURANT_RESEARCH.md)
§8 (complete) and §8 “Still left” (Cloud/Net/owner-only).

1. **Staff visibility matrix** — **Done 2026-10-27** (Team chips + nav).  
2. **Shop-enabled units + sibling pick-lists** — **Done 2026-10-27** (areas, stations, tax rates).  
3. **Floor: Occupied without order** — **Done 2026-10-27** (`FloorState::Seated`).

Mirror one-line status in [`KHAYNJO.md`](KHAYNJO.md) §4b.
