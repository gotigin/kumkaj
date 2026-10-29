# Kumkaj developer notes

Commands and paths below are from the **workspace root** (parent of `docs/`).

**Product focus:** [Moklanjo](../apps/moklanjo/docs/MOKLANJO.md) — courier BOS (books +
booking desk). Package id: production `com.moklanjo` · development
`com.moklanjo.debug`.

## Common workspace — no Kumkaj core

The only shared thing between Kumkaj apps is this **common workspace**:
every app (`apps/moklanjo`, `apps/khaynjo`, …) is built on the same
**Makepad 2** (`vendor/makepad`) with similar libraries, because BOS
apps have similar requirements (money in/out, parties, day/period
books, receipts, reports). Similar requirements justify one workspace
and one Makepad platform — **not** shared product code.

- **There is no Kumkaj core** — no shared crate, shared books engine, or
  product/business code that more than one app depends on. Do not create one.
- Each app owns all of its code (`src/domain.rs`, `store.rs`, `fiscal.rs`,
  `locale*.rs`, `net.rs`, `main.rs`, …), even when copied from a sibling.
  Reuse is **copy-then-diverge**; duplication is preferable to coupling.
- A change for one app must not impact another app. If serving an app
  would require changing shared core code in a way that affects others,
  duplicate the code into that app instead. A future app never needs a
  change to shared core code — it copies what it needs and owns it.
- Workspace-level sharing is only: Cargo workspace membership, vendored
  Makepad 2 + patches, similar dependency versions / `Cargo.lock`, and
  root build/packaging tooling parameterised per app (`scripts/`,
  `tools/*-dev`). Changing Makepad/dependencies is a deliberate platform
  change affecting all apps — do not use it to sneak product logic into
  multiple apps, and fork a root tool/script per app rather than let one
  app's tooling change break another.
- **Scripts** (2026-10-26): root `scripts/` holds only tooling that names
  no app — Makepad setup and patches, the Android activity bridge, env
  loading, Windows cross-build and resource staging, the MSI builder,
  icon/font generators, the Apple scaffold. Each one requires the caller
  to say which app (`KUMKAJ_APP_DIR` and friends) and fails without it;
  none defaults to Moklanjo any more. Anything that names one app — its
  package scripts, installers and uninstallers, `Setup.bat`,
  `desktop-ids.sh`, `License.rtf`, MSI upgrade codes (`msi.sh`), Play /
  web packaging — lives in `apps/<app>/scripts/`.

Canonical doctrine: [`KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md) §1–2; concepts checklist: [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md).

## Live development (Cursor)

**Khaynjo** (restaurant OS, [`apps/khaynjo`](../apps/khaynjo/)) mirrors
Moklanjo's tooling one-for-one: `cargo khaynjo-dev`,
`cargo khaynjo-android-dev`, `cargo khaynjo-windows-dev`. Identity
differs by domain: Moklanjo uses moklanjo.com
(`com.moklanjo` · `com.moklanjo.debug`); Khaynjo uses khaynjo.com
(`com.khaynjo` · `com.khaynjo.debug`). Each app's packaging lives in its
own `apps/<app>/scripts/` (see § Common workspace → Scripts).

```bash
cargo moklanjo-dev
```

Loads [`apps/moklanjo/env/development.env`](../apps/moklanjo/env/development.env) then `cargo run -p kumkaj-moklanjo -- --hot --remote`.

- `--hot` — Splash/`script_mod!` hot reload on save
- `--remote` — Tweaker via **F12**
- Makepad **2 / `main` only** — never Makepad 1 docs/examples/APIs
- Local Makepad 2 tree: `vendor/makepad`
  - First time / after clean: `./scripts/setup-makepad.sh`
  - Path-dep from each app (`apps/moklanjo`, `apps/khaynjo`). The script rebuilds the tree from the pinned upstream commit (`MAKEPAD_REV`) and applies every patch in `patches/makepad/` in the order its `PATCHES` list gives; any patch that fails stops the script
  - `vendor/makepad` is gitignored: **an edit there is lost on the next setup unless it is also a patch in `patches/makepad/` listed in `PATCHES`**. After changing a vendor file, regenerate its patch, then rerun the setup in a scratch copy and diff it against `vendor/makepad`
  - Nested Settings lists claim wheel events so the page does not scroll with the list
  - Page `ScrollYView` keyboard: ↑/↓ step, PgUp → top, PgDn → bottom (`App::handle_scroll_key`). Skipped while a TextInput has key focus.
- Multi-script UI fonts live **per app**: `apps/moklanjo/resources/fonts/` and `apps/khaynjo/resources/fonts/` (Noto scripts, FA icons, LXGW CJK, Noto Color Emoji — each app carries its own copy, like its code). Desktop packages copy only that app's `<crate_name>/resources/` (underscores — Makepad crate name) — not a second `makepad_widgets/resources/` font tree. Re-fetch with `./scripts/fetch-ui-fonts.sh apps/<app>` (no argument: every app) if fonts are missing.
- **Desktop window size & maximize** — see § below.
- **Windows console** — production builds (`KUMKAJ_ENV=production` / `package-windows.sh`) use the GUI subsystem (no terminal). Dev builds keep a console for logs.
- **Min layout width 320 css px** — `app_shell` uses `Fill{min: 320.}` inside a horizontal scroll. The UI never lays out narrower than 320; if the device/window is below 320, content scrolls horizontally
- **Dev size** — `W×H` in the window caption bar, left of the Kumkaj title (terracotta on navy). Controlled by `KUMKAJ_SHOW_SIZE_BADGE` in the env file (not Cargo `--release`).
- **Button label inset** — every face within an app sets layout `padding` **and** `label_walk.margin` (Makepad 2 Fill + Ellipsis otherwise pins ink to the face edge)
- **Reusable layout widgets** — see § Makepad 2 UI faces (agent guide) below (selection + truncation). Product summary: [`DESIGN.md`](DESIGN.md) § Makepad 2 faces.

## Desktop window size & maximize

Expected behaviour on **Linux (Wayland/GNOME), Windows, and macOS** for Moklanjo client-side chrome:

| Action | Expected result |
|---|---|
| First launch | Floating window **1024×768** (DSL + store defaults) |
| Later launch | Last **floating** size/position from `window_geom.json` under the app data dir |
| Max button (single click) | Toggle maximize ↔ restore |
| Caption bar double-click | **Same** toggle as the max button (not a second code path) |
| Restore with no saved float size | **1024×768** (never a monitor-fill size) |
| Resize floor | **320 css px** width and height |

`window_geom.json` must **never** store maximized/fullscreen geometry. Persisting ~monitor size (e.g. `2560×1048` on an ultrawide) made the next launch look maximized while floating; the next `set_maximized` then fought Mutter on multi-monitor layouts.

### Wayland / Mutter (do not regress)

Vendor patches live under `vendor/makepad` (path-dep). Toggle logic is owned by `Window::toggle_chrome_maximize` in `vendor/makepad/widgets/src/window.rs`. Platform ops: `MaximizeWindow` / `RestoreWindow` in `linux_wayland.rs`.

RTL text alignment rides on two more vendor patches: `TextFlow::paragraph_align_x` in `vendor/makepad/widgets/src/text_flow.rs` (resting paragraph alignment; open table cells still override it) and `pub TextInput::label_align` in `vendor/makepad/widgets/src/text_input.rs` (patch files `patches/makepad/text_flow_paragraph_align.rs.patch` and `patches/makepad/text_input_label_align_pub.rs.patch`, applied by `scripts/setup-makepad.sh`). Each app sets them from `apply_text_alignment` while `locale::is_rtl` is active; glyph shaping and bidi ordering are Makepad’s own (rustybuzz + unicode-bidi in `draw/src/text/shaper.rs`). The window layout is not mirrored.

**Mouse cursor state** (`patches/makepad/mouse_cursor_state.rs.patch`): Wayland mapped `MouseCursor::Arrow` to the context-menu shape (focused buttons looked like a menu); it now matches Default. Widget `SetCursor` is ignored while the pointer is in the window-edge resize band so Text/Hand cannot wipe Nw/E/S/…. `View` / `Button` / `TextInput` re-assert their cursor on `FingerHoverOver` (HoverIn only fires once); focused buttons keep Hand; caption drag uses Grab. Both apps also set `ColResize` while hovering the rail divider.

**Bugs we hit (do not reintroduce)**

1. **`display_context.screen_size` is the window size** — never use it as “fills the screen ⇒ maximized”; that made every max click take the restore path.
2. **Stale maximized configure after restore** — re-latched `chrome_maximized` so later clicks only restored. Suppress compositor “adopt maximized” briefly after a chrome restore/maximize.
3. **Second `set_maximized` while Mutter still has maximized** — on a 2×2 ultrawide layout this jumped toward another output and then left maximize dead. Track `compositor_maximized` from configure `states` only; skip `set_maximized` when it is already true.
4. **Mutter post-restore float configure** — after restore agrees at `1024×768`, Mutter often sends another **floating** configure at the old max extent (`2560×1048`, `maximized=false`). Without a float guard the window balloons again (works / not / works). Keep `float_guard_size` ~2s and ignore monitor-fill floating configures.
5. **Caption double-click must not call `xdg_toplevel.move` on the first press** — that ate double-click and used a divergent compositor-only toggle. Defer move until pointer travels &gt;5px; on double-click emit `Event::WindowChromeMaximizeToggle` so the **same** widget toggle runs as the max button.
6. **Do not remember near-monitor sizes as restore targets** — treat `width ≥ 1920` and `height ≥ 900` as monitor-fill; fall back to `1024×768`.

**Code map**

| Piece | Role |
|---|---|
| `Window::toggle_chrome_maximize` | Single-click max + caption double-click |
| `Event::WindowChromeMaximizeToggle` | Wayland caption → widget toggle |
| `WaylandWindow::compositor_maximized` | Last configure maximized/fullscreen bit |
| `pending_restore_size` / `float_guard_*` | Force float size; reject Mutter re-expand |
| `apps/moklanjo` `persist_window_geom` | Skip fullscreen / monitor-fill sizes |
| Dev logs | `MaximizeWindow` / `RestoreWindow` / `xdg_toplevel configure` / `caption double-click` |

```text
❌ Persist maximized size into window_geom.json
❌ set_maximized again while compositor_maximized
❌ Accept floating 2560×1048 right after restore to 1024×768
❌ Caption double-click toggles in wayland_state instead of the widget
❌ Start xdg_toplevel.move on the first caption press of a double-click

✅ One toggle path (widget) for max button and caption
✅ Restore → 1024×768 (or last real float size) + float guard
✅ Skip set_maximized when already maximized per configure states
```

Manual check (GNOME Wayland, multi-monitor OK): max → restore → max → restore several times; caption double-click the same; window stays on the current output; terminal may show `ignoring post-restore monitor-fill`.

## Makepad 2 UI faces (agent guide)

Use this when changing bottom nav, choice toggles, chips, or any button label. The same app's UI code ships to **Linux / Windows / Android** — fix faces once *in that app* (examples below are Moklanjo's `apps/moklanjo`), not per platform and not in another app.

Cursor rule (always apply when editing kumkaj UI): `.cursor/rules/makepad-layout-faces.mdc`.

**State-guard rule (all apps, each app's own code):** never call
`sync_chrome` / `set_status` / `go` (or anything that locks app state)
while holding a live `app_state()` guard — the mutex is non-reentrant
and the UI thread deadlocks on itself. Drop guards in an inner scope
first. Each app enforces this in its own tree (e.g. Khaynjo's
`apps/khaynjo/scripts/check-lock-guards.sh`); a future app copies the
check, it does not share it.

**Agent verification rule (all apps):** no GUI automation on the
owner's machine for verification — code, tests and store-level checks
only, unless the owner asks.

### What Makepad 2 actually does

`makepad_widgets::Button` has **no `selected` / pressed-sticky API**. Face and label colors are mixed by animators only:

| Animator | Meaning |
|---|---|
| `hover` | Pointer over |
| `down` | Finger/mouse held |
| `focus` | Key focus (sticks after click if `grab_key_focus` is true) |
| `disabled` | Disabled |

So “this tab is active” **cannot** be expressed by leaving a Button focused or hovered. If the idle face uses the same navy fill on `color_hover` / `color_focus` as the selected face, **every previously touched tab keeps looking selected**.

### Bugs we hit (do not reintroduce)

1. **Multiple bottom-nav peers lit** — Idle `TabBtn` used primary navy on hover/focus (same as `TabBtnActive`). After Sell → Buy → More, Sell (and others) still looked active. Focus also stuck on the last clicked tab.
2. **Wrong peer lit while another screen showed** — Treating focus/hover as selection instead of painting from `Screen` / `nav_tab_for`.
3. **Right-truncated CTA labels** — Full-width faces (`NavBtn`, `PayBtn`, `MoreRow`, `Row*`) used `max_lines: 1` + `TextOverflow.Ellipsis`. Makepad centers a Fill label in a Fill button; long / translated strings become `record_pu…` / clipped ink. Multi-word choices in `ChipBtn` (Fit max 160) truncated the same way (`Use platform na…`, `Tax invoice`, `On account (AP)`).
4. **Raw locale keys on buttons** — `locale::t` falling through to the key (`record_purchase`). English `en()` must define every chrome key; see `locale::tests::english_chrome_never_returns_raw_keys`.
5. **Item / rate photos** — Moklanjo has no built-in food-menu art pack. Thumbnails are name initials or a user-picked custom photo under the store’s item-images dir (not a packaged `resources/menu/` catalogue).
6. **Whole app frozen by list photos** — A `PortalList` given its full content height (the page-scroll rule) draws *every* row, and each row decoded its full-size photo synchronously in `draw_walk`. A ~100-dish list took tens of seconds per frame in a dev build (≈250 ms per 470×1024 WebP unoptimised), so every screen showing item photos “hung” (Khaynjo, 2026-10-23). Rule for every app: never call `load_image_file_by_path` on an original photo from a list’s draw — show a small thumb made off the UI thread (initials until it lands; Khaynjo’s `src/thumbs.rs`), and give long lists their own capped, self-scrolling viewport so only visible rows draw. Image codec crates are `opt-level = 3` in the workspace dev profile so photo work stays fast in `*-dev` builds.
7. **Settings / CTAs freeze the UI (wrap + Fill)** — Makepad turtle logs `flow: Right { wrap: true } does not support width: Fill` every frame when a **wrapping** flow walks a **Fill** child; log spam freezes the app (Khaynjo Settings → Add printer / route buttons, 2026-10). Two cases:
   - **Button face:** `align.x ≥ 0.25` on a Fill-width button makes `Button::draw_face` force a Fill label walk. With `flow: Right{wrap: true}` that is the hang. **Do not** “fix” by removing wrap or left-aligning labels — that brings back truncated/cropped CTAs (bug 3). The shipping fix is patch `button_label_center` (`patches/makepad/button_label_center.rs.patch`): when the face wraps, keep a **Fit** label walk; text still centers via `text_align` + `ink_centered`.
   - **Parent View:** never put `width: Fill` children (`NavBtn`, `Stack*`, `SegBtn`, …) inside `flow: Right{wrap: true}`. Use `ActionStack` / `flow: Down`, or Fit-width chips in a wrapping strip.

### Required approach (shipping)

**Selection (bottom nav + dual choices)**

1. DSL: Overlay of **idle** + **active** faces (`TabBtn` / `TabBtnActive`, or `StackSecondaryBtn` / `StackPrimaryBtn`, or `ChipBtn` / `ChipBtnActive` for short chips only).
2. Idle face: **must not** reuse the active fill on hover/down/focus (nav idle = transparent fill; muted text only).
3. Active face: solid selected look; nav tabs use `grab_key_focus: false` and `reset_hover_on_click: true`.
4. Rust: paint from app state only:
   - Bottom nav → `paint_nav_tabs` (exactly one peer from `nav_tab_for(screen)`; More children keep **More**).
   - Dual choices → `paint_choice_pair` / `paint_choice_state`.
5. On every paint: `clear_button_animators` (`reset_hover` + `animator_cut` `hover.off` / `focus.off`) on **both** faces before setting visibility, so a hidden/deselected face cannot stay lit.
6. Clicks: `choice_chip_clicked` (idle **or** active); clear key focus when changing bottom-nav peer.

**Intent → face (same job → same look)**

| Intent | What the click does | Face |
|---|---|---|
| **Navigate** | Leaves this screen / opens another peer | `HubDestBtn` + mark (hub lists) or chrome `TabBtn` / `NavChip` |
| **Select** | Stays here; picks among options | `DeskTab` / `SegBtn`/`SegBtnOn` / `ChipBtn`/`ChipBtnActive` |
| **Submit / do** | Commits work on this form | **One** filled `RowPrimaryBtn` / `StackPrimaryBtn` (or `PayBtn` on ticket) |
| **Quiet alt** | Cancel / clear / secondary do | `RowSecondaryBtn` / `StackSecondaryBtn` / outlined `NavBtn` |
| **Destroy** | Void / refund / delete | `DangerBtn` |

Hard rules: never use a filled primary to mean “this navigate link is more important”; never use `ChipBtn` as a one-shot CTA (NDR / Show PIN / line Cancel → `QuietFitBtn`); exclusive closed sets (tax invoice, amend pay, paid/AP, staff role, …) → `SegBtn`, not Stack primary/secondary twins; short filters (units, catalog facets) → `ChipBtn`; `PayBtn` is ticket take-payment only. **Never** put `Row*` / `Stack*` (Fill + `wrap: true`) at `width: Fit` — wrap measures one glyph wide and stacks letters vertically (`S`/`h`/`o`/`w`…); use `QuietFitBtn` (Fit, no wrap) for inline quiet actions.

**Labels (no right truncation)**

| Kind of label | Face |
|---|---|
| Short chip in a horizontal strip | `ChipStrip` + `ChipBtn` / `ChipBtnActive` (Fit max 160, ellipsis OK) |
| Closed radio set (equal Fill share) | `SegBtn` / `SegBtnOn` in a `Right` row of Overlay cells |
| Ultra-short side-by-side (≤ ~4 chars) | `ActionRow` + `RowPrimaryBtn` / `RowSecondaryBtn` |
| Multi-word / translated CTA | `ActionStack` + `StackPrimaryBtn` / `StackSecondaryBtn`, or outlined `NavBtn` / `PayBtn` / `MoreRow` / `HubDestBtn` / `DangerBtn` |
| Side-by-side fields (no CTAs) | `FormFieldRow` + `FieldInput` (equal `Fill`) |

Full-width CTA faces: `width: Fill`, `height: Fit`, `flow: Right{wrap: true}`, `align: Align{x: 0.5 y: 0.5}` (hub dest left-aligned), label walk **`width: Fit`** (not Fill — see bug 7 / `button_label_center` patch), **no** `max_lines: 1`, **no** `Ellipsis`. Always `padding` **and** `label_walk.margin`. Never strip wrap or center to dodge a hang — fix the Fill-in-wrap walk instead.

Never invent a one-off button type (`LangSourceBtn`, etc.) when Stack/Nav/HubDest/Danger already cover it. Never put multi-word phrases in `ChipBtn` / `ActionRow`.

**FormFieldRow (no flex-shrink):** Makepad will crush a Fit CTA beside Fill inputs until wrap stacks each letter. Example regression: **Add rate** as a vertical navy pill overlapping Unit. Always:

```text
FormFieldRow{ price FieldInput; unit FieldInput }
ActionStack{ New item StackSecondaryBtn; Add rate StackPrimaryBtn }
```

### Nav drag-to-swap (do not regress)

Implemented in `pointer_nav_drag` / `nav_drop_slot` / `nav_drag_autoscroll` (`main.rs`). Behaviour contract: tap = navigate; press on a tab + move = drag (ghost chip follows, drop target wears the active face, release swaps and persists); press on whitespace = scroll; hold a dragged tab at a bar edge = auto-scroll. This works flawlessly today — do not “simplify” the pieces below; each one is load-bearing.

1. **Slots are positions.** `paint_nav_tabs` paints slot N with `nav_order[N]`; the order lives in books meta `ui_nav_order` (`Store::nav_order_csv` / `set_nav_order_csv`). A swap exchanges two slots and re-saves. Never hard-code tab→slot.
2. **Spatial gesture split only.** The bars keep whitespace lanes around the tabs (padding inside the scroll view: bottom bar 12px top/bottom/ends + 6px gaps; rail 12px sides). The pickup hit test (`nav_slot_at`, pad 0) must stay exact — any padding eats the lane and lane swipes start drags instead (observed: a 5px pad turned an above-chip swipe into a swap). Drop targeting may pad (2×5px). Do not reintroduce direction- or timing-based gesture decisions (“sideways = scroll”, hold-to-pick-up timers): the natural sideways drag then never picks up, which reads as “drag is disabled”, and timing misfires on laggy remote input.
3. **Once picked up, the drag owns the gesture.** `handle_event` calls `pointer_nav_drag` before `ui.handle_event`; past the 10px pickup threshold it consumes moves/ups so the bar's own scroll view cannot scroll the drop target away mid-drag. The release is consumed too, so no click follows; `nav_suppress_click` is cleared on every MouseDown.
4. **Frozen content rects for hit tests during a drag.** At pickup, capture every slot's rect + the bar's scroll offset into `NavDrag.rects` (content coordinates). The drop target comes from `nav_drop_slot`: the frozen rect containing `pointer + live scroll`. Never hit-test live `area().rect()` mid-drag — during edge auto-scroll the areas lag the scrolling content (observed: drop target frozen on the pre-scroll tab across 107 frames / 971px of scrolling, and the drop swapped the wrong pair). The ghost origin likewise comes from the pickup-time rect.
5. **The ghost lives in a Modal** (`nav_drag_modal`, transparent backdrop, overlay content). As a plain sibling it paints under the page scroll views — visible over the rail, clipped/invisible over the page and in bottom-bar mode.
6. **Edge auto-scroll** runs on a NextFrame loop armed at pickup (`nav_autoscroll_frame`, renewed in `nav_drag_autoscroll` while the drag is active; it dies with the drag). 56px edge zones; step ≈2–12px/frame proportional to depth into the zone; clamped to `[0, total − visible]`; drop target recomputed every frame. **End clamp:** held past the first/last tab (within a 28px cross-axis band) the target is that end tab — without it, holding at the far edge targets the trailing end padding and the drop silently does nothing.
7. **Faces:** the drop target wears the active face via `paint_nav_tabs` (its `drop` comes from `nav_drag()`), reusing the idle/active overlay — one more place where “paint from state, clear animators” above is load-bearing.

Manual regression check (no automated GUI test exists): narrow window (bottom bar) and wide (rail) — tap navigates; a lane swipe scrolls without reordering; a sideways drag from a tab swaps; holding at each edge auto-scrolls and drops onto a tab that was off-screen (confirm in sqlite: `SELECT value FROM meta WHERE key='ui_nav_order'`); pickup stays accurate while the bar is mid-scroll.

### Code map (`apps/moklanjo/src/main.rs`)

| Piece | Role |
|---|---|
| `TabBtn` / `TabBtnActive` | Bottom-nav idle / selected faces |
| `HubDestBtn` | Hub navigate jump + mark (never filled primary) |
| `SegBtn` / `SegBtnOn` | Closed radio sets (select in place) |
| `NavBtn`, `PayBtn`, `MoreRow`, `Stack*`, `Row*`, `DangerBtn` | Full-width CTAs by intent (centered, wrap, no ellipsis; Fit label — see bug 7); `PayBtn` = ticket pay only; `DangerBtn` = void/refund/delete |
| `QuietFitBtn` | Inline Fit quiet action (Show PIN, NDR reason, ticket Cancel/NC/Move) — never `Row*` at `width: Fit` |
| `FormFieldRow` | Side-by-side fields only — never CTAs |
| `ChipBtn` / `ChipBtnActive` | Short strip filter chips only — never one-shot actions |
| `nav_tab_for` | `Screen` → which bottom peer is active |
| `paint_nav_tabs` | Exactly one footer peer visible-as-selected |
| `pointer_nav_drag` / `nav_drop_slot` / `nav_drag_autoscroll` | Tab drag-to-swap, drop targeting, edge auto-scroll (see “Nav drag-to-swap”) |
| `paint_choice_pair` / `clear_button_animators` | Dual-choice visibility + animator clear |
| `paint_choice_state` | All dual choices after chrome sync |
| `locale::en` + `english_chrome_never_returns_raw_keys` | Human labels, never snake_case keys |
| `locale::t` + `shipped_languages_translate_chrome` + `rtl_languages_detected` | Static UI fully translated in 29 languages (no English fallback); RTL detection drives `apply_text_alignment` |

```text
❌ Idle TabBtn color_hover = selected navy
❌ Rely on focus/hover as “selected”
❌ NavBtn / MoreRow with Ellipsis; ChipBtn for “Tax invoice” / language source / NDR / Show PIN
❌ RowSecondaryBtn{width: Fit} / any Fill+wrap face forced Fit (vertical letter stack)
❌ Filled primary for a Day-book (or hub) navigate link (“Book a parcel” miracle CTA)
❌ FieldInput + FieldInput + NavBtn{width: Fit} in one Right row
❌ Patch one screen only — faces are shared across Linux / Windows / Android

✅ Overlay idle+active + paint from Screen + clear animators
✅ Intent first: HubDest for jumps · SegBtn for radios · ChipBtn for short filters · one primary per form · Danger for destroy
✅ Wrap on full-width CTAs; ChipBtn only for short chips
✅ FormFieldRow for fields; ActionStack for multi-word CTAs under them
✅ Fix that app's face definitions once (all platforms for that app)
```

## Development vs production env

Each concrete product uses **reverse-DNS under its own domain** (Moklanjo: `com.moklanjo`; Khaynjo: `com.khaynjo`), not under `com.kumkaj.*` — see vision §1 and each app's brief.

| | Development | Production |
|---|---|---|
| Env file | [`apps/moklanjo/env/development.env`](../apps/moklanjo/env/development.env) | [`apps/moklanjo/env/production.env`](../apps/moklanjo/env/production.env) |
| How | `cargo moklanjo-dev`, `cargo moklanjo-android-dev`, `cargo moklanjo-windows-dev` | `./scripts/package-*.sh` |
| `KUMKAJ_APP_ID` | `com.moklanjo.debug` | `com.moklanjo` |
| `KUMKAJ_APP_LABEL` | Moklanjo Dev | Moklanjo |
| Size badge (`KUMKAJ_SHOW_SIZE_BADGE`) | on (`1`) — `W×H` in caption | off (`0`) |
| Windows console | yes (logs) | no (GUI subsystem) |

`apps/moklanjo/build.rs` bakes the selected env file into the binary (`KUMKAJ_ENV` wins; otherwise debug Cargo profile → development, release → production). **Cargo profile ≠ app env:** `cargo moklanjo-windows-dev` builds the **debug** profile with `development.env` (Dev app id / console). Android live-dev still uses Cargo `--release` for install speed, but exports `KUMKAJ_ENV=development` so the **debug** package id and badge stay on.

### Platform naming (dev vs prod)

Same `KUMKAJ_APP_ID` / `KUMKAJ_APP_LABEL` on every OS. How each platform surfaces them:

| Platform | Development | Production |
|---|---|---|
| **Android** package name / application id | `com.moklanjo.debug` | `com.moklanjo` |
| **Android** launcher label | Moklanjo Dev | Moklanjo |
| **iOS / macOS** bundle id (when signed) | `com.moklanjo.debug` | `com.moklanjo` |
| **Linux** binary | `cargo moklanjo-dev` (host) · `package-linux-dev.sh` | `dist/linux/` + `install.sh` via `package-linux.sh` |
| **Windows** (Win10 + Win11 x86_64) | `cargo moklanjo-windows-dev` → `dist/moklanjo-windows-dev/` (exe + resources) | `./apps/moklanjo/scripts/package-windows.sh` → `dist/moklanjo-windows/` |
| **macOS** binary (when built on Mac) | `cargo moklanjo-dev` | package script / packager |
| **Desktop packager** (`Cargo.toml` `[package.metadata.packager]`) | Dev installs ignore this; scripts pass `--package-name` / env | `identifier = "com.moklanjo"`, `product_name = "Moklanjo"` |
| **Web** | local `cargo moklanjo-dev` / wasm as configured | site **https://kumkaj.com** (`web/`, `./apps/moklanjo/scripts/package-web.sh`) — not an OS package id |
| **Play Store** listing | side-load / internal only | application id `com.moklanjo` (`web/play-store-listing.txt`) |

### Data directories (keyed by `KUMKAJ_APP_ID`)

Dev and prod can sit side-by-side on one machine because paths differ by id. No extra vendor folder under the id.

| Platform | Development | Production |
|---|---|---|
| Linux | `~/.local/share/com.moklanjo.debug/` | `~/.local/share/com.moklanjo/` |
| Windows | `%APPDATA%\com.moklanjo.debug\` | `%APPDATA%\com.moklanjo\` |
| macOS | `~/Library/Application Support/com.moklanjo.debug/` | `~/Library/Application Support/com.moklanjo/` |
| Android | `/data/user/0/com.moklanjo.debug/files/` | `/data/user/0/com.moklanjo/files/` |
| Local servers | `servers.json` + `servers/{id}/moklanjo.sqlite3` (one DB per business/server) | same |
| Client cache | `client/moklanjo.sqlite3` | same |
| Backups | `…/<APP_ID>/backups/` | same |
| Override | `KUMKAJ_DATA_DIR` | same |

**DeviceSetup (every launch):** language (platform default, immediate apply) + Server/Client. Server Mode creates a unique local server if none exist, otherwise create-new or open-existing. See [`DESIGN.md`](DESIGN.md) § Launch / auth.

### Product / template default (onboarding)

**Moklanjo** should open with courier defaults (Account/Cash booking, kg
rates language). The workspace may still contain other template chips for
later brand products; do not make “None” the story for courier demos.
- Pay methods: Cash · instant rail (UPI / Pix / PayNow / … per country pack) · Card · **Credit** · **Split Cash+rail** (Server and paired Client)
- **UI theme:** Stitch Tactile Countertop ([`DESIGN.md`](DESIGN.md); full pack at `../../stitch_tactile_countertop_design_system/`)
  - Logo palette: surface `#D6E0EC`, navy `#02030B` / `#072058`, orange `#C44E10`, yellow `#F7B043` (see [`DESIGN.md`](DESIGN.md))
  - White glyphs only on navy chrome — never light ink on light wells / placeholders
  - PIN, Booking, and the three-total ledger share the same tokens

## Android (device / emulator)

```bash
cargo moklanjo-android-dev
```

Equivalent: `./apps/moklanjo/scripts/android-dev.sh`

Requires `ANDROID_HOME` (or `~/Android/Sdk`), a connected device/emulator, and Makepad’s Android toolchain (auto-installed on first run). Builds, installs, and launches **`com.moklanjo.debug`** (from `apps/moklanjo/env/development.env`).

**Launcher / logo:** Master art is `apps/moklanjo/assets/moklanjo-app-icon.png`. Run `./apps/moklanjo/scripts/generate-icons.sh` (Khaynjo: `./apps/khaynjo/scripts/generate-icons.sh`) to rebuild Android mipmaps, desktop `icon_*.png` / `icon.ico`, and in-app **`kumkaj-mark.webp`** (WebP). Reinstall Android after regenerating.

Android has **no** Splash `--hot` like desktop — each invocation rebuilds and relaunches. Day-to-day UI iteration stays on `cargo moklanjo-dev`; use `cargo moklanjo-android-dev` to verify on device.

Release APK/AAB packaging: `./apps/moklanjo/scripts/package-android.sh` → `dist/android/kumkaj-moklanjo.apk`.
Khaynjo APK (production id `com.khaynjo`, no device needed): `apps/khaynjo/scripts/package-android.sh` → `dist/khaynjo-android/kumkaj-khaynjo.apk`.

**Release signing (both apps, `scripts/android-env.sh`).** cargo-makepad's
`build` always signs with Makepad's bundled debug key. An Android update
installs only over an app signed with the *same* key, and uninstalling to
get past a key change deletes that device's books — so anything handed to a
shop must carry the app's own release key from the first install. Make one
per app, once, and keep it (and its password) backed up outside the repo:

```bash
cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release -- \
  android --sdk-path="$ANDROID_HOME" keystore-create ~/keys/khaynjo.keystore
KUMKAJ_ANDROID_KEYSTORE=~/keys/khaynjo.keystore KUMKAJ_ANDROID_KEYSTORE_PASS=… \
  apps/khaynjo/scripts/package-android.sh
```

With the keystore set, the package script re-signs the APK (`apksigner`,
password through the environment, never argv) and also builds the Play
bundle (`khaynjo.aab` / `kumkaj-moklanjo.aab`, `build-aab --keystore`).
Without it the APK is debug-signed and its `README.txt` says so. Package
builds pass `--version-code=auto` (YYYYMMDDHH), so every build installs as
an upgrade and Play accepts it; dev builds keep `version_code = 1`. The
scripts pick their own APK by the production label (`khaynjo.apk`,
`kumkajcourier.apk`) — Moklanjo's used to copy every APK and AAB under
`target/`, other apps' and dev builds included.

**Emulator on this host (x86_64):** Google's system images here are
x86_64 without ARM translation, so the arm64 APK will not install. Add
the target once (`rustup target add x86_64-linux-android`) and build with
cargo-makepad's `--abi=x86_64`. Create the AVD with `ANDROID_USER_HOME`,
`ANDROID_EMULATOR_HOME` and `ANDROID_AVD_HOME` pointed at a scratch folder:
`avdmanager` otherwise rewrites the developer's `~/.android/devices.xml`
(it renamed it to `devices.xml.old` once, 2026-10-26). A LAN test across
emulator and desktop uses `adb forward tcp:17847 tcp:7847` (the
emulator's own address is not reachable from the host).

## Desktop install / uninstall (Linux + Windows)

Packages ship **Install** and **Uninstall** so owners never hunt data paths.
Dev and prod stay separate (debug suffix on id + install folder).

| | Development | Production |
|---|---|---|
| Env | `apps/moklanjo/env/development.env` | `apps/moklanjo/env/production.env` |
| App id | `com.moklanjo.debug` | `com.moklanjo` |
| Label | Moklanjo Dev | Moklanjo |
| Linux package | `./apps/moklanjo/scripts/package-linux-dev.sh` → `dist/linux-dev/` | `./apps/moklanjo/scripts/package-linux.sh` → `dist/linux/` |
| Windows package | `cargo moklanjo-windows-dev` → `dist/moklanjo-windows-dev/` | `./apps/moklanjo/scripts/package-windows.sh` → `dist/moklanjo-windows/` |
| Linux install dir | `~/.local/lib/kumkaj-moklanjo-debug/` | `~/.local/lib/kumkaj-moklanjo/` |
| Linux command | `kumkaj-moklanjo-dev` | `kumkaj-moklanjo` |
| Windows install dir | `%LOCALAPPDATA%\Programs\Moklanjo Dev\` | `%LOCALAPPDATA%\Programs\Moklanjo\` |
| Data dir | `…/com.moklanjo.debug/` | `…/com.moklanjo/` |

**Install:** run `install.sh` (Linux) or double-click **`Setup.bat`** (Windows) from the package folder. Adds Start Menu / app menu entries, including **Uninstall …**. On Windows, do not open `install.ps1` in Explorer (that often opens Notepad) — use `Setup.bat`.

**Uninstall:** from the menu, Settings → Apps (Windows), double-click `Uninstall.bat`, or re-run `uninstall.sh` / `uninstall.ps1`. Always prompts:

- **Keep data** (default) — books stay; run uninstall again later to delete them  
- **Remove data** — deletes the app-id data directory  

If app files are already gone but data remains, re-running uninstall still offers to remove leftover data.

**Explicit env wrappers** (no flag to remember — use these from the repo):

| | Development | Production |
|---|---|---|
| Linux | `./apps/moklanjo/scripts/uninstall-linux-dev.sh` | `./apps/moklanjo/scripts/uninstall-linux-prod.sh` |
| Windows | `apps\courier\scripts\uninstall-windows-dev.ps1` | `apps\courier\scripts\uninstall-windows-prod.ps1` |
| Android | `./apps/moklanjo/scripts/uninstall-android-dev.sh` | `./apps/moklanjo/scripts/uninstall-android-prod.sh` |
| Android (Khaynjo) | `./apps/khaynjo/scripts/uninstall-android-dev.sh` | `./apps/khaynjo/scripts/uninstall-android-prod.sh` |

(Linux/Windows wrappers call the shared uninstaller with `--dev` / `-Dev` or `--prod` / `-Prod`. Android uses `adb uninstall` of that package id.)

Each app's installers are its own (`apps/<app>/scripts/`: `install-*.{sh,ps1}`, `uninstall-*`, `Setup.bat`, `Uninstall.bat`, `desktop-ids.sh`, `License.rtf`) — they name that app's exe, folders and labels. `scripts/stage-desktop-package.sh` copies the calling app's set into a package. (Khaynjo's Windows folder package used to stage Moklanjo's, which looked for `kumkaj-moklanjo.exe`; its MSI licence page said "Moklanjo" — fixed 2026-10-26.)

## Windows (cross from Arch / native on Windows)

**Decision (product):** Microsoft Store distribution is the **real Windows goal**.  
**Now:** per-user **MSI** (`./apps/moklanjo/scripts/msi.sh` → `dist/Moklanjo-<ver>.msi`, dev: `./apps/moklanjo/scripts/msi.sh dev`; Khaynjo: `./apps/khaynjo/scripts/msi.sh`) — single file, double-click install, Start Menu shortcut, uninstall from Settings → Apps. Installs to `%LOCALAPPDATA%\Programs\Moklanjo\`; **no admin**; fresh versions auto-replace old (MajorUpgrade). Uninstall keeps local data by default (`%APPDATA%\com.moklanjo\`) and asks separately whether to delete **business records**, **shop setup** (schema/databases), and **screen preferences**; Start Menu **Delete books data** offers the same categories. (Do not put prompts on the *install* path — that broke real installs earlier.) Folder + `Setup.bat` package remains the fallback.  
**Later:** wrap as **MSIX** for Store publish.

One **x86_64** PE binary covers **Windows 10 and Windows 11**. Dev and prod use the same env split as Linux/Android (`com.moklanjo.debug` vs `com.moklanjo`).

Makepad loads fonts from **`kumkaj_moklanjo/resources/`** next to the `.exe` (crate name uses underscores). Hyphenated `kumkaj-moklanjo/resources/` will not load UI fonts.

| | Development | Production |
|---|---|---|
| Command | `cargo moklanjo-windows-dev` | `./apps/moklanjo/scripts/package-windows.sh` |
| Output | `dist/moklanjo-windows-dev/` (copy whole folder) | `dist/moklanjo-windows/` (copy whole folder) |
| Install on Windows | Double-click `Setup.bat` | Double-click `Setup.bat` |
| Cargo profile | **release** (both — Windows debug PE was ~400MB) | **release** |
| What “Dev” means | `com.moklanjo.debug` + separate install/data + console | prod id, GUI subsystem |
| Env | `apps/moklanjo/env/development.env` | `apps/moklanjo/env/production.env` |
| Data on Windows | `%APPDATA%\com.moklanjo.debug\` | `%APPDATA%\com.moklanjo\` |

**Cross-compile from Arch Linux** (this host):

```bash
sudo pacman -S mingw-w64-gcc
rustup target add x86_64-pc-windows-gnu
cargo moklanjo-windows-dev          # Dev package → copy whole dist/moklanjo-windows-dev/ folder
./apps/moklanjo/scripts/package-windows.sh  # Prod package → copy whole dist/moklanjo-windows/ folder
./apps/moklanjo/scripts/msi.sh             # Prod MSI → dist/Moklanjo-*.msi (msi.sh dev for Dev)
apps/khaynjo/scripts/msi.sh        # Khaynjo prod MSI → dist/Khaynjo-*.msi (msi.sh dev → KhaynjoDev; own upgrade codes)
# Then on Windows: double-click the MSI (or Setup.bat in the folder package)
# Makepad: MAKEPAD_PACKAGE_DIR=. with kumkaj_moklanjo/resources/ beside the .exe
```

Linker is set in [`.cargo/config.toml`](../.cargo/config.toml) (`x86_64-w64-mingw32-gcc`). Day-to-day UI work stays on `cargo moklanjo-dev` (Linux); Windows scripts only produce `.exe` packages to test on a real Windows box (or Wine if you choose).

**On a Windows machine with Rust:** the same scripts pick `x86_64-pc-windows-msvc` when available and build natively.

### Receipts, printing and files on Android

Makepad exposes the clipboard (`Cx::copy_to_clipboard` → `ClipboardManager`).
`open_url` is still unimplemented. Kumkaj adds activity methods, applied
by `scripts/apply-android-activity-bridge.sh` before an Android build:

- **Copy receipt** puts the text on the system clipboard.
- **Print / share** prints on a network printer when one is set (Settings →
  Printers on this device); otherwise it opens an `ACTION_SEND` chooser
  (`shareText`). Either way it writes `receipts/receipt-YYYYMMDD-HHMMSS.txt`
  under the app files dir. Desktop prints through the system (`lp` /
  `Out-Printer`) — see LAN § Printing.
- A gallery or file-picker `content://` URI is read with `readContentUri`
  and saved as a real file before it becomes an item photo, venue icon or
  a backup to restore (`backups/picked-<ts>.sqlite3`).
- **Save a backup copy…** writes through the system save dialog
  (`ACTION_CREATE_DOCUMENT` → `FileDialogAction::SaveFileSelected`, not
  `FileSelected`) and `Cx::write_content_uri` (`writeContentUri`, Makepad
  patch `android_write_content_uri` + `patches/makepad/MakepadActivity-write.java`).
  Desktop keeps **Also copy backups to a folder…**. Both stamp
  `backup_copy_at`; after `COPY_DUE_DAYS` (7) with no copy off the device,
  Settings → Backup says so (`backup_copy_overdue_days`).

Because the receipt file is under `getFilesDir`, another app cannot open it
without a `FileProvider`; the share sheet and clipboard carry the text.

The dish photo shelf (`assets/items`, ~130 MB) is not packed into the APK,
so Android shows initials for shelf dishes until the owner sets a photo.

`arboard` and `rfd` are desktop-only dependencies — neither compiles for
Android, and `arboard` being unconditional used to break the Android build
outright.

## First-run unlock

A new server seeds an **Owner** whose PIN cannot be used: the first start
in Server Mode asks the Owner to choose a PIN (`complete_owner_setup`,
which refuses `0000`). Packages and guides must not mention a default PIN.

After updating past schema v8, all staff PINs reset to `0000` (fast local SHA-256 hasher — Argon2 froze the UI on unlock; local-device PIN threat model).

A shop that was **already trading** when that reset ran gets `meta.pin_reset_required = 1`,
and the Lock screen then refuses to open the till until the Owner picks a new PIN
(the first-run setup flow, reused — `Store::complete_pin_reset`, which rejects
`0000`). The login-id/password route is blocked while the flag is set. Fresh
installs never set the flag: they already have first-run Owner PIN setup.

Five consecutive wrong PINs block unlock for 30s with a countdown in the status
line. The counter and deadline live on `AppState` (in memory) — a restart clears
them, which is fine when the attacker already has the device and the real
protection is physical.

Lock screen: **Owner** | **Staff** (pick staff, then PIN), or optional **login id + password**.

Bottom tabs: **Books** · **Sell** · **Buy** · **More** (Items, sales/day close/reports/inventory/branches/credit/team/backup live under More). Feature flags hide More rows the signed-in person cannot use. Unlock lands on **Books**.

## Data location

See **Platform naming** / **Data directories** above. Short form: DB and backups live under the baked `KUMKAJ_APP_ID` on each OS; override with `KUMKAJ_DATA_DIR`.

### Backup format — unencrypted

**A backup is a plain, unencrypted copy of the SQLite database.** `Store::backup_to`
runs `VACUUM INTO`, so the file is one self-contained `.sqlite3` with every sale,
customer, credit balance, staff row and PIN *hash* in it. There is no password on
the file and no encryption at rest.

That means:

- Anyone who can read the backup file can read the shop's whole trading history.
  Treat it like the cash box: keep it on storage only the owner controls, and
  think twice before syncing the backups folder to a shared drive.
- PINs are salted SHA-256 hashes, not plaintext (4–12 digits allowed). Short
  PINs are still brute-forceable offline; a leaked backup should be followed by
  new PINs.
- Restoring does not need the original device, so a copied backup is a complete
  clone of the till.

Encrypting backups (SQLCipher or an app-level passphrase) is deliberately not
implemented — it is a Kumkaj Cloud concern, not a local-POS one.

### Restore (clean install)

1. Quit Kumkaj.
2. Copy a backup file to `…/<KUMKAJ_APP_ID>/backups/restore.sqlite3`.
3. Launch → More → Backup & settings → **Restore backup**.
4. Unlock and confirm today’s sales still appear.

Restore replaces the live DB through a temp-then-rename, then deletes the old
`-wal`/`-shm` sidecars — a stale journal replayed over restored pages used to
lose rows. `Store::restore_from_to_path` takes an explicit destination so the
behaviour is testable without touching the real data dir. Restore is refused on
a Client (it would overwrite the pairing with the Server's books).

Unit coverage: `backup_and_restore`, `backup_restore_preserves_sale_on_reopen`,
`restore_from_path_preserves_sale`.

## LAN Client / Server (`net.rs`, `phone.rs` — each app its own copy)

One device runs the server inside the app (Server Mode); other devices
pair to it over the shop's Wi‑Fi (Client Mode). No internet, no cloud.
**Any supported device can be either** — Windows, Linux or Android, in
any mix. Never write a guide, test plan or default that assumes a specific
platform as the server or client. What differs per platform (firewall,
start-up, keep-awake, printing, backup copies) is the table in
[`LAN_QUICK_START.md`](LAN_QUICK_START.md) §5 — keep it in step with the code.

| | Moklanjo | Khaynjo |
|---|---|---|
| TCP (requests) | **7857** | **7847** |
| UDP (discovery) | **7858** | **7848** |
| Windows firewall rule | `Moklanjo LAN` | `Kumkaj Khaynjo LAN` |

The two apps use different ports so both can serve from one device.

**Requests.** Plain HTTP/1.1 + JSON; every outcome is a JSON reply
(`{"ok":false,"error":…,"code":…}` on refusal), never a dropped
connection. Headers: `X-Kumkaj-Device` (the device token from pairing),
`X-Kumkaj-Session` (from `/login`), `X-Kumkaj-Request` (one id per write).

| Path | Who | Purpose |
|---|---|---|
| `GET /health`, `GET /hello` | anyone | liveness; app + venue name for discovery |
| `GET /track/{awb}` (Moklanjo) | anyone | public parcel status — no pairing |
| `POST /pair` | anyone (rate-limited) | pairing code → device token |
| `GET /sync` | paired device | snapshot of the books — **no PIN or password hashes, no pairing code** |
| `POST /login` | paired device | staff PIN checked **on the server** → session |
| `POST /sale` `/settle` `/undo` `/void` `/refund` `/amend` | signed-in device | money writes, done as the signed-in person |
| `POST /kot` `/running` `/kot_advance` `/table_billing` (Khaynjo) | signed-in device | kitchen and table actions |

**Rules that keep it safe to run all day**

- **Server is the sole writer.** A device's tables are a sync cache the next
  snapshot overwrites; `Store::assert_server_writer` refuses local writes on a
  client, so no UI path can bypass it. The `*_from_sync` writers skip the
  check — that is the server's own data arriving.
- **Each write once.** The server stores the reply per `X-Kumkaj-Request`;
  a retry after a lost reply gets the same answer, never a second sale or KOT.
- **Rights and PINs are judged on the server.** A manager's approval PIN is
  sent as typed; `check_right` (per-person rights, Team → Rights) decides.
  The approver is recorded and the reason names both people.
- **Brute force waits.** 5 wrong pairing codes or PINs from one address make
  it wait 5 minutes (`Guard`); the owner can issue a **New pairing code** and
  **Forget** a device (Settings → paired devices) — it is refused at once.
- **No "database is locked".** Each store sets a busy timeout and IMMEDIATE
  transactions; the server opens a connection per request beside the
  counter's own. Tests: `two_tablets_and_the_counter_writing_at_once_lose_and_double_nothing`
  (Khaynjo), `two_devices_and_the_office_booking_at_once_lose_and_double_nothing` (Moklanjo).

**The device side (`phone.rs`).** One worker thread talks to the server;
the screen never waits on the Wi‑Fi.

- *Queued* writes (payments, KOTs, settlements) are saved on the device first
  (`lan_outbox`) and sent in order — through a Wi‑Fi drop or an app restart.
  The header shows "Offline · N waiting".
- *Now* writes (undo, void, refund, amend) act on what the server holds this
  moment, so they are never kept for later: offline they say so.
- The worker pulls the server's books every 20 s (Khaynjo's Floor / Kitchen /
  Order also ask every 8 s), so what another device did shows up by itself.

**Keeping the server up.** While serving, the app holds the device awake
(`SetThreadExecutionState` on Windows, `systemd-inhibit` on Linux, the
screen kept on on Android — Makepad patch `android_keep_screen_on`).
Settings → **Start with this computer** (Windows, Linux; hidden where
`startup::supported()` is false, i.e. Android) adds a per-user entry — the
Windows Run key or an XDG autostart file — that launches with `--serve`
(`startup.rs`), so devices work again without anyone opening the app or
unlocking its PIN screen. Both run at the user's sign-in, so a shop that
wants it back after a power cut turns on automatic sign-in. An Android
Server has no equivalent: the app must be opened again.

**Firewall.** `net::firewall_allows_phones` checks; Settings → **Allow
other devices to connect** shows only when the firewall is in the way.
Windows: `netsh` inbound rules (TCP + UDP, `remoteip=localsubnet`) through
one UAC prompt — the per-user MSI cannot add them without admin. Linux:
whichever of firewalld (`firewall-cmd --query-port`) or ufw
(`/etc/ufw/ufw.conf` `ENABLED=yes`, `user.rules`) is on gets both ports
through one `pkexec` prompt; with neither on, nothing is asked. Android
has no inbound firewall for apps. Test: `ufw_settings_are_read`.

**Clock.** Every reply carries `X-Kumkaj-Time` (Server's Unix seconds);
the device keeps the difference (`net::clock_skew_secs`) and, past
`CLOCK_WARN_SECS` (5 min), the header says "This device's clock is N min
off the shop's server". Bills are stamped on the Server, but device-side
times (waiting writes, "last update") would mislead. Test
`replies_carry_the_server_clock`.

**Printing (`print_share.rs`, per device).** Printers belong to the
device, not the books: `printers.json` in the data folder holds a
**pool** of named printers (`id` / `label` / `target`) and **routes**
(ordered printer ids per feature). Settings → **Printers on this device**
edits both. A route tries the first printer, then the next on failure
(fallback, not fan-out); an empty route means that feature does not
print. Khaynjo routes: **Bill**, plus **KOT per kitchen station**
(`""` = unassigned / default kitchen). KOT lookup: station → `""` →
**Bill** (one printer on Bill still prints every kitchen ticket). Moklanjo routes:
**Receipt** and **AWB label**. Legacy files (`kot`/`bill` or `receipt`
strings) migrate on load; blank fields stay empty (no invented “Default” printer). Settings → **Find printers** lists system printers
(`lpstat` / `Get-Printer` on desktop) and hosts on this Wi‑Fi that
accept TCP on port 9100 (ESC/POS) — that scan is what works on Android
too. A target is a system printer's name, or an
address (blank = this computer's usual OS printer) — `192.168.1.50` or
`host:port`, default 9100.

**Print targets (owner chooses — large office or small kitchen):**

| Target | Behaviour (every supported OS) |
|---|---|
| Network `ip:9100` | Short ASCII ESC/POS over TCP |
| Named / default system printer | That OS print path (`lp` / `Out-Printer`) — paper size and driver are the owner's |

Do not hard-code “all KOTs must be raw” or “all Bills must be laser” in
code: shops put whichever printer they want on Bill vs each kitchen
station route. Network ESC/POS jobs stay short ASCII (max 4 KiB; no
default `GS v 0` raster). Failures via `take_problems`. Paired-device
KOTs / Print bill print at the Server.

**Backups.** The server writes a backup automatically once a day
(`auto_backup_if_due`; the last 14 are kept in the data folder's `backups/`)
and Settings → **Also copy backups to a folder…** keeps a second copy, e.g. on a USB drive.

**Discovery.** "Find the shop's server" broadcasts on the UDP port and falls
back to scanning the /24; replies list the venue name so staff pick the right
shop. Same Wi‑Fi trust model as before: no TLS.

## Branches = separate local servers (vision §5)

A branch is its own business, so it is its own **local server** (books):
Server Mode → create/open another. One set of books never holds two
branches. Apps may still carry a `branch_id` column and a single
built-in branch row (legacy shape; the books' own code/name used on
invoice numbers) — new code must not add in-book branch switching.

- Meta: `org_name` (the business name), `active_branch_id` (the books' single branch row)
- **Export:** day/period snapshot → `backups/branch_snapshot_{CODE}_{YYYY-MM-DD}.json` (+ clipboard)
- **Import (owner's roll-up):** place a branch's file as `backups/import_branch_snapshot.json`, then Import — read-only, never writes into the importing books' sales
- **Roll-up:** this restaurant's live day + imported snapshots side by side (stale/missing called out)
- Every finalize enqueues `sync_outbox` (`sale.finalize`). The queue is write-only; local billing never waits on it, and no process drains it until Kumkaj Cloud exists. When a drain marks an event synced, `mark_outbox_synced` also stamps `sales.server_at` (triple timestamps).

## Look at the screen (UI changes are not done until seen)

Tests and "no `[E]` at startup" do not show what a person sees. A floor
redesign once shipped uncoloured cards, black emoji dots, a lowercase
"floor" and off-centre initials because nobody looked (2026-10-23). Any
change to a screen is checked on the real app before it is called done:

```bash
Xvfb :99 -screen 0 1280x900x24 &          # a private display, not the desktop
env -u WAYLAND_DISPLAY DISPLAY=:99 KUMKAJ_DATA_DIR=/tmp/kd KUMKAJ_PRINT_SINK=off \
    ./target/debug/kumkaj-khaynjo &        # X11 backend on the virtual display
DISPLAY=:99 xdotool mousemove X Y click 1  # drive it
DISPLAY=:99 import -window root shot.png   # and look
```

Seed realistic data first (Khaynjo: `tests/visual_seed.rs`, opt-in) so
every state is on screen, not just the empty one.

Makepad visual traps found this way (do not reintroduce):

1. **`set_uniform(color)` does not restyle a widget** whose colour comes
   from its DSL `draw_bg.color` — use stacked pre-coloured backgrounds and
   show the one for the state (the overlay-faces pattern).
2. **Colour emoji (🟡 🟢 …) render as black glyphs** — never carry meaning
   in emoji colour; draw real swatches (`RoundedView` with a colour).
3. **A `Fill` label keeps its text at the top** — centre text in a `View`
   with `align: {x: 0.5, y: 0.5}` holding a `Fit` label (avatar initials).
4. **`TextInput` has no `visible`** — hide inputs through a wrapper `View`.
5. **Line height is larger than the font size** — budget ~1.6 × font size
   per label line when giving a fixed-height tile its height.
6. **A list's own `spacing` adds to its height** — content-sized lists
   count `spacing × (rows − 1)` or the last row is clipped.
7. **A key with no English entry shows the raw key** ("floor") — every
   label key needs `en()`; nav labels are in
   `english_chrome_never_returns_raw_keys`.
8. **Several wide buttons in a narrow column squeeze into vertical
   letters** — short chips in a wrapping row, or stack them.
9. **A `WrapLabel` (selectable `TextFlow`) takes the press** — a tappable
   row whose text is a selectable `WrapLabel` never sees the tap; set
   `selectable: false` on text inside anything you tap. **Everywhere
   else, display text stays selectable** (`WrapLabel` / `ReportText`) so
   the owner can copy totals, reports, and labels — see
   [`DESIGN.md`](DESIGN.md) § Text selection & copy. Do not size tile
   grids from shell `content_width` alone when the screen adds its own
   padding (Floor cropped the last column); measure with
   `cx.turtle().next_walk_width` in `draw_walk` like the dish grid
   ([`DESIGN.md`](DESIGN.md) § Always responsive).
10. **A redraw asked for inside a widget's own draw is dropped** — a
    content-sized list that finds its rows changed height while drawing
    must schedule `cx.new_next_frame()` and redraw on that frame, or a
    collapsed row leaves its old height as a gap.
11. **A filled field loses its placeholder** — forms then become rows of
    unlabelled numbers. Fields set `float_caption: true` (Makepad patch
    `text_input_float_caption`): once filled, the placeholder's short form
    stays as a small label; empty, the placeholder stays centred.
12. **xdotool cannot type into Makepad fields under X input methods** —
    launch the app with `XMODIFIERS=@im=none` on the virtual display.
13. **A screen scrolls as one page** (two separately scrolling halves were
    rejected 2026-10-24). A big list inside it is a bounded box with its own
    thin scrollbar and *scroll chaining* — the wheel scrolls the box while it
    can move that way, then the page (browser behaviour). Lists that grow
    with the page draw only the rows inside the window (`page_window`).
    A picker that needs the whole height (Khaynjo's menu: search, category
    tabs, dish tiles) is a `Modal` window over the screen instead — the
    page behind it holds still (2026-10-25).
14. **Sideways-scrolling tab strips get ‹ › chevrons** at the ends, each
    shown only while there is more that way (`TAB_STRIPS`,
    `sync_tab_chevrons` in each app). The chevron is *drawn* by the
    `ChevronBtn` face's shader (`draw_bg.point` 1 = ›, −1 = ‹), never a
    "‹ ›" glyph: a glyph sits on the text baseline and read low in the strip.
17. **Count every list a sync carries against months of data.** Khaynjo's
    open kitchen tickets were never closed by a kitchen that cooks from the
    paper KOT, so every tablet's sync grew to 3 MB at 9,000 bills (it is
    417 KB with tickets older than 12 h no longer open). The opt-in
    `months_of_trading_stay_quick` test in each app's `net.rs` seeds 9,000
    bills and fails if the snapshot passes 600 KB or a step takes 3 s.
15. **Native input behaviour, every app:** a visible page scrollbar; ↑/↓ a
    line, PgUp/PgDn a screenful, Home/End the ends (skipped only while a
    text field has focus — a focused button must not swallow them);
    Back/Forward history across screens (mouse back/forward buttons,
    Alt+←/→, edge swipe, Android back; lock and first-run setup clear it).
    Nested in-screen layers (Settings section → hub, open suggest list)
    dismiss on Back **before** leaving the screen — never skip to Day book;
    drag-to-scroll by touch only on vertical pages/lists — a mouse drag
    never flings them (wheel / scrollbar / keys). Horizontal tab strips
    (`ScrollXView`) allow mouse press-drag on the whitespace around the
    chips so nav drag-to-swap on a chip is not stolen; desk panes that
    fill the strip set `scroll_bar_x.mouse_capture_overload: true` so
    mouse press-drag still pans (Makepad patch `drag_scroll_touch_only`).
    Selectable `WrapLabel` / TextFlow yields sideways drags to an ancestor
    ScrollX (commits text-select only when the drag is mostly vertical —
    `text_flow_selection`). Nested scroll chaining: inner list keeps the
    wheel only while it can still move that way; at the edge the page/desk
    takes over (DEV §13). Every `ScrollYView` keeps **right padding 16**
    so content never sits flush on the vertical scrollbar; desk `ScrollX`
    keeps **bottom padding** clear of the horizontal bar.
16. **`ScrollYView{…}` in the DSL binds when the widgets are imported** —
    overriding `mod.widgets.ScrollYView` later changes nothing unless the
    name is re-bound (`let ScrollYView = mod.widgets.ScrollYView`). The
    old page-scrollbar style silently never applied because of this.
18. **A single `\n` in a `WrapLabel` is a space** (it is markdown): lines
    joined with `\n` collapse into one run-on paragraph — Settings → Units
    read as a wall of text (2026-10-26). A list or a set of figures goes in
    the app's `ReportText` card (rows built in code with `set_rows`, or
    printable text through `parse_report`); a blank line (`\n\n`) only
    splits paragraphs of prose.
19. **Right-aligned `Label` text overruns its box by a few px** — at a
    clipped edge (a card's right side in right-to-left languages) the
    last glyph is cut. Keep a right `margin` on such labels; `padding`
    would replace the theme's default label padding and shift the text.




## Smoke tests

```bash
cargo test -p kumkaj-moklanjo
cargo test -p kumkaj-khaynjo
```

**Tests never print on paper (all apps).** A test that reached `lp` /
the print dialog printed on the developer's real printer on every run
(Khaynjo, 2026-10-23). Every OS hand-off goes through one place per app
(`print_share::print_text`) and obeys a **print sink**:
`Os` in the app, `Capture` under unit tests (jobs kept per test thread;
tests assert the exact text and count via `print_share::take_captured`),
`Off`/`Capture` via `KUMKAJ_PRINT_SINK` for integration tests and smoke
runs. Paper is never the test oracle — captured text is, which checks
more (station, no prices, KOT number) than looking at a slip. Guard:
`apps/khaynjo/scripts/check-tests-never-print.sh` runs the suite with
fake `lp`/`lpr`/`xdg-open` first on `PATH` and fails if any was called.
`apps/moklanjo/scripts/check-tests-never-print.sh` is the same guard for
Moklanjo, which got its print sink (2026-10-24) so store-level printing
can never reach the OS printer from a test.

## Packaging

```bash
./apps/moklanjo/scripts/package-linux.sh       # prod → dist/linux/ (+ install.sh / uninstall.sh)
./apps/moklanjo/scripts/package-linux-dev.sh   # debug → dist/linux-dev/
./apps/moklanjo/scripts/package-windows.sh     # prod → dist/moklanjo-windows/ (+ Setup.bat)
./apps/moklanjo/scripts/msi.sh [prod|dev]      # MSI → dist/Moklanjo[Dev]-<ver>.msi
cargo moklanjo-windows-dev              # debug → dist/moklanjo-windows-dev/
./apps/moklanjo/scripts/package-android.sh
./apps/moklanjo/scripts/package-web.sh         # kumkaj.com site → dist/web/
./apps/khaynjo/scripts/windows-prod.sh # Khaynjo prod → dist/khaynjo-windows/
./apps/khaynjo/scripts/package-linux.sh       # Khaynjo prod → dist/khaynjo-linux/ (+ install.sh / uninstall.sh)
./apps/khaynjo/scripts/package-linux-dev.sh   # Khaynjo debug → dist/khaynjo-linux-dev/
./apps/khaynjo/scripts/msi.sh [prod|dev]
./apps/khaynjo/scripts/package-android.sh
./scripts/package-apple-scaffold.sh apps/<app>   # iOS/macOS deferred until Mac host
```

Desktop install/uninstall: see **Desktop install / uninstall** above.

## Unlock model

- **Owner** — hosts this local business; unlock with the Owner PIN chosen at first start
- **Staff** — pick a non-owner person, then their PIN
- Optional login id + password (Team can set once; only the user can change password later)
- More → **Team**: Owner creates staff (role title + permission preset + PIN + features); anyone can update their own PIN
- Sale **created_by** is set on finalize and never rewritten on amend

## Publish checklist (Play / web / desktop)

1. `cargo test -p kumkaj-moklanjo`
2. `./apps/moklanjo/scripts/package-linux.sh` → smoke `dist/linux/run.sh` / `./install.sh`
3. `./apps/moklanjo/scripts/package-windows.sh` (or `msi.sh`) → `dist/moklanjo-windows/` (+ `Setup.bat`); optional `cargo moklanjo-windows-dev` for Dev
4. Clone full Makepad `main`, set `MAKEPAD_ROOT`, run `./apps/moklanjo/scripts/package-android.sh`
5. Set `KUMKAJ_ANDROID_KEYSTORE` (+ `_PASS`) for step 4 — signed APK + AAB (Android § Release signing); create Play Console listing (title, screenshots from Stitch, privacy: local-only data)
6. `./apps/moklanjo/scripts/package-web.sh` → deploy `dist/web/` to **https://kumkaj.com**
7. iOS / macOS: scaffold only until Mac signing — `./scripts/package-apple-scaffold.sh apps/moklanjo`

Platform ids: see **Platform naming** above (`com.moklanjo` / `com.moklanjo.debug`, domain kumkaj.com).

## Manual UI smoke (shipping)

1. First Server run: choose the Owner PIN; later runs unlock with it
2. Finish onboarding — Moklanjo is locked; Fresh or Book, then Done
3. Booking desk: weigh / rate / cash or account; confirm bill totals (Actual / Applied / Rounded)
4. Day book shows today; Pay out records carrier bills & expenses; More → Parties settle; Day close; Reports
5. More → Rates: zones/services, tax rate name or %, optional photo (initials or custom — no food-menu pack)
5b. Settings → Country & tax: apply `GB` (VAT, inclusive, £), then `US` (sales tax, exclusive, $); save a business id and check it prints
6. Ops / COD / Riders when fleet features are used
7. Branches: create a second local server in Server Mode; export a day snapshot from each, import it on the owner's device, open the roll-up
8. Extra till: pair IP + code after the shop allows extra tills → book against the shop

## Schema

Current meta `schema_version` **1** (pre-release: one create, no upgrade ladder).

Fresh DBs get the full books shape in a single `CREATE TABLE` pass in `Store::migrate`:

- org / branches, categories, products/rates (tax, category, modifiers, stock, units), sales (invoice, tax, money lane, authorship, **triple timestamps**), sale lines (milli qty), payments, revisions, customers / credit + payable ledgers, purchases / purchase lines, expenses, money movements, stock movements, tax rates, units + conversions, sync outbox, staff features, **entry_attachments** (notes / WebP images / PDF on sale·party·expense·tracking). Moklanjo carries courier-agency scope only — no dining tables, kitchen stations or KOT schema; a future food product brings its own
- meta defaults: venue, onboarding, country pack (India until Settings changes it), invoice seq, entity template, etc.

Pre-release: if a local DB from an older numbered schema (2–17) misbehaves, delete it and reopen. After first public release, introduce `schema_version` 2+ only for real upgrades. Triple timestamps: [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md).

### Invoice numbers

Finalize reserves `INV-{BRANCH_CODE}-{seq:06}` from `meta.next_invoice_seq`
inside the sale's own transaction, so a crash can never hand the same number to
two bills. Branch code falls back to `MAIN` when blank. Amending corrects a bill
and keeps its number. Receipts print `Bill INV-MAIN-000042` rather than a
truncated UUID a customer cannot quote back.

### Stock is enforced, not clamped

`finalize_sale` refuses the whole sale when a tracked item is short (quantities
aggregated per product first), and the decrement carries a `stock_qty >= ?`
guard so a second till that took the last unit loses the race cleanly. The old
`MAX(stock_qty - qty, 0)` let the oversell through and left the count lying.

The Sell grid and cart `+` refuse first, via `Cart::add_or_bump_in_stock`, so
the seller finds out before the customer does. Void, quick undo and refund put
the units back and record a `void`/`undo`/`refund` stock movement — otherwise a
reversed charge would strand stock that strict enforcement then made unsellable.

## Fiscal country packs (`apps/moklanjo/src/fiscal.rs`)

`COUNTRY_PROFILES` holds one `CountryProfile` per country: tax system and name, inclusive/exclusive pricing, seed rates, optional receipt split (India CGST/SGST), instant-pay label, business-id label, credit wording, date order, and the number style (`decimal_sep`, `thousands_sep`). `XX` is the neutral custom pack and is the only one that leaves currency/language alone.

### Number style

`decimal_sep` and `thousands_sep` ride on the profile and are persisted as the `meta` keys of the same name. `thousands_sep` is read verbatim rather than trimmed, because a space (France, Sweden, South Africa) and an empty string (no grouping) are both real choices; `decimal_sep` falls back to `.` when blank.

- `locale::format_money_ex(minor, currency, decimal_sep, thousands_sep)` does the formatting; `locale::format_money` is the `.`/`,` default. `Money::format_localized(currency, &fiscal)` is the call site wherever fiscal settings are in hand — receipts, day close, the Sell strip, lists.
- `locale::parse_major_input` accepts every local habit (`13,5`, `1.234,50`, `1 234,50`, `1'234.50`). With both marks present the last one is the decimal; a lone mark three digits from the end is grouping, unless the currency really carries three decimals.
- Edit fields use `locale::format_major_input_ex` (local decimal mark, never grouped).

### Optional service charge

`meta.service_charge_bps` (0 = off) adds `lines_total × bps / 10000` on top of every bill, after tax, and is frozen onto `sales.service_paise`. No country pack sets one — it is a house rule, so it also survives a country switch. `FiscalSettings::service_on` is the single place the arithmetic lives, and `Store::preview_sale_total` includes it so split pay still balances.

Finalize math lives in `fiscal::{tax_on_amount, net_of_tax, gross_with_tax}`:

| Pricing | tax | sale total | net |
|---|---|---|---|
| Inclusive | `line × bps / (10000 + bps)` | `draft.total` (unchanged) | `draft.total − tax` |
| Exclusive | `line × bps / 10000` | `draft.total + tax` | `draft.total` |

The cart stays tax-agnostic under both modes — `SaleDraft::assert_balanced` only checks `lines_total + rounding = total`. `Store::finalize_sale` resolves the pack, freezes `tax_pricing`/`tax_label` onto the sale row, and uses the taxed total for the payment row, the credit ledger and day totals. `Store::preview_sale_total` gives the UI (and split pay) the same figure before charging.

The instant rail is still stored as `PayMethod::Upi` (`"upi"`) in every country; only `PayMethod::label_for` changes, so historical rows and sync payloads keep parsing. `SyncSnapshot` carries the Server's `fiscal` settings and `tax_rates` so Client terminals print the same wording.

**Not claimed anywhere:** statutory e-invoicing, fiscal-printer protocols, or tax filing. Seed rates are starting points a seller edits.
