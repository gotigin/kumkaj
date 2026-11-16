# Bringing Moklanjo, Kirano and Tabiyat to Khaynjo's maturity (2026-11-14)

Owner brief (2026-11-14): the three apps keep their own design and concepts
(never a copy of Khaynjo's screens), but get Khaynjo's **maturity**: how bug-free
and carefully it is built, the shared lessons, its type sizes, spacing and
responsiveness; the same test suites; a way to run the **Windows App
Certification Kit (WACK)**; then a full audit of all four apps with fixes.

This file is the tracker. An agent that picks the work up continues from the
first unchecked row and keeps the rows honest (what was verified, how).

## Status

| # | Item | Moklanjo | Kirano | Tabiyat | Notes |
|---|---|---|---|---|---|
| 1 | Mouse drag pans, mouse selection on a 0.5 s hold (`MOUSE_HOLD_TO_SELECT` at startup) | ☑ | ☑ | ☑ | Without it a drag on text **selects** instead of panning — the regression the owner rejected in Khaynjo (2026-11-13) |
| 2 | One selection root (`main_column`) + Copy / Select all menu (`selection_menu.rs`), screen change clears it, 3 strings in 110 packs | ☑ | ☑ | ☑ | DESIGN § Text selection & copy |
| 3 | Dish / product photos decode off the UI thread (`load_thumb_at` async + `handle_thumb_decodes`) | ☑ | ☑ | ☑ | Khaynjo: first + Add dishes 2.2 s → 0.19 s |
| 4 | Loading screen for a screen switch that took ≥ 1 s last time (`go_with` → `nav_measure`, `nav_loading`) | ☑ | ☑ | ☑ | DEV UI-thread item 14 |
| 5 | `text_field_has_focus` counts a field only while it owns the IME rect; a screen change drops a hidden field's focus; cart scan uses `get_item` (also Khaynjo) | ☑ | ☑ | ☑ | Khaynjo: header said "Locked" after a PIN unlock |
| 6 | Type sizes, spacing, responsive breakpoints match Khaynjo's tokens | ☑ | ☑ | ☑ | Breakpoints were already shared (720 / rail 220 / desk 600 \| 520). Kirano / Tabiyat components already matched; Moklanjo NavBtn, RowSecondaryBtn (15 → 14 pt), Stack*Btn, chips, role chips, customer / sale rows now use Khaynjo's padding. Moklanjo's own cards / cart rows / FieldInput keep their design |
| 7 | Same test suites: shell_tests guards, user journeys, visual baselines (`cert/baselines`) | ☑ | ☑ | ☑ | Ported the Khaynjo guards that describe these apps (Kirano / Tabiyat +6 shell tests → 58, + `screen_preview_pixels_are_the_printed_png`; Moklanjo +4 → 27) — each one first failed on a real gap that was then fixed (FormScrollX, brand stacks, chip heights, tappable rows / picker names). Not ported (Khaynjo-only screens): kitchen / floor / order-type 2×2 / PIN prompt label / cart pitch / report Cols. User journeys were already shared. Visual baselines pinned 2026-11-14 (5 each: unlocked, rail collapsed / expanded, a work screen, Settings) from green `--tier=visual` runs |
| 8 | WACK: packaging + `scripts/windows/run-wack.ps1` + DEV.md § Windows App Certification Kit | ☑ | ☑ | ☑ | All four: exe manifest / version info / icon (`scripts/windows/win_resources.rs`), Makepad `windows_end_session` patch, `product-cert.sh --tier=wack` (Linux pre-check, PASS ×4 2026-11-14), `run-wack.ps1` for the real run on Windows — **not yet run on a Windows machine** |
| 9 | Full audit (all four) and fixes | ◐ | ☑ | ☑ | Done: every Khaynjo fix since the 2026-11-08 fork checked against the three (list below), runtime DSL errors, hard-coded UI strings (none), `app_t` under a guard (new lock-guard rule, clean ×4), list walks off-screen, todo!/unimplemented! (none). Kirano / Tabiyat: Khaynjo's floor / kitchen code removed (owner ask, 2026-11-14 — § Floor and kitchen removed below). Open: Moklanjo popup placement has no tests (`place_popup` is Khaynjo's) |

## Lessons to carry (from Khaynjo)

Read with `docs/DEV.md` (UI-thread rules, bug lists) and
`docs/KHAYNJO_APP_REDESIGN.md` § 3.

- Never call `app_t` / anything that locks `app_state` while holding the
  guard (`catalogue_download_progress_text` froze the UI).
- Measure freezes on the **debug** build the owner runs, per tap; the stall
  meter logs ≥ 250 ms spans.
- Draw-time decodes (images, PNG previews) go off the UI thread or are cached
  by content key.
- A scan that walks list items runs only on the screen that shows them, and
  only over drawn rows (`get_item`, never `item`).
- Help text lives in the heading ℹ, never inline paragraphs.
- Snackbar colours: only errors are red.
- Unique names (stations, tables) are compared case-sensitively, trimmed.
- Every new string goes into `en()` and all 110 packs.

## Found and fixed on the way (2026-11-14)

- **Kirano / Tabiyat:** `FormScrollX` was used (Parts pay methods, Buy pay
  mode) but never defined — a runtime DSL error, both strips never built
  (since before 419af3a). Defined as in Khaynjo.
- **Kirano / Tabiyat / Moklanjo:** `ChipStrip` lacked the mouse press-drag pan
  (`mouse_capture_overload`) Khaynjo got on 2026-11-13.
- **Moklanjo / Kirano / Tabiyat:** shop name + server id sat Fit beside each
  other in a Right row on the header, lock and setup screens (cropped on a
  375 px phone — Khaynjo fixed it 2026-11-09): one Fill stack now.
- **Kirano / Tabiyat:** Take payment methods were five Fill SegBtns (Split
  half hidden on a phone — rejected in Khaynjo): fixed chips in a sideways
  `till_pay_scroll`. Scroll bars used Khaynjo's red (fork leftover): app colours.
- **Moklanjo:** `ChipBtn` pinned `height: 34` → Fit.
- **Kirano / Tabiyat:** snackbar classifier lacked Khaynjo's "already exists"
  = warn and "added / created / updated / removed / applied" = success.
- **Moklanjo / Kirano / Tabiyat:** a Windows panic vanished with the GUI
  subsystem; now one message box with the `last_panic.txt` path (Khaynjo 1.0.5).
- **Kirano / Tabiyat:** page-window lists kept the pre-resize width (no redraw
  when `page_viewport` / `floor_hviewport` change) — Khaynjo finding 4.
- **All four:** `text_field_has_focus` built every cart line (`list.item`) on
  each call; now `get_item`.
- **Kirano / Tabiyat:** Settings → Units and Areas · stations · tax rates rows
  never took a tap (a View emits FingerUp only with a cursor) and showed raw
  ids ("tax:<uuid> · GST 5%"): `ReportText{tappable: true}` +
  `sibling_picker_keys` (Khaynjo 2026-11-13).
- **Kirano / Tabiyat:** Bill & KOT preview PNG round trip on every repaint and
  a duplicate `sync_bill_kot_tabs` on open (Khaynjo's 1.4 s freeze): pixels +
  content key (`show_slip`, `slip_page_gray`, `slip_preview_key`).
- **Kirano / Tabiyat:** the Floor tile walk ran on every action on every
  screen and built list items (Khaynjo's first-actions freeze): only on Floor,
  only drawn rows.
- **Kirano / Tabiyat:** Bill & Slip format form brought to Khaynjo's
  (2026-11-15): one list of bill lines, the open line edited where it sits,
  item columns and totals one row each, the side preview beside the open line
  on wide screens, every row responsive (`layout_slip_editors`,
  `align_slip_previews`); the hidden print-order list and the old
  column picker are gone. Ported from Khaynjo's code (identical before), with
  each app's colours and widgets.
- **Open — Kirano / Tabiyat bill Logo** (2026-11-15): Khaynjo's Bill tab
  has a Logo line (PRINTER.md § Logo, lesson 23); the two apps share the
  Bill form and need the line, the picture line in the raster and the books
  ops ported.
- **Done 2026-11-16 (was open) — Moklanjo / Kirano / Tabiyat OS document on CUPS** (2026-11-15):
  they still send the older picture job — Kirano / Tabiyat
  `print-scaling=none` under 180 mm, Moklanjo the bare slip fitted to the
  page — so the slip is centred on the sheet (a blank band above it) and
  Moklanjo's fills the page at any width. Since 2026-11-16 Khaynjo prints
  OS documents at real size on a page exactly the printer's paper (owner's
  model: paper per printer on Settings → Printers, filled by Find, never
  lost; the Bill / KOT preview drawn on any printer's paper — `os_paper.rs`,
  `NamedPrinter::paper_width_mm`, Windows `windows_slip_script`, Android
  `printSlipPages`; PRINTER.md § Paper and preview, lesson 24) — that is
  the path to port, not the 210 mm page (it assumed A4). The Android Java
  and JNI are shared (patches), so the three apps only need their Rust side.
  The dropdown face fix (`ComboFace` / `ComboInput`) is already in all four.
  **Ported 2026-11-16** (owner: "update them all … all the three apps"):
  `os_paper.rs`, the printer's paper on `NamedPrinter`, Find asking each
  driver, the real-size send (CUPS / Windows / Android `print_slip`), the
  Printers list `name · paper` opened in place (`PrinterPaperRows`, any
  number of printers) and the preview on any printer's paper — Kirano /
  Tabiyat on the Bill tab, Moklanjo for the focused job (receipt or AWB
  label) on Settings → Printers (`SlipPicture`). Each app's 18 paper tests
  and suites green, `--tier=fast` PASS ×4, the screens checked on Xvfb.
- **Open — Kirano / Tabiyat sample bill and Table line** (found 2026-11-16):
  the Bill form still has Khaynjo's **Table** line, and the sample bill
  prints "Dine In: D1" with Tea and Coffee (`render_sample`) — restaurant
  words in a grocery / chemist app. Remove the line and use each shop's own
  sample goods (owner rule: no restaurant code in these two, 2026-11-14).
- **All four:** `check-lock-guards.sh` now also flags `app_t(` under a live
  `app_state()` guard (the Menu template download deadlock class) — clean.
- **Android (all apps, shared Makepad):** touch selection on the AGS2 — held
  word cut by finger jitter, OS bar without Copy, Cut / Paste on read-only
  copy, Select all closing the bar. DEV.md § Text selection.

## Floor and kitchen removed from Kirano and Tabiyat (2026-11-14)

Owner: "Kirano and Tabiyat still carry Khaynjo's floor and kitchen code behind
their counter-only switches — just clean this, remove it." Both apps are now
counter shops in code, not only in settings. Same change in both (Tabiyat was
ported from Kirano's diff, then checked for parity).

Removed — shell:
- Floor and Kitchen screens, their nav peers, slots and icons; the
  `venue_mode` switch everywhere.
- Sell: table pick / chip, order-type strip, Send KOT / reprint / batch
  cancel / print cancelled, kitchen-station picker, table actions (move,
  merge, split, split evenly, fire course, hold, call waiter), the sent-line
  panel (cancel / NC / move); on each cart line the Hold / In print / Mark
  buttons, the KOT group header and the "→ kitchen" destination.
- Team: Waiter and Store (kitchen) roles, Shop / Sections features, the
  "Cancel a sent item", "Complimentary", "Move / merge tables" and "Print
  bill again" rights, the section checklist.
- Settings: kitchen "sections" (Settings → Pickers is now Tax rates), the
  Bill & KOT panel's KOT tab (KOT slip format), KOT auto-print, the
  station printer routes; item form "Shelf / location" and area prices
  (both only worked through tables / kitchen stations); delivery-by.
- Waitlist, reservations, floor columns / zoom / areas, the floor dropdown
  popup and the nav drag ghost.

Removed — library:
- Tables, areas, running orders, KOTs, kitchen stations, reservations,
  waitlist, waiter calls, courses / holds / splits; the store code for them,
  the 14 books ops, the LAN routes `/kot*`, `/running`, `/table_billing` and
  the guest `/kiosk`, `/expo`, `/tokens`, `/t/…` pages; tables / KOTs /
  stations / running orders in the LAN snapshot and live patches; the KOT
  print kind, kitchen routes and KOT slip format; `VenueMode` (new books no
  longer seed `venue_mode = restaurant`); finalize's table argument and the
  KOT tickets it wrote; Waiter / Kitchen roles, the four table / KOT rights
  and the floor / kitchen / KOT-station staff features.
- 292 unused restaurant strings from `locale.rs` and all 110 packs.

Kept on purpose (printing compatibility): the `kot` print format id (a bill
printer may use that narrow slip) and saved kitchen routes in
`printers.json`, the shared slip style types still named `Kot*`. The
restaurant SQLite tables / columns and the cart-line / sale KOT and table
fields were kept here at first, then removed with the migrations (next
section).

Tests: restaurant tests removed (Kirano lib 344 → 246, shell 58 → 47,
journeys 11 → 10); the LAN retry / concurrency and phone-queue tests now
send counter sales instead of KOTs. `--tier=visual` re-run after each layer.

## Onboarding Client, one-app features, no migrations, dropdowns, printers (2026-11-14)

Owner: "Only when for the first time onboarding, client option is selected
then it takes too long and the ui is like locked and cancel button doesn't
works … features which are concrete and specific to one app … aren't
duplicated … We haven't published other 3 apps so we can still clear the
migration and old books support." Then: dropdowns keep "Done" after they
close and do not look like dropdowns; and wherever the owner is sent to
Settings → Printers to add a printer, give them Find printers right there.

**Client tap on first onboarding (all four):**
- Cancel used to wait for the open worker, which on a first Android install
  also competed with the menu-photo unpack (Khaynjo 1,459 files / 143 MB).
  Every role open and every Cancel now starts a new generation
  (`role_open_gen`): Cancel returns to Server / Client at once, and a worker
  that finishes later sees it is stale and drops its result off the UI
  thread. Verified on Xvfb with the Client open slowed to 8 s: back on the
  role picker 0.6 s after Cancel, still there 9 s later.
- The bundled item photos are unpacked only for a Server (books open or
  Server chosen) — a Client never waits for them.

**One-app features removed from the apps that do not need them:**
- Kirano / Tabiyat: Khaynjo's Rajdhani App Template offer (it could never
  appear), the Zomato / Swiggy aggregator file import, recipes and raw
  materials (`store_recipes.rs` → `store_reorder.rs`, reorder levels only),
  allergens, order types (dine-in / takeaway / delivery) and delivery-by,
  the cart-line KOT / station / running-order fields and every remaining
  table / KOT SQL; Kirano also Tabiyat's Schedule H / Rx flags.
  `store_running.rs` (bill settings by then) is `store_bill.rs`.
- Kirano / Tabiyat ship their own starter catalogues (neighbourhood grocery,
  chemist counter); a Khaynjo counter-only rule had hidden their Import
  buttons. Shown again — Kirano imported all 504 items with photos on a
  copy of older test books.
- Tabiyat: the chemist strings (barcode, composition, Rx note, Schedule H /
  Rx toggles, batch, expiry) were in every pack but never applied — wired.
  Moklanjo's "AWB" barcode stub left Tabiyat.
- Rajdhani names out of Kirano / Tabiyat tests (`Asha Stores`), the
  Rajdhani image script out of both apps; 172 (Kirano) / 182 (Tabiyat)
  strings nothing used out of `locale.rs` and all 110 packs.

**No migrations (Moklanjo, Kirano, Tabiyat — Khaynjo keeps its ladder):**
- One schema per app (`src/store_schema.rs`, `CREATE … IF NOT EXISTS`),
  generated from a fresh file of the previous build and checked column by
  column (same order, same types / defaults). Gone: every `ALTER TABLE …
  ADD COLUMN` list, `migrate_ops` / `migrate_courier_ops_tables` /
  `migrate_local_extras_tables`, the one-shot KOT close, the branch
  back-fill, the staff-cache one-shot, the pre-per-user amount fallback,
  and the install-level `printers.json` copy / old `{kot, bill}` /
  `{receipt}` formats.
- Kirano / Tabiyat also dropped 13 restaurant tables (tables, KOTs, running
  orders, kitchen stations, floor areas, reservations, waitlist, waiter
  calls, split tables, recipes, stock carry, delivery) and their columns
  (sale table / order type, item station / area prices / allergens, feedback
  table, LAN outbox table). Every dropped column had a default, so books
  from an earlier test build still open (checked on a copy). Tabiyat keeps
  the column name `allergens` for the Rx note for the same reason.

**Dropdowns (all four):** every control that opens a pick list sits in
`ComboField`, which draws ˅ / ˄ from app state at draw time; dropdown faces
are their label only (`Done · …` removed — it stayed after the list closed
another way). DEV.md § Dropdowns rule 9.

**Find printers where a printer is missing (all four):** printer checklists
with nothing saved show a Find printers row; "no printer" warnings carry a
Find printers button; both open the Printers sheet (Find / Cancel / saved
list; Done reopens the checklist). Moklanjo's sheet saves what Find found
and gives the tapped printer the missing job. DEV.md § Dropdowns rule 10,
PRINTER.md. Khaynjo checked on Xvfb end to end (Kitchen → station → Find
printers → sheet → Find → Done → the station list shows the printers).


## Owner queue, afternoon (2026-11-14) — the same in all four

| Change | Where it is written down |
|---|---|
| Live LAN socket: both loops poll (100 ms) instead of parking in a 25–35 s read — a tablet's KOT took up to 24 s, now ≈70 ms; silent socket closes after 45 s; frames read with the `101` head kept; slow phone writes logged (`lan_write_slow`) | DEV.md § LAN "Both loops poll" |
| The usblp lookup for a `usb:` printer reads `usbmisc` and resolves the relative sysfs link. (Routing a CUPS USB queue straight to its usblp node was removed 2026-11-15 — one machine's setup made the rule.) | PRINTER.md "What went wrong" 9–10 |
| Shop settings mirrored on every Client (`SHOP_PREF_KEYS`, snapshot + deltas); sidebar layout is a `BooksOp`; Khaynjo: GST wording and Tax invoice off by default | DEV.md § LAN "Shop settings on every device", DESIGN.md "One screen for every device" |
| Cluster texts removed (state restated, copies of tips, instruction bars, empty-state instructions, bracketed explanations); layout guard `layout_labels_carry_no_explanations`; `scripts/ui-text-inventory.sh` | DESIGN.md "Explanations live in ℹ" |
| Bill / KOT / receipt-label formats save themselves (no Save buttons) | DESIGN.md "Formats save themselves" |
| Settings holds the sidebar: one list, icons on every row, rows tap through; Sidebar screen removed; slot icons drawn by code only (`slot_icons_have_no_layout_picture`) | DESIGN.md "Settings holds the sidebar", DEV.md § WorldFonts |
| Dish search: names first, then aliases / groups; an exact short code leads (Khaynjo, Kirano, Tabiyat) | KHAYNJO.md menu window row |
| Opening the books logs its stages (`Kumkaj open:`, `role_open`) | DEV.md rule 7 |
| Khaynjo: the Rajdhani pack names the shop and brings its logo (`shop_name`, `logo_base64`) | CATALOGUE_URLS.md |
| Khaynjo: Export template — a configured shop as a format-2 pack, applied whole on a new device; packs carry no PINs (first PIN from the Owner in Team). **Not yet in Moklanjo / Kirano / Tabiyat** (owner: Khaynjo first, then the other three) | apps/khaynjo/docs/CATALOGUE_URLS.md "Export template" |

Visual baselines refreshed for the Settings redesign and the corrected
sidebar icons (2026-11-14), each old/new pair checked by eye first. The
visual gate in `product-cert.sh` had never failed: ImageMagick 7 prints
`90206.7 (0.0783045)` and the gate only matched bare integers, so a
Settings screen 90 000 pixels off passed. It now reads the leading number,
fails above `CERT_VISUAL_MAX_AE` (default 50000) and fails when compare
gives no number.
