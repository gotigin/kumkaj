# Kumkaj UI design (shipping)

Source pack (optional): `../../stitch_tactile_countertop_design_system/`.  
This file is the **canonical** in-repo summary. When Stitch packs disagree, **later shipping decisions win** (see also vision doc + [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)).

## Navigation (shipping)

| Tabs | Role |
|---|---|
| **Books · Sell · Buy · More** | Canonical shipping shell (from cross-category commons) |

| Tab | Purpose |
|---|---|
| **Books** | Home after unlock — today’s money in/out, AR/AP, COGS/gross/net, cash vs bank; shortcuts to day close, reports, credit |
| **Sell** | Money in — tap items, three-total ledger, tender / credit / split; Account vs Cash booking labels |
| **Buy** | Money out — purchase (paid or on-account AP) and expense; qty with item units |
| **More** | Items catalogue, Inventory (stock adjust), Sales, Day close, Reports, Credit (AR + AP), Tables/Kitchen when on, Branches, Team, Backup, Settings (units) |

Tables, Kitchen, Sales, Day close, Reports, Inventory, Branches, Credit, Team, Backup, **Items** live under **More** (or one tap from Books). Floor / KDS remain optional zoom layers — not a rewrite of this IA. Progressive complexity: **books peers → extra tills & team → Tables & kitchen if needed → branches**.

Commons evidence: [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) § Cross-category research. Sell is not the product identity; money out is a peer of money in.

### Launch / auth screens (outside tabs)

| Screen | When |
|---|---|
| **DeviceSetup** | **Every app start** (multi-user / multi-business on one device). One screen: **language** + **Server Mode / Client Mode**. |
| **Lock** | Owner/Staff PIN (extra till prefers Staff when team exists) |

#### DeviceSetup — language (every launch)

- Screen opens in the **platform / OS language** so non-English users can read it.
- Options: **Use platform native language** (follow the device; updates if the OS language changes) or **Use user selected language** (catalogue list; platform language pre-selected as the default pick).
- Changing language **updates this screen immediately** (Server/Client labels and help text included).
- Choice is persisted when the user continues into Server or Client Mode. Settings can change it later.

#### DeviceSetup — Server Mode (local multi-server)

Same device may hold **several local servers** (different products or businesses). Each server is a separate books database with a **unique id** (`servers/{id}/kumkaj.sqlite3`, registry in `servers.json`).

| Situation | Behaviour |
|---|---|
| **No local server yet** | Server Mode **creates** a new server (unique id) and continues to Lock / Owner PIN. |
| **Servers already exist** | Server Mode shows **Create new server** (optional name) or **Open** an existing one from the list, then continues with that server. |

**Client Mode** uses a separate client cache DB (`client/kumkaj.sqlite3`) and pairs over LAN to a Server — it does not pick from the local server registry.

Legacy single-file `kumkaj.sqlite3` at the data-dir root is **migrated once** into the first registered server.
| **Onboard** | After first Owner PIN — start template chips (**None** default / Counter / Courier / Food / Personal), then Fresh vs Book. None applies if nothing else is tapped. |
| **Settings** | Switch start template anytime (labels/defaults only — no sample catalogue). None is the bare-minimal default. |
| **Choice chips** | Selected face = `ChipBtnActive` visible; idle = `ChipBtn`. Horizontal `ChipStrip` scrolls with wheel/scrollbar only (not drag), so taps register. |
| **Min width** | Layout floor **320 css px**. Narrower viewports scroll horizontally (`app_hscroll` / `app_shell`). |
| **Button ink** | Shared faces use `padding` + `label_walk.margin` so labels never sit flush on the face edge. Long dual CTAs use `ActionStack`, not half-width `ActionRow`. |
| **Reports** | Today / month books, party aging, expenses, bank cashbook posts — generic books, not a vertical report suite. |
| **ClientPair** | Extra till until paired (IP + pairing code) |
| **DayClose** | Books shortcut or More → Day close (totals + optional drawer count + copy) |
| **Reports** | Books shortcut or More → Reports & expenses (period P&L with COGS) |
| **Credit** | Parties, AR settle/write-off, AP pay/write-off |
| **Settings** | Country pack, venue, start template, **Units** + conversions, backup |
| **Buy** | Bottom tab — purchase + expense (money out) |
| **Inventory** | More → Inventory & stock (adjust / track; purchase also on Buy) |
| **Branches** | Settings or More → Branches & org (when more than one branch) |

Owner order after unlock: **Books** (today’s position) → **Sell** or **Buy** as peers → optional Floor / KDS when Tables & kitchen is on.

The shipping UI is Makepad using these DESIGN tokens. Full Stitch pack pixel parity is not required to ship.

## Color tokens (live)

| Role | Hex | Use |
|---|---|---|
| Surface | `#f9f9fc` | App canvas |
| Surface wells | `#e8e8ea` / `#f3f3f6` / `#ffffff` | Recessed trays, cards |
| On-surface | `#1a1c1e` | Body ink on light faces |
| Muted | `#45464e` / `#75777f` | Hints, placeholders |
| Navy | `#041534` / `#1b2a4a` | Header, tabs, primary actions |
| Terracotta | `#a23e18` | Accents, borders, warnings |
| Brass | `#c59b27` | Status marks on navy |

**Rules**

- Cool surface `#f9f9fc` wins over parchment/cream `#f7f4eb` for body canvas.
- Cream / white glyphs **only** on navy chrome (window caption). Never cream-on-white body, placeholders, or hover.
- Terracotta is border/ink on light surfaces — not peach fills (`#ffdbcf`).
- Light faces: dark ink identical in rest / hover / focus / down.
- Navy faces: white ink identical in all states.
- Never flip ink lightness when the face flips (Makepad mixes colors mid-hover).

## Contrast checklist (Makepad)

- Theme label ink = `#1a1c1e` (never white/cream as default).
- Placeholders = `#75777f` (hover `#45464e`) — muted dark on light wells (`FieldInput` / `CartField`); never white-on-gray. Re-bind `draw_text.color_empty*` on stock inputs after theme (same bake issue as the caret).
- Pay / tab buttons on navy: white text in every state.

## Product cues that must stay obvious

- Empty catalog: clear CTA to Items under More (no sample items are pre-loaded).
- Sold out / stock: Items → Sold out / Restock; Inventory for tracked qty; Sell grid blocks unavailable.
- Tables & kitchen: Settings toggle; More rows hidden until on.
- Order type: Counter / Dine-in / Takeaway chips on Sell only when Tables & kitchen is on.
- Kitchen stations: cycle filter on Kitchen; station on Items when food is on.
- Extra tills: pairing code on Lock / Settings when extra tills are allowed.
- Multi-branch: header shows `Shop · CODE` when more than one branch exists.
- Books home: empty/first-day copy teaches money in, money out, parties — not “open Sell to begin.”
- Books time: day books follow **user / business time** (`user_at`); device write time and cloud sync time are separate fields (see [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) § Triple timestamps).
