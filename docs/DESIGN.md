# Kumkaj UI design (shipping)

Source pack (optional): `../../stitch_tactile_countertop_design_system/`.  
This file is the **canonical** in-repo summary. When Stitch packs disagree, **later shipping decisions win** (see also vision doc + [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)).

**Current product focus:** [`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md) — UI names below use courier wording; other concrete products will override labels the same way.

## Navigation (shipping)

| Tabs | Role |
|---|---|
| **Day book · Booking · Pay out · Rates · Ops · …** | Kumkaj Courier shell — no **More** hub; former More destinations are first-class peers |

| Tab | Purpose (Courier) |
|---|---|
| **Day book** | Home after unlock — today’s money in/out, AR/AP, cash vs bank; shortcuts to day close, bank/P&L, parties |
| **Booking** | Money in — book parcels (rates × kg), three-total ledger, tender / party credit; Account vs Cash booking |
| **Pay out** | Money out — carrier bills / purchases and expense vouchers |
| **Rates · Ops · COD · Riders · Sales · Day close · Reports · Parties · Team · Settings · Lock** | Former More rows — same destinations, shown as scrollable bottom chips / rail peers |

Progressive depth for Courier: **books + booking desk → AWB/barcode → tracking/manifest/COD → extra tills/team**. See [`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md).

Booking is not the whole product; Day book and Pay out are peers.

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

Same device may hold **several local servers** (different products or businesses). Each server is a separate books database with a **unique id** (`servers/{id}/courier.sqlite3`, registry in `servers.json`).

| Situation | Behaviour |
|---|---|
| **No local server yet** | Server Mode **creates** a new server (unique id) and continues to Lock / Owner PIN. |
| **Servers already exist** | Server Mode shows **Create new server** (optional name) or **Open** an existing one from the list, then continues with that server. |

**Client Mode** uses a separate client cache DB (`client/courier.sqlite3`) and pairs over LAN to a Server — it does not pick from the local server registry.

Pre-release: delete any older local data dirs (`com.kumkaj.app*`, root `kumkaj.sqlite3`, etc.) — there is no migration from those layouts.
| **Onboard** | Kumkaj Courier: courier defaults baked in (Account/Cash booking, kg). Fresh vs Book for opening rates/parties. Other brand products bake their own defaults. |
| **Settings** | Country pack, venue, units, backup (no sample catalogue). |
| **Choice chips** | Short labels only (`ChipBtn` / `ChipBtnActive` in `ChipStrip`). Multi-word choices are **not** chips. |
| **Stacked CTAs** | Multi-word / long labels → `ActionStack` + `StackPrimaryBtn` / `StackSecondaryBtn` (full width, wrap, no ellipsis). Same faces as Server/Client Mode. Never `ChipBtn` / `ActionRow` for those. |
| **Form field rows** | Side-by-side inputs only → `FormFieldRow` (equal `Fill` fields). **Never** put a Fit/`NavBtn` CTA in that row — Makepad has no flex-shrink; wrap stacks letters vertically (`A d d r a t e`) and overlaps the field. CTAs go in `ActionStack` under the fields. |
| **Bottom nav** | Phone chrome (< **720 css px**), horizontally scrollable. Active peer = only the `*Active` face visible (`paint_nav_tabs`); idle stays transparent on hover/focus. Peers include Day book · Booking · Pay out **plus** every former More destination (role-gated). |
| **Side rail** | Tablet / desktop chrome (≥ **720 css px**), vertically scrollable. Same peer set as bottom nav (`nav_rail` / `apply_shell_layout`). Bottom tabs hide; rail shows. Drag the divider to resize; rail floor **148** css px, main column floor **360** css px. Short chrome labels + wrap (no ellipsis). Top-level peers have no Back button / history swipe. |
| **Booking split** | Side-by-side when **content** column ≥ ~640 css px (rates \| cart, both `Fill`). Narrower: `flow: Down`, each pane full width — never a Fixed cart with empty right gutter. Shell chrome still flips rail/bottom at 720. |
| **CTA labels** | Full-width faces (`NavBtn`, `PayBtn`, `MoreRow`, `Stack*`, `Row*`) wrap — never `Ellipsis`. `ChipBtn` ellipsis only for short strip chips. |
| **Min width** | Layout floor **320 css px**. Narrower viewports scroll horizontally (`app_hscroll` / `app_shell`). Desktop restore ignores saved widths below 720 so a squeezed session cannot lock the app into a phone frame. |
| **Button ink** | Shared faces use `padding` + `label_walk.margin` so labels never sit flush on the face edge. Long dual CTAs use `ActionStack`, not half-width `ActionRow`. |
| **Reports** | Today / month books, party aging, expenses, bank cashbook posts — generic books, not a vertical report suite. |
| **ClientPair** | Extra till until paired (IP + pairing code) |
| **DayClose** | Day book shortcut or Day close peer |
| **Bank book & P&L** | Day book shortcut or Reports peer — period P&L, aging, bank posts, expense voucher |
| **Parties** | Consignor / Carrier; AR settle; AP pay |
| **Settings** | Country pack, venue, **Units** + conversions, backup |
| **Pay out** | Bottom tab — carrier bill / purchase + expense voucher |

Owner order after unlock: **Day book** → **Booking** or **Pay out** as peers.

The shipping UI is Makepad using these DESIGN tokens. Full Stitch pack pixel parity is not required to ship.

## Work-surface layout & interaction (all Kumkaj apps)

Canonical for every Kumkaj app's screens (each app's doc owns only its
trade-specific instantiation — e.g. Khaynjo's menu/ticket split,
Courier's rates/cart split). Learned from shipped regressions; do not
re-learn per app.

- **The work surface gets the room.** The primary grid/list is the
  work surface; companion panes (ticket, cart, detail) stay lean.
  Side-by-side only when the content column is wide enough (≈ ≥ 640
  css px); below that, stack full-width — never a Fixed companion
  gutter with empty space.
- **Context lives on the work pane.** The state the operator needs
  mid-task (totals, mode chips, primary action) sits on the pane they
  are working, in short labels — one glance tells them what this
  record is.
- **Fixed small option sets are segmented grids, not scroll strips.**
  A closed set of choices renders all-visible as Fill chips; reserve
  scrollable `ChipStrip`s for open-ended sets (a half-hidden chip at
  common widths reads as a bug).
- **Fill minimums must fit the breakpoint.** Pane `min`s plus spacing
  stay well under the side-by-side threshold, or controls are pushed
  off the window edge.
- **No books jargon on operational work surfaces.** Books words
  (“Active ledger”, …) belong in Books/Reports, never in front of the
  operator mid-task; the work-surface header shows task state + Clear.
- **Empty states carry their next action.** Name the action that fills
  the pane (“Add …” → its screen), not just an explanation.
- **Whole-card tap targets.** List cards are the tap target (whole-card
  `finger_up` + `was_tap()` alongside `Button::clicked`), not just a
  small pill inside the card.
- **List rows keep one measured height.** Cap multi-line card content
  at draw (full text stays on the record) or the measured row
  oscillates.
- **Capability-aware chrome** (vision §30): peers for a capability
  show only when the business runs that capability; a business that
  never uses it never sees its screens.
- **Role-aware home**: unlock lands on Day book (the owner's four
  questions, [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)); the trade's
  work surfaces are one tap away.

Technical counterparts (state-guard deadlock, PortalList mechanics,
agent verification): [`DEV.md`](DEV.md) § Makepad 2 UI faces
(agent guide).

## Makepad 2 faces (agents — selection & truncation)

Full cause / approach / code map: [`DEV.md`](DEV.md) § Makepad 2 UI faces (agent guide). Cursor rule: `.cursor/rules/makepad-layout-faces.mdc`.

**Selection:** Makepad 2 `Button` has **no selected bit** — only hover/down/focus/disabled animators. Shipping pattern is Overlay **idle + active** faces; Rust paints visibility from app state (`paint_nav_tabs` / `paint_choice_pair`) and **cuts those animators** so a previous face cannot stay lit. Idle nav chips stay **transparent** on hover/focus; only the active face uses the navy “selected” fill. Each destination has its own peer (no More hub).

**Labels:** Never `max_lines: 1` + `Ellipsis` on full-width / multi-word / translated CTAs (`NavBtn`, `PayBtn`, `MoreRow`, `Stack*`, `Row*`). Those faces **wrap** (`height: Fit`, `flow: Right{wrap: true}`). `ChipBtn` ellipsis is allowed **only** for short strip chips (Fit max 160). Always set both `padding` and `label_walk.margin`.

**Form rows:** `FormFieldRow` = fields only. Multi-word CTAs (`Add rate`, `New item`, …) live in `ActionStack` below — never `NavBtn{width: Fit}` beside `FieldInput`s in one `Right` row.

## Color tokens (live)

Derived from `apps/courier/assets/courier-app-icon.png` (navy C path, sky blue, orange motion, yellow accents).

| Role | Hex | Use |
|---|---|---|
| Surface | `#D6E0EC` | App canvas (cool blue-gray) |
| Surface wells | `#ffffff` / `#EEF3F8` / `#E0E8F0` | Input wells, cards, secondary chips (white on canvas) |
| On-surface | `#0B1220` | Body ink on light faces |
| Muted | `#3A4554` / `#56575D` | Hints, placeholders (dark on **white** wells — never gray-on-gray) |
| Navy | `#02030B` / `#072058` / `#0A3A8C` | Chrome, tabs, primary actions (logo blues) |
| Selection | `#2E5BAD` | Text selection band (dark ink stays readable) |
| Orange | `#C44E10` / `#DF5E16` | Accents, price ink, warning fills (logo orange) |
| Yellow | `#F7B043` | Status marks on navy (logo yellow) |

**Rules**

- Cool blue-gray surface wins over parchment/cream for body canvas.
- White glyphs **only** on navy chrome (caption, primary CTAs, active tabs). Never light ink on light wells, placeholders, or hover.
- Caption **double-click** and the max button are the same maximize/restore toggle (see [`DEV.md`](DEV.md) § Desktop window size & maximize).
- Orange is accent ink/fill on light surfaces — not pale peach fills.
- Light faces: dark ink identical in rest / hover / focus / down.
- Navy faces: white ink identical in all states.
- Never flip ink lightness when the face flips (Makepad mixes colors mid-hover).

### Color tokens — Khaynjo (restaurant product)

Derived from `apps/khaynjo/assets/khaynjo-app-icon.png` (chef hat and
cloche on a red-orange gradient; sampled with ImageMagick). Same
contrast rules as above, with red chrome in place of navy: white
glyphs only on deep-red chrome (caption, primary CTAs, active tabs);
dark ink identical across states on light faces; never flip ink
lightness mid-hover.

| Role | Hex | Use |
|---|---|---|
| Surface | `#F9F4F0` | App canvas (warm cream) |
| Surface wells | `#ffffff` / `#F6E3D3` / `#EFD2BC` | Input wells, cards, secondary chips (white on canvas) |
| On-surface | `#260B08` | Body ink on light faces |
| Muted | `#6B4A3F` / `#8A6A5C` | Hints, placeholders (dark on **white** wells) |
| Deep red | `#9A0E10` / `#69090C` / `#B43A12` | Chrome, tabs, primary actions (logo reds; darkest → down, darkest-red → hover) |
| Selection | `#F69D2C` | Text selection band (dark ink stays readable) |
| Accent | `#B43A12` | Accents, price ink, warning fills (logo red-orange, darkened so white ink on it stays ≥ 5.9:1) |
| Gold | `#F69D2C` / `#F7B043` | Status marks on deep-red chrome (logo gold) |

**Contrast law (same as Courier — no light-on-light, no dark-on-dark):**
light faces always carry dark ink (`#260B08`/`#6B4A3F`) in every state;
deep-red faces always carry white ink in every state; ink lightness
never flips while a face flips. Verified pairings: white on `#9A0E10`
8.6:1, white on `#B43A12` 5.9:1, white on `#69090C` 12.8:1, `#260B08`
on `#F9F4F0` 17.0:1, `#260B08` on gold `#F69D2C` 8.6:1, idle nav ink
`#F2B183` on deep red 4.7:1. The logo's bright `#D64D21` is **not**
used under white text (4.3:1) — it survives only as the sampled
source colour, darkened to `#B43A12` for shipping faces.

## Contrast checklist (Makepad)

- Theme label ink = `#0B1220` (never white/cream as default).
- Placeholders = `#3A4554` (hover `#0B1220`) on **white** `FieldInput` wells; never gray placeholder on gray inset. Re-bind `draw_text.color_empty*` on stock inputs after theme (same bake issue as the caret).
- Pay / tab buttons on navy: white text in every state.

## Product cues that must stay obvious

- Empty rates: CTA to More → Rates & services (no sample rates pre-loaded).
- Extra tills: pairing code on Lock / Settings when allowed.
- Multi-branch: header shows `Shop · CODE` when more than one branch exists.
- Day book empty/first-day copy teaches money in, money out, parties — not “open Booking to begin.”
- Day books follow **user / business time** (`user_at`); device write / cloud accept times are separate ([`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) § Triple timestamps).
- Walk-in desk, AWB/barcode, tracking: product roadmap in [`KUMKAJ_COURIER.md`](../apps/courier/docs/KUMKAJ_COURIER.md).
