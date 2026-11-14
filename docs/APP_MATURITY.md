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
| 9 | Full audit (all four) and fixes | ◐ | ◐ | ◐ | Done: every Khaynjo fix since the 2026-11-08 fork checked against the three (list below), runtime DSL errors, hard-coded UI strings (none), `app_t` under a guard (new lock-guard rule, clean ×4), list walks off-screen, todo!/unimplemented! (none). Open for the next pass: Kirano / Tabiyat still carry Khaynjo's floor / kitchen / order-type code paths behind counter-only switches — prune or keep deliberately; Moklanjo popup placement has no tests (`place_popup` is Khaynjo's) |

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
- **All four:** `check-lock-guards.sh` now also flags `app_t(` under a live
  `app_state()` guard (the Menu template download deadlock class) — clean.
- **Android (all apps, shared Makepad):** touch selection on the AGS2 — held
  word cut by finger jitter, OS bar without Copy, Cut / Paste on read-only
  copy, Select all closing the bar. DEV.md § Text selection.
