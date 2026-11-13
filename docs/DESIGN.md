# Kumkaj UI design (shipping)

> **Each app's design is its own (2026-10-23).** Kumkaj apps do not need
> to look or flow alike. Every product gets the design that gives *its*
> users the best experience — similar to a sibling or different, either
> is fine; great UX for that trade is what matters (vision §24). This file
> therefore holds two kinds of content: **lessons every app should know**
> (Makepad behaviour, layout and interaction rules learned from shipped
> bugs — technical truths, not a house style), and **Moklanjo's** shipping
> design below. Other apps keep their design in their own docs (e.g.
> [`KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md) §5b and its own colour
> tokens). Learning from competitors means doing better than them, not
> copying their screens.

Source pack (optional): `../../stitch_tactile_countertop_design_system/`.  
This file is the **canonical** in-repo summary. When Stitch packs disagree, **later shipping decisions win** (see also vision doc + [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)).

**Current product focus:** [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md) — UI names below use courier wording; other concrete products will override labels the same way.

## Navigation (shipping)

| Tabs | Role |
|---|---|
| **Day book · Booking · Pay out · Rates · Ops · …** | Moklanjo shell — no **More** hub; former More destinations are first-class peers |

| Tab | Purpose (Moklanjo) |
|---|---|
| **Day book** | Owner / Sales home after unlock — today’s money in/out, AR/AP, cash vs bank; hub shortcuts only for screens that person may open. Waiters and other limited roles unlock onto their first permitted peer (e.g. Floor / Order), not Day book |
| **Booking** | Money in — book parcels (rates × kg), three-total ledger, tender / party credit; Account vs Cash booking |
| **Pay out** | Money out — carrier bills / purchases and expense vouchers |
| **Rates · Ops · COD · Riders · Sales · Day close · Reports · Parties · Team · Settings · Lock** | Former More rows — same destinations, shown as scrollable bottom chips / rail peers |

Progressive depth for Moklanjo: **books + booking desk → AWB/barcode → tracking/manifest/COD → extra tills/team**. See [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md).

Booking is not the whole product; Day book and Pay out are peers.

### Launch / auth screens (outside tabs)

| Screen | When |
|---|---|
| **DeviceSetup** | **Every app start**. One screen: **language** + **Server Mode / Client Mode**. |
| **Lock** | Owner/Staff PIN. First time: **Server → Owner**, **Client → Staff** (when team exists); after that the last chip is remembered per books file (`lock_prefer_owner`). |

#### DeviceSetup — language (every launch)

- Screen opens in the **platform / OS language** so non-English users can read it.
- Options: **Use platform native language** (follow the device; updates if the OS language changes) or **Use user selected language** (catalogue list; platform language pre-selected as the default pick).
- Changing language **updates this screen immediately** (Server/Client labels and help text included).
- Choice is persisted when the user continues into Server or Client Mode. Settings can change it later.

#### DeviceSetup — Server Mode (offline cloudless LAN = **one** local server)

Offline / LAN Server–Client: **one shop server per install**. Server Mode creates it if missing, otherwise opens that one books file (`servers/{id}/…sqlite3` + `servers.json`). There is **no** create-another / open-list / remove-server picker on DeviceSetup.

| Situation | Behaviour |
|---|---|
| **No local server yet** | Server Mode **creates** the server and continues to Lock / Owner PIN. |
| **Server already exists** | Server Mode **opens** it (no list). |
| **Wipe local data** | **Owner** in Settings → Backup: same categories as Windows uninstall (**business records** / **shop setup** / **screen preferences**) + Owner PIN. Shop setup wipe returns to DeviceSetup. |
| **Emergency unlock PIN (duress)** | **Offline cloudless LAN only** (not a cloud-account feature). Separate from Backup wipe. Owner sets an optional second PIN on Team. Typing it at unlock wipes **books entries only** (sales/ledgers/history/open floor work) and opens as Owner looking normal — **menu/prices/tables/staff/settings/schema stay**. Does not delete the shop server. Vision §17. |

**Another outlet offline** = another device (or OS user profile) with its own install — not a second server inside this app. **Multi-server on one install** is reserved for a **future cloud** plan, not offline LAN.

**Client Mode** uses a separate client cache DB (`client/…sqlite3`) and pairs over LAN to a Server.

Pre-release: delete any older local data dirs (`com.kumkaj.app*`, root `kumkaj.sqlite3`, etc.) — there is no migration from those layouts.
| **Onboard** | Moklanjo: courier defaults baked in (Account/Cash booking, kg). Fresh vs Book for opening rates/parties. Other brand products bake their own defaults. |
| **Settings** | Country pack, venue, units, backup (no sample catalogue). A **hub** in both apps (each with its own sections and look — Moklanjo rows are `HubDestBtn` + mark): one section open at a time, Back closes it to the hub first. |
| **Control face by intent** | **Same job → same look.** Navigate elsewhere = `HubDestBtn` + mark (hub lists) or chrome `TabBtn`/`NavChip` — never a filled primary. Select in place = `DeskTab` / `SegBtn` / `ChipBtn`. Submit on this form = **one** filled primary (`RowPrimary` / `StackPrimary`, or `PayBtn` on the ticket). Quiet alt = secondary / outlined `NavBtn`. Destroy = `DangerBtn`. Frequency does **not** promote a navigate link to primary. |
| **Choice chips** | Short filter labels only (`ChipBtn` / `ChipBtnActive` in `ChipStrip`). Exclusive closed sets (tax lane, pay method, staff role, …) use **`SegBtn` / `SegBtnOn`**. Multi-word CTAs and one-shot actions are **not** chips. |
| **Stacked CTAs** | Multi-word / long labels → `ActionStack` + `StackPrimaryBtn` / `StackSecondaryBtn` (full width, wrap, no ellipsis). Same faces as Server/Client Mode. Never `ChipBtn` / `ActionRow` for those. Never use primary for “go to screen X”. |
| **Form field rows** | Side-by-side inputs only → `FormFieldRow` (equal `Fill` fields). **Never** put a Fit/`NavBtn` CTA in that row — Makepad has no flex-shrink; wrap stacks letters vertically (`A d d r a t e`) and overlaps the field. CTAs go in `ActionStack` under the fields. |
| **Bottom nav** | Phone chrome (< **720 css px**), horizontally scrollable. Active peer = only the `*Active` face visible (`paint_nav_tabs`); idle stays transparent on hover/focus. Peers include Day book · Booking · Pay out **plus** every former More destination (role-gated). |
| **Side rail** | Tablet / desktop chrome (≥ **720 css px**), vertically scrollable. Same peer set as bottom nav (`nav_rail` / `apply_shell_layout`). Bottom tabs hide; rail shows. Drag the divider to resize; rail floor **148** css px, main column floor **360** css px. **‹** on the rail collapses it to zero width; **›** in the header restores it (`rail_collapsed`). Phone bottom tabs are unchanged. Short chrome labels + wrap (no ellipsis). **Navigation:** browser-style visit history for peers and drill-downs; header ‹ whenever Back can run (hidden only on the first unlocked screen). Platform Back/Forward match ‹ — see `docs/DEV.md` § Native input. |
| **Booking / Order desk split** | Khaynjo sticky chrome = **only** Dine-in\|Takeaway\|Delivery\|Counter. Order title, total, table pick, Add dishes scroll inside Added-items. Parcel/Added-items \| Counter stay **side by side** in a Fit row inside horizontal `ScrollXView` (gap **16**). Pane widths Fixed in `apply_shell_layout`. Khaynjo: Mins stay **600 \| 520** — broad panes, the desk pans sideways (mouse press-drag from anywhere incl. text, or touch swipe). A 340 \| 380 trial (2026-11-13) cramped both panes and was reverted; **no jump buttons** either (tried and removed the same day). Moklanjo: booking top strip + same side-by-side at **every** width, phones included (owner, 2026-11-02): on a phone each pane is one screen wide less a 28 px peek of the other, swipe sideways between them (`booking_desk_pane_widths`). Shell chrome flips rail/bottom at 720. |
| **CTA labels** | Full-width faces (`NavBtn`, `PayBtn`, `MoreRow`, `Stack*`, `Row*`) wrap — never `Ellipsis`. `ChipBtn` ellipsis only for short strip chips. |
| **Min width** | Layout floor **375 css px**. Narrower viewports scroll horizontally (`app_hscroll` / `app_shell`) — no crop. Desktop window resize is floored at the same 375. Desktop restore ignores saved widths below 720 so a squeezed session cannot lock the app into a phone frame. |
| **Button ink** | Shared faces use `padding` + `label_walk.margin` so labels never sit flush on the face edge. Long dual CTAs use `ActionStack`, not half-width `ActionRow`. |
| **Reports** | Today / month books, party aging, expenses, bank cashbook posts — generic books, not a vertical report suite. |
| **ClientPair** | Extra till until paired (IP + pairing code). Body **Back** returns to DeviceSetup (auth screens hide the unlocked header — see DEV § Native input / Back). |
| **DayClose** | Day book shortcut or Day close peer |
| **Bank book & P&L** | Day book shortcut or Reports peer — period P&L, aging, bank posts, expense voucher |
| **Parties** | Consignor / Carrier; AR settle; AP pay |
| **Settings** | Country pack, venue, **Units** + conversions, backup |
| **Pay out** | Bottom tab — carrier bill / purchase + expense voucher |

Owner / Sales order after unlock: **Day book** → trade peers (Booking / Order, Pay out / Buy, Floor…). Limited roles skip Day book when they lack Books rights — first permitted chrome peer is home.

The shipping UI is Makepad using these DESIGN tokens. Full Stitch pack pixel parity is not required to ship.

## Soft keyboard (Android — every app)

When a text field is focused and the soft keyboard is open, the focused field
must sit **above** the keyboard (`KeyboardView` lift from the field rect + a
**measured** IME height). Hide snaps the shift to zero.

**Cause of the long regression:** on API 26–29 with `adjustNothing`, the
activity never reports IME height, so lift had nothing real to use. Guessed
heights, app-side re-lift, and “drop lift to stop glitches” each made the next
bug (beige gap, freeze, fields buried again). **Solution (Claude Opus, one
attempt, PR #1):** measure height with `MakepadImeProbe`, use stock
`KeyboardView`, snap hide to 0 — do not re-add estimates or app re-lift.
Full cause/solution table: [`DEV.md`](DEV.md) bug **6f** (also **6d**, **6e**).

## Always responsive (every app)

Layouts must fit the window they are in — no cropped columns, no
controls past the right edge, no “designed for one width” grids.

- Size grids and tile columns from the **widget’s turtle / walk width**,
  not a shell-wide estimate that ignores screen padding (Floor tiles
  cropped when `content_width` was used before padding — fix: measure
  in `draw_walk` like the dish grid). Prefer **big** tiles: Khaynjo Floor
  is **1 column from the 375 css px app floor**, more columns only when another
  ~280 px tile fits (`FLOOR_TILE_MIN_W`).
- Prefer wrap / reflow / scroll over Fixed widths that only fit one
  desktop size. Desk side-by-side only when the content column is wide
  enough; otherwise stack. Sole `InfoHelp` headings: titled = Fit title +
  tip + Fill trail (ℹ hugs the label); tip-only InfoHelp stays Fit.
- Phone (< **720**) and tablet/desktop (≥ **720**) chrome differ
  (bottom tabs vs side rail); content inside each mode still reflows.
- **Page-scroll lists:** expand-to-content `PortalList`s must not use a
  Fixed pitch shorter than the real Fit row — that crops the last cards
  into the next block and makes the page look unscrolled. Measure
  (or match a Fixed DSL card height). Outer page `ScrollYView` owns
  wheel **and** mouse/touch press-drag (`mouse_capture_overload`); do
  not nest a cart/list fling on those lists. Sideways chip strips
  (`ChipStrip`, `FormScrollX` — Order categories, filters) pan the same
  way: a mouse press **on a chip** and drag pans the strip
  (`scroll_bar_x.mouse_capture_overload: true`), a click still picks the
  chip. Without it a mouse could drag only from the gaps between chips. See `docs/DEV.md` §13 /
  bug 8.

## Text selection & copy (every app **and** every Makepad website)

**One model, everywhere (2026-11-13).** It replaced a stack of 14
partial Makepad patches whose "document host" took the press away from
the blocks and guessed blocks from their text, leaving some cards
selectable and their neighbours dead (Khaynjo.com feature grid,
KOT cards, right-column cards) — so the site had selection switched off
and the app's Ctrl+A did nothing. Makepad side: one patch,
`patches/makepad/text_selection.rs.patch`; app side: each app's own
`src/selection_menu.rs` (Khaynjo first).

| What | Rule |
|---|---|
| **What is copy** | Display text a person reads: headings, body, totals, figures, report rows, empty states, help, session line, site copy. It is a selectable block: `WrapLabel` / `ReportText` rows (apps), `Markdown` typography (sites). Never a plain `Label` for display text. |
| **What is not** | Text **inside a tap target** (floor tiles, cart lines, KOT cards, nav chips, site tiles with a hit button, rows of a tapped list — `ReportText{tappable: true}`: Settings → Areas · stations · tax rates, Units, printer pool), button faces, and chrome marks (shop name / role on the maroon header, badges, step-number discs, footer brand). `selectable: false` / plain `Label`, with a one-line comment saying which. Also text right beside a PIN / pairing field on Lock / DeviceSetup: a tap on copy takes key focus and would drop the soft keyboard. |
| **Drag selects like a browser page** | **Apps** (owner, 2026-11-13): a mouse press-drag **pans** pages, the Order desk and chip strips exactly as before — even when it starts on text; mouse selection starts on a **0.5 s still hold** then drag (same as touch), double-click (word), triple-click (block), Ctrl/Cmd+A or the right-click menu (`text_flow::MOUSE_HOLD_TO_SELECT`, set at app start). **Sites:** mouse press-drag on text selects. From there, in both: the selection **keeps going across blocks** in reading order — the rest of the first block, every block between (cards, columns, sections), the part of the block under the pointer; back inside the first block it shrinks to that block. Double-click = word, triple-click = the block. On a web page a drag may also **start in a margin** (`select_from_gaps`); in an app a drag on whitespace scrolls the page instead. While dragging, the **wheel scrolls** and the selection follows; the pointer **past the top / bottom edge auto-scrolls**. Touch (**Chrome on Android, 2026-11-13**): a swipe always scrolls; a **0.5 s still hold selects the word** under the finger on every touch platform (the OS long-press on Android only reached a block that still owned the touch, so Android never selected — the OS event stays a second trigger); a finger just past a word's last letter takes that word; drag on to extend it (across blocks too). On lift the selection shows **two blue knobs** under its ends (`draw_sel_handle` on the root) and the menu **above** it, centred; drag a knob to move that end — across blocks, the other end holds — and the menu comes back on lift. A **swipe keeps** the selection (it scrolls with the page, even when the swipe starts on a row that takes focus); a **tap** anywhere else clears it (a tap on a button still works); Copy clears it. Every block keeps its own press, so none can be dead; blocks are found by `selection_leaf()`, never by text heuristics. |
| **Select all** | **Ctrl/Cmd+A** with no text field focused selects **every selectable block the screen shows** (one `View { selection_root: true }` per window/page: Khaynjo `main_column`, the site's `page`; a sheet over the page — Add dishes, UPI QR — is its own root while open). A focused text field keeps Ctrl+A for itself. |
| **Copy** | **Ctrl/Cmd+C** copies exactly what is highlighted: one block's part, a drag across blocks, or after Select all every block — in reading order, one block per line. Any press clears a page-wide or cross-block selection; every screen change clears it too. |
| **Menu** | Right-click (mouse) **on text or on a live selection** opens **Copy / Select all** at the pointer; on a control, a tile or blank page it does nothing (no menu over buttons). After a touch selection (and after each knob drag) the menu is for the **whole** selection, above it: Android / iOS show the OS bar (its Select all is Ctrl+A); desktop touch shows the app's menu (`TextSelectionAction::Menu { rect }`); a browser shows the site's menu. Copy in every menu runs the same path as Ctrl/Cmd+C, so it copies exactly what the keys would. App menu strings are translated like every other string. |
| **Band** | The selection band sits **under** the ink: light `#CFE3FF` under dark ink (app copy and fields alike — `draw_selection.color`, not `draw_block.selection_color`), a dark band (`#3D0507`) under light ink on red bands. Never a band that hides the text it marks. |

Ctrl/Cmd+C uses the system clipboard (`Cx::copy_to_clipboard`; Khaynjo
also writes through `arboard` like its other Copy actions — see
[`DEV.md`](DEV.md)). On the web canvas the right-click menu replaces the
browser's own (which only offers "Save image as…").

**Do not** add a per-widget selection exception beyond the tap-target /
chrome-mark rule above, take a block's own press away (the old
`selection_gestures: false` host made blocks dead), or ship a
display paragraph as a plain `Label`. A new screen gets selection for
free from its `WrapLabel`s and the window's root.

**Open (khaynjo.com):** fill the 14 world-shelf “Photo coming” countries —
see [`sites/khaynjo/README.md`](../sites/khaynjo/README.md) § Remaining.

## Work-surface layout & interaction (lessons for every app)

Lessons learned from shipped regressions — how Makepad lays out and
behaves, and what broke. Every app should know them so nobody re-learns
them; they are not a shared look. Each app decides its own screens, flows
and visual design for its users (e.g. Khaynjo's menu/ticket split and
coloured floor, Moklanjo's rates/cart split).

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
- **Lists and summaries are cards with rows, never run-on text.** Several
  items or figures (units, printers, audit, history, open bills, a day's
  totals) go in a card — a heading, label/value rows with the value at
  the right, plain lines — or a list widget; only a sentence stays a
  paragraph. Each app has its own card (`ReportText` in Moklanjo and
  Khaynjo). A list with nothing in it says so in a short hint — never
  an empty card.
- **Whole-card tap targets.** List cards are the tap target (whole-card
  `finger_up` + `was_tap()` alongside `Button::clicked`), not just a
  small pill inside the card.
- **List rows keep one measured height.** Cap multi-line card content
  at draw (full text stays on the record) or the measured row
  oscillates.
- **Capability-aware chrome** (vision §30): peers for a capability
  show only when the business runs that capability; a business that
  never uses it never sees its screens.
- **Role-aware home**: unlock lands on the **first chrome peer that
  person may open** (`unlocked_home_from` / `nav_tab_permitted`). For
  Owner / Sales that is usually Day book (the owner's four questions,
  [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)); for a Waiter with Floor +
  Order only, Floor (or Order) — never a Day book full of money hubs
  they cannot use. Hub rows and money sections use the **same** rights
  as nav (do not regress — [`DEV.md`](DEV.md) § Staff rights vs chrome).

Technical counterparts (state-guard deadlock, PortalList mechanics,
agent verification): [`DEV.md`](DEV.md) § Makepad 2 UI faces
(agent guide).

## Makepad 2 faces (agents — selection & truncation)

Full cause / approach / code map: [`DEV.md`](DEV.md) § Makepad 2 UI faces (agent guide). Cursor rule: `.cursor/rules/makepad-layout-faces.mdc`.

**Intent first (navigate ≠ select ≠ submit ≠ destroy):**

| Intent | Face |
|---|---|
| Leave this screen / peer jump | `HubDestBtn` + colored mark (Day book and similar hubs); chrome `TabBtn` / `NavChip` |
| Pick among options here | `DeskTab` (page tabs) · `SegBtn`/`SegBtnOn` (closed radio set) · `ChipBtn`/`ChipBtnActive` (short filter only) |
| Commit work on this form | **One** `RowPrimaryBtn` / `StackPrimaryBtn` (or ticket `PayBtn`) |
| Cancel / clear / quiet alt | `RowSecondaryBtn` / `StackSecondaryBtn` / outlined `NavBtn` (full width); inline Fit quiet → `QuietFitBtn` (never `Row*` at `width: Fit`) |
| Void / refund / delete | `DangerBtn` |
| Long / freezing work in flight | **Replace** the start CTA with **Progress (state) + Cancel** until done or cancelled — never leave one button that freezes the UI (pair, find server, open books, find printers, import, backup, …). Progress on a WrapLabel / Fit face; Cancel is `QuietFitBtn`. Work runs off the UI thread. Canonical: [`DEV.md`](DEV.md) bug **7b**, `.cursor/rules/busy-action-chrome.mdc`. |

Never promote a hub navigate link to filled primary because it is “the main thing.” Day book’s five jumps (Book a parcel / Sell, Expense / Buy, Day close, Reports, Parties) are **equal** `HubDestBtn` rows.

**Selection:** Makepad 2 `Button` has **no selected bit** — only hover/down/focus/disabled animators. Shipping pattern is Overlay **idle + active** faces; Rust paints visibility from app state (`paint_nav_tabs` / `paint_choice_pair`) and **cuts those animators** so a previous face cannot stay lit. Idle nav chips stay **transparent** on hover/focus; only the active face uses the navy “selected” fill. Each destination has its own peer (no More hub).

**Labels:** Never `max_lines: 1` + `Ellipsis` on full-width / multi-word / translated CTAs (`NavBtn`, `PayBtn`, `MoreRow`, `HubDestBtn`, `Stack*`, `Row*`, `DangerBtn`). Those faces stay **centered** (hub dest left-aligned) and **wrap** (`height: Fit`, `flow: Right{wrap: true}`). Label walk is **Fit** (Makepad patch `button_label_center` — a Fill label inside wrap freezes the UI; do not remove wrap/center to dodge that). `ChipBtn` ellipsis is allowed **only** for short strip chips (Fit max 160). Always set both `padding` and `label_walk.margin`.

**Snackbar colour = what happened (owner, 2026-11-13).** Only an **error** is
red. A warning is **amber** with dark ink, information **slate**, a success —
including "Station Tandoor added — attach its printers: …" — **green**. A
next-step note in brand maroon read as "you did something wrong". Messages
already in the snackbar are not repeated as a line on the screen (the Order
ticket's status line under + Add dishes is gone). Classification:
`snack::classify` (words), faces in `ui_script.rs` `app_snackbar`.

**Names people pick by are unique (owner, 2026-11-13).** Ids are the key
everywhere and most names may repeat, but **kitchen stations and dining
tables** are told apart by name (KOT papers, Floor, bills, pickers): a second
"Tandoor" or "D1" (same branch) is refused at the store
(`Store::name_taken`) — add and rename alike. The match is **case-sensitive**
(owner, 2026-11-13): "tandoor" next to "Tandoor" is allowed; only surrounding
spaces are ignored.

**Explanations live in ℹ, not in paragraphs (owner, 2026-11-13).** The ℹ
exists so screens stay clean: what a screen, section or field does goes in
the InfoHelp tip **beside its heading** — never as a grey paragraph under it,
and never as a line restating what the controls already show (the selected
Start fresh button, the logo image, an Allow button, the field placeholder).
Status lines carry state and results only, short ("Stamp: on file",
"Added 121 dishes…"). A heading with only an explanation under it becomes
heading + ℹ (Alerts, Trading history, Delete local data, Shop templates,
How do you want to start?). Before adding text, ask: if it explains, it is
a tip.

**InfoHelp (screen / section titles):** Titled rows are **`width: Fill`** with **`WrapLabel{width: Fit}` + tip + Fill `trail`** so ℹ sits **next to** the label on every screen (never at the far end — Day book / LANGUAGE, 2026-11). Gap is `tip_btn.margin.left` (6). **Tip-only** rows hide the trail and force **Fit**. Never `Label{width: Fill, wrap}` (turtle NaN, DEV bug 7). Empty title stays **hidden**. Guarded by `info_help_layout_tests` in each app — **fix the InfoHelp face, not one screen.**

**Generous hit targets (most places):** Accordions, Settings hub rows, sidebar cards, and checkbox / radio / switch rows should respond on the **whole face or label row** — not only a chevron, 18 px box, or pill thumb (DEV bug **7c**). Keep tip ℹ **next to** its title (Fit caption + Fit InfoHelp); extend the tap with a Fill `_ext` sibling when the card is wider than that cluster — use **`AccordionHdrHitBtn`** (no hover paint) so the row still looks like **one** header, not title / ℹ / empty slab. **Exceptions are intentional:** a narrower hit is fine when a full-row tap would fight another gesture (sidebar show/hide pill vs drag-reorder; checklist ↑↓ vs check) or make the screen worse — say so in a short comment.

**Form rows:** `FormFieldRow` = fields only. Multi-word CTAs (`Add rate`, `New item`, …) live in `ActionStack` below — never `NavBtn{width: Fit}` beside `FieldInput`s in one `Right` row. Never put `width: Fill` CTAs inside a parent `flow: Right{wrap: true}` View (same freeze); stack them with `ActionStack` / `flow: Down`.

## Color tokens (live)

Derived from `apps/moklanjo/assets/moklanjo-app-icon.png` (navy C path, sky blue, orange motion, yellow accents).

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
| Selection | `#CFE3FF` | Text selection band (ServedFill / light blue; dark ink stays readable) |
| Accent | `#B43A12` | Accents, price ink, warning fills (logo red-orange, darkened so white ink on it stays ≥ 5.9:1) |
| Gold | `#F69D2C` / `#F7B043` | Status marks on deep-red chrome (logo gold) |

**Contrast law (same as Moklanjo — no light-on-light, no dark-on-dark):**
light faces always carry dark ink (`#260B08`/`#6B4A3F`) in every state;
deep-red faces always carry white ink in every state; ink lightness
never flips while a face flips. Verified pairings: white on `#9A0E10`
8.6:1, white on `#B43A12` 5.9:1, white on `#69090C` 12.8:1, `#260B08`
on `#F9F4F0` 17.0:1, `#260B08` on gold `#F69D2C` 8.6:1, idle nav ink
`#F2B183` on deep red 4.7:1. The logo's bright `#D64D21` is **not**
used under white text (4.3:1) — it survives only as the sampled
source colour, darkened to `#B43A12` for shipping faces.

### Color tokens — Kirano (grocery product)

Derived from `apps/kirano/assets/kirano-app-icon.png` (orange cart,
leaf green, cream field — sampled with ImageMagick). Same contrast
rules: white glyphs only on orange chrome; dark ink on light faces in
every state; never flip ink lightness mid-hover.

| Role | Hex | Use |
|---|---|---|
| Surface | `#FDF7EA` | App canvas (warm cream) |
| Surface wells | `#ffffff` / `#FCE1B4` / `#C2B9A9` | Input wells, cards, secondary chips |
| On-surface | `#031303` | Body ink on light faces |
| Muted | `#1C460E` / `#56575D` | Hints, placeholders (dark on **white** wells) |
| Orange chrome | `#CD380E` / `#F96303` | Chrome, tabs, primary actions (logo oranges; deepest for down/hover) |
| Selection | `#A7A536` | Text selection band |
| Leaf green | `#1C460E` | Accents, status marks on cream / price ink |

### Color tokens — Tabiyat (pharmacy product)

Derived from `apps/tabiyat/assets/tabiyat-app-icon.png` (teal field,
lime cross, white type — sampled with ImageMagick). Same contrast
rules: white glyphs only on teal chrome; dark ink on light faces in
every state.

| Role | Hex | Use |
|---|---|---|
| Surface | `#FBFCFC` | App canvas (cool white) |
| Surface wells | `#ffffff` / `#A2CFCF` / `#E8F4F4` | Input wells, cards, secondary chips |
| On-surface | `#023637` | Body ink on light faces |
| Muted | `#025B60` / `#074D3E` | Hints, placeholders |
| Teal chrome | `#023637` / `#025B60` / `#017B85` | Chrome, tabs, primary actions |
| Selection | `#37BE28` | Text selection band (dark ink stays readable) |
| Lime | `#37BE28` | Accents, medical cross cue, status marks on teal |

## Contrast checklist (Makepad)

- Theme label ink = `#0B1220` (never white/cream as default).
- Placeholders stay muted on **white** `FieldInput` wells in every state
  (Moklanjo `#56575D`, Khaynjo `#8A6A5C`) — never jump to body ink on
  hover/focus, or empty fields look pre-filled. Re-bind `draw_text.color_empty*`
  on stock inputs after theme (same bake issue as the caret).
- Pay / tab buttons on navy: white text in every state.

## Product cues that must stay obvious

- Empty rates: CTA to More → Rates & services (no sample rates pre-loaded).
- Extra tills: pairing code on Lock / Settings when allowed.
- Multi-branch offline: one Server install (or OS user) per outlet (vision §5); the header shows that shop's own name. Multi-server on one DeviceSetup is cloud-only later.
- Day book empty/first-day copy teaches money in, money out, parties — not “open Booking to begin.”
- Day books follow **user / business time** (`user_at`); device write / cloud accept times are separate ([`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md) § Triple timestamps).
- Walk-in desk, AWB/barcode, tracking: product roadmap in [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md).
