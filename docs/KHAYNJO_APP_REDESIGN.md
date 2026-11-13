# Khaynjo app — UX audit and redesign plan (2026-11-13)

Scope: the Khaynjo **app** first, khaynjo.com second. Kumkaj brand only.
Doctrine: vision §2 / §24, [`DESIGN.md`](DESIGN.md) (work-surface
lessons, control face by intent), [`KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md) §5b. The catalogue
templates are **out of scope** and untouched (`sites/khaynjo/static/v1/catalogue`
schema / URLs, `publish-catalogue.sh`, `catalogue_remote.rs` download / import).

## 1. How the audit was done

- Neutral demo shop only: `tests/visual_seed.rs` now seeds **Demo Kitchen**
  (Mains / South Indian / Starters / Drinks, D1–D8 and P1–P4 in every visit
  state). Never a client's menu or staff.
- A development-env release build (`KUMKAJ_ENV=development cargo build
  --release -p kumkaj-khaynjo`) on a private Xvfb with `--remote`, Owner
  unlocked by the smoke hooks, every chrome peer opened and captured at
  **390×844** (phone), **820×1180** (tablet portrait) and **1280×800**
  (laptop), top of page and one PageDown.
- Screenshots (before / after / selection) are kept outside the repo in
  `local/khaynjo-redesign-2026-11-13/` (gitignored).

## 2. Findings (before)

Severity: **A** blocks or hides the job; **B** slows it or misleads; **C** polish.

| # | Screen | Size | Sev | Finding |
|---|---|---|---|---|
| 1 | Order | all three | A | The **Counter** pane (Total, tenders, Take payment) sits off the right edge: pane mins 600 \| 520 never fit beside the rail at 1280 px, and on a phone / tablet only a 28 px peek says it exists. The cashier's main action is found by swiping sideways. |
| 2 | Floor | all three | A | Columns default **4** when unset: tiles overflow every phone, the 820 px tablet and the 1280 px laptop (1.3–2.5 tiles visible, the rest cropped). Fit screen existed but was not the default. |
| 3 | Floor | all three | B | The Columns pick (a full-width filled "4" bar) plus a full-width − + Auto row take ~90 px of chrome above the tables, louder than the tables themselves. |
| 4 | Floor | resize | B | After a window resize / rail toggle the grid drew once with the old viewport width and nothing redrew it: a one-column Floor stayed wider than the screen, tile icons cut off. Same stale cache for page-window lists (`page_viewport`). |
| 5 | Settings | all three | C | The 15 hub rows use the filled `StackSecondaryBtn` (centered beige pills); a hub-face restyle was tried and the owner preferred the pills — kept. |
| 6 | Day book | all three | B | Every figure and label was a plain `Label`: no text on Day book could be selected or copied; Ctrl+A did nothing and right-click had no Copy. (Fixed with the selection model — DESIGN § Text selection & copy.) |
| 7 | Order | all three | C | Pay-method chips (Cash · UPI · Card · Credit · Split) are a scroll strip and Split is half hidden at common widths. Kept: the strip is a guarded earlier decision (see plan 1). |
| 8 | Phone chrome | 390 | B | 13 peers in the bottom strip: ~2.5 chips visible, the rest behind ‹ ›. Floor, Kitchen and Settings are 2–6 taps away for a waiter; the order is the owner's (Settings → Sidebar) but the default puts books peers before floor peers. |
| 9 | Header | all three | C | The server id "(4f71d56e)" sits under the shop name on every screen — a support detail on the work surface. |
| 10 | Tablet | 820 | B | The side rail takes ~240 px of an 820 px portrait tablet (30 %); every work surface is squeezed (Order panes pan, Floor gets one column). |
| 11 | Kitchen | all three | C | KOT cards are fixed-pitch (needed for the paged list, DEV bug 6c) and mostly empty for one- or two-dish tickets; the Start / Done button sits far from the dishes. |
| 12 | Parties | all three | C | The Add party form is the first thing on the screen; the people list (what the owner opens Parties for) starts below the fold. |
| 13 | Buy / Day book | all three | C | "Running low — buy soon" repeats as plain red text on two screens; on Buy it is the reason to act and could link to the items. |

## 3. Done in this pass

| Finding | Change | Where |
|---|---|---|
| 1 | **Reverted.** Narrower desk mins (340 \| 380) and a Payment › / ‹ Order jump pair were tried; the owner found both panes cramped — back to **600 \| 520** with the sideways pan. | `app/impl_04.rs` `apply_shell_layout`, `helpers.rs` `order_desk_pane_widths` (HEAD) |
| 2 | Columns unset → **Fit screen (0)**; a number the owner chose is kept (device UI preference, no schema change). | `store.rs` `floor_columns`, `app/impl_06.rs`, `app_state_impl.rs`, `store_ops.rs` test |
| 3 | One quiet row: − + Auto (fixed widths) then the Columns pick as an outlined `StackLightBtn` (Fill, last — no flex-shrink). | `ui_script.rs` `floor_cols_row` |
| 4 | `page_viewport` / `floor_hviewport` that change redraw the list that reads them. | `app/main_impl.rs` |
| 5 | Settings hub: a `HubRowBtn` restyle was tried and **reverted** on the owner's review — hub rows stay `StackSecondaryBtn`. | `ui_script.rs` |
| 6 | One text-selection model for the app and khaynjo.com, browser-like (drag across blocks, wheel / edge scroll while dragging, Ctrl/Cmd+A, translated Copy / Select all menu on text only, long-press on touch) — DESIGN § Text selection & copy, DEV § Text selection. | `src/selection_menu.rs`, `ui_script.rs`, `patches/makepad/text_selection.rs.patch` |
| — | Chip strips (`ChipStrip`, `FormScrollX`): mouse press on a chip + drag pans the strip (`mouse_capture_overload`), as on touch. | `ui_script.rs` |
| — | **Templates** mirror onboarding: Menu → Item templates = PUBLIC (Khaynjo Items Template chip → Import selected) + PRIVATE (12-digit code → "Rajdhani Template" → Import), items only; onboarding names the pack "Rajdhani **App** Template". Café / Khaynjo starter chips removed. | `ui_script.rs` `menu_tmpl_section`, `app/impl_02.rs` `paint_menu_public_card` / `paint_menu_private_card`, `app/main_impl.rs`, `store.rs` `apply_app_template_pack` |
| — | **Inline explanations → ℹ** (DESIGN "Explanations live in ℹ"): Onboard (status paragraph, "Logo: Kumkaj mark…", the empty "What you get" header, "Fresh selected…"), Shop templates blurb, Menu photo hint, Settings Alerts / Trading history / Delete local data paragraphs, Team pick hint, Brand icon line, Extra tills off line; Stamp / Payment QR lines are short states. | `ui_script.rs`, `app/impl_02.rs`, `app/impl_03.rs`, `app/impl_04.rs`, `app/impl_06.rs` |
| — | Settings → Areas · stations · tax rates showed raw ids ("tax:98aa6ad5-… · GST 5%") and its rows (and Units, printer pool) never took a tap: rows now show names; `ReportText{tappable: true}` gives the row a cursor and non-selectable text. | `helpers_books.rs` `sibling_picker_keys`, `widgets/report_sidebar.rs` |
| — | Header said "Locked" after a PIN unlock until Menu opened: Makepad never clears its IME area, so `text_field_has_focus` saw a stale field and sync_chrome stayed in its typing early-return (also why a card painted after Check code did not show). Focus counts only while the focused area owns the IME; a screen change drops a hidden field's focus. | `app/impl_05.rs` `text_field_has_focus`, `app/impl_04.rs` `show_screen`, `app/impl_02.rs` `sync_chrome` |
| — | **Regression of this pass, fixed:** a mouse drag that started on text no longer panned the Order desk / pages (the text took it for selection and blocked scrolling). Apps now pan on a mouse drag and select on a 0.5 s hold; A/B'd against the pre-session build. | `text_flow.rs` `MOUSE_HOLD_TO_SELECT`, `is_drag_selecting` |
| — | Kitchen → Stations: the section collapsed (page jumped to the top) on the first tap into its form — the tab force-showed the body while the accordion state said closed. One state now. | `app/impl_04.rs` `sync_kitchen_panels`, `app/match_event.rs` |
| — | Counter tickets: **Print selected again** said "KOT has no dishes to print: <id>" (also on the pre-session build) — Counter dishes live only on the stored ticket. Picked dishes now reprint from it by name. | `net.rs` `reprint_selected_kots`, test `counter_kot_reprints_the_picked_dishes` |
| — | Order ticket: the status line under + Add dishes repeated the snackbar — hidden. Snackbar: only errors red; warn amber, info slate, success green ("Station … added — attach its printers" is green). | `ui_script.rs` `receipt_label`, `app_snackbar`; `snack.rs` |
| — | Kitchen stations and dining tables refuse a duplicate name on add and rename (case-sensitive, spaces trimmed). | `store.rs` `name_taken` |
| — | UI freezes (pre-existing, measured on the dev build with a copy of the owner's data): Settings → Bill & KOT 1.4 s → under 250 ms (slip preview drawn once as pixels, no PNG round trip, skipped when unchanged); first actions after start 1.4 s → gone (Floor tile scan only on the Floor screen, only drawn rows). Previews pixel-identical to before. | `widgets/report_sidebar.rs` `show_slip`, `print_share.rs` `slip_page_gray`, `app/match_event.rs` |
| — | Dish photos decode off the UI thread; dev builds optimise dependencies (first + Add dishes 2.2 s → 0.19 s in `khaynjo-dev`); a screen switch that took ≥ 1 s last time shows a loading screen first. | `helpers.rs` `load_thumb_at`, `Cargo.toml`, `app/impl_04.rs` `go_with` |
| — | KOT kinds: **Copy from** any other kind (line sizes, weight, alignment; the kind keeps its banner, item prefix and strike). | `print_share.rs` `copy_line_styles`, KOT tab |
| — | Pointer cursor follows what is under it (hover-out order, tappable rows, re-hover after click/scroll on Linux). Touch text selection like Chrome on Android (hold timer, knobs, menu above the selection). | `text_selection.rs.patch` |
| — | Settings → Printers = Find + the saved printers, nothing else (no per-printer send method, no test-page picker, no "(none yet — tap Find)" boxes, no idle hint line). | `ui_script.rs`, `app/impl_03.rs`, `helpers_books.rs` |
| — | Deadlock: `catalogue_download_progress_text()` locked `app_state` (via `app_t`) inside callers holding the guard — any Menu template download froze the UI. It now takes the translated label. | `runtime.rs` |

Checked on screen (Xvfb `--remote`, 1280×800 plus the earlier 390×844 /
820×1180 passes): Order, Floor, Settings pages, Onboard, DeviceSetup
templates, Menu templates end to end (private: 20 → 141 products, staff
and settings unchanged; public: 20 → 1469).

## 4. Plan (next passes, in this order)

1. ~~Order — pay methods as a segmented grid~~ (finding 7) — **not doing**:
   the scroll strip is an earlier owner decision, guarded by
   `info_help_layout_tests` ("Take payment chips must scroll on phone
   widths"; cramped Fill SegBtns were rejected). Past decisions win over
   this audit; ask the owner before reopening.
2. **Kitchen** (finding 11): content-sized KOT cards inside the paged list
   (measure per pitch class: 1–2 dishes / 3–5 / more) so a short ticket is a
   short card; Start / Done beside the dishes; "To cook now" stays first.
   Verify on the AGS2 with a 40-ticket soak — the paged list is load-bearing
   for Android scroll (DEV bug 6c).
3. **Phone chrome** (finding 8): role-aware default order — a Waiter sees
   Floor · Order · Kitchen first; Owner keeps Day book first; Settings →
   Sidebar still owns the order. Consider a 5-slot bottom bar + "More" sheet
   only if the owner agrees (DESIGN says no More hub for Moklanjo; Khaynjo may
   differ, vision §24).
4. **Tablet** (finding 10): rail collapses to an icon rail (≈ 72 px) between
   720 and 1000 px; labels on hover / long-press; full rail from 1000 px.
5. **Header** (finding 9): server id only on Settings → Other devices and
   Diagnostics; header keeps shop name + who is signed in.
6. **Parties** (finding 12): people list first, Add party as a collapsed card
   ("+ Add party") above it.
7. **Buy / Day book** (finding 13): "Running low" as a card with the items
   and one action (Record purchase for these).
8. Then Menu, Sales, Day close, Reports, Inventory, Team — same audit grid;
   no catalogue template changes.

Each item is checked against earlier decisions first (DESIGN, KHAYNJO.md,
the `shell_tests` guards) and shown to the owner before it ships — two items
of this pass (desk jump buttons, hub row restyle) were reverted on review.

Every new string goes into `en()` and all 110 packs
(`./scripts/check-ui-translations.sh --app khaynjo`); faces follow
`makepad-layout-faces`, busy CTAs `busy-action-chrome`, state guards
`app-state-lock`, taps `press-touch`.

## 5. Owner decisions this pass changed (say so if you disagree)

- **Order desk:** unchanged in the end — mins 600 \| 520 and the pan (the
  smaller-mins trial was reverted on review).
- **Floor Columns default:** was 4 when unset, now Fit screen. Shops that
  already chose a number keep it.
