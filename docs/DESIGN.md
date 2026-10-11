# Kumkaj UI design (shipping)

Source pack (optional): `../../stitch_tactile_countertop_design_system/`.  
This file is the **canonical** in-repo summary. When Stitch packs disagree, **later shipping decisions win** (see also vision doc).

## Navigation (shipping)

| Tabs | Role |
|---|---|
| **Sell · Items · More** | Canonical shipping shell |

Tables, Kitchen, Sales, Day close, Reports, Inventory, Branches, Credit, Team, Backup live under **More**. Floor / KDS / Khata 4-tab shells remain optional zoom layers — not a rewrite of current IA. Progressive complexity: **shop (extra tills off) → extra tills & team → Tables & kitchen if needed → branches**. Most sellers finish a sale without LAN or KOT.

### Launch / auth screens (outside tabs)

| Screen | When |
|---|---|
| **DeviceSetup** | Every launch — Server Mode or Client Mode |
| **Lock** | Owner/Staff PIN (extra till prefers Staff when team exists) |
| **Onboard** | After first Owner PIN — start template chips (**None** default / Counter / Courier / Food / Personal), then Fresh vs Book. None applies if nothing else is tapped. |
| **Settings** | Switch start template anytime (labels/defaults only — no sample catalogue). None is the bare-minimal default. |
| **Choice chips** | Selected face = `ChipBtnActive` visible; idle = `ChipBtn`. Horizontal `ChipStrip` scrolls with wheel/scrollbar only (not drag), so taps register. |
| **Min width** | Layout floor **320 css px**. Narrower viewports scroll horizontally (`app_hscroll` / `app_shell`). |
| **Button ink** | Shared faces use `padding` + `label_walk.margin` so labels never sit flush on the face edge. Long dual CTAs use `ActionStack`, not half-width `ActionRow`. |
| **Reports** | Today / month books, party aging, expenses, bank cashbook posts — generic books, not a vertical report suite. |
| **ClientPair** | Extra till until paired (IP + pairing code) |
| **DayClose** | More → Day close (totals + copy) |
| **Reports** | More → Reports & expenses (top items + expense log + tax inside sales) |
| **Inventory** | More → Inventory & stock |
| **Branches** | Settings or More → Branches & org (when more than one branch) |

Seller order after unlock: **Sell** (tap item · three-total ledger) → tender / credit / split → optional Floor / KDS when Tables & kitchen is on.

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

- Empty catalog: clear CTA to Items (no sample items are pre-loaded).
- Sold out / stock: Items → Sold out / Restock; Inventory for tracked qty; Sell grid blocks unavailable.
- Tables & kitchen: Settings toggle; More rows hidden until on.
- Order type: Counter / Dine-in / Takeaway chips on Sell only when Tables & kitchen is on.
- Kitchen stations: cycle filter on Kitchen; station on Items when food is on.
- Extra tills: pairing code on Lock / Settings when extra tills are allowed.
- Multi-branch: header shows `Shop · CODE` when more than one branch exists.
