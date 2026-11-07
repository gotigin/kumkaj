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

## Test harness (copy-paste — do not rely on memory)

From the **workspace root**. Never opens on your desktop `:0` / Wayland session.

```bash
# Fast money/KOT/COD/khata paths (Store API, ~seconds, no GUI)
./scripts/run-user-journeys.sh              # both apps
./scripts/run-user-journeys.sh khaynjo
./scripts/run-user-journeys.sh moklanjo

# Real UI on private Xvfb (seed → unlock → peers/actions → PNGs → PASS/FAIL)
./scripts/run-ui-smoke-isolated.sh          # both apps, full desk day (default)
./scripts/run-ui-smoke-isolated.sh khaynjo
./scripts/run-ui-smoke-isolated.sh moklanjo
SMOKE_SCRIPT=peers ./scripts/run-ui-smoke-isolated.sh khaynjo   # chrome peers only
SMOKE_SCRIPT=soak KUMKAJ_SMOKE_SOAK_SECS=86400 \
  ./scripts/run-ui-smoke-isolated.sh khaynjo                    # ~1 day soak
```

Shots + `RESULT.txt` land under `/tmp/kumkaj-ui-smoke-<pid>/` (or `SMOKE_OUT=…`).
Detail: § Look at the screen below. Cargo aliases for books only:
`cargo khaynjo-journeys` / `cargo moklanjo-journeys` (prefer the script so
printers stay fenced).

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
  - After patches, `scripts/fix-makepad-rust-deprecations.sh` migrates deprecated `std::u64::MAX` / `core::usize::MAX` / `Atomic::fetch_update` → associated `MAX`/`MIN` and `try_update` (real API updates, not `allow(deprecated)`). Android scripts also run it on `MAKEPAD_ROOT` (`vendor/makepad-full`, used only for `cargo-makepad`) so device builds stay clean
  - `vendor/makepad` is gitignored: **an edit there is lost on the next setup unless it is also a patch in `patches/makepad/` listed in `PATCHES`**. After changing a vendor file, regenerate its patch, then rerun the setup in a scratch copy and diff it against `vendor/makepad`
  - Nested Settings lists claim wheel events so the page does not scroll with the list
  - Page `ScrollYView` keyboard: ↑/↓ step, PgUp → top, PgDn → bottom (`App::handle_scroll_key`). Skipped while a TextInput has key focus.
- Multi-script UI fonts live **per app**: `apps/moklanjo/resources/fonts/` and `apps/khaynjo/resources/fonts/` (Noto scripts, FA icons, LXGW CJK, Noto Color Emoji — each app carries its own copy, like its code). Re-fetch with `./scripts/fetch-ui-fonts.sh apps/<app>` (no argument: every app) if fonts are missing. **Desktop icon fonts — do not regress (2026-11-02):** see § WorldFonts / packaged icon fonts below.
- **OS-native interaction** — the host OS must be able to treat our app like
  other apps on that OS (shortcuts, window/shell behaviour, platform
  conventions). See § OS-native interaction below.
- **Desktop window size & maximize** — see § below (detail under OS-native).
- **Windows console** — production builds (`KUMKAJ_ENV=production` / `package-windows.sh`) use the GUI subsystem (no terminal). Dev builds keep a console for logs.
- **Min layout width 375 css px** — `app_shell` uses `Fill{min: 375.}` inside a horizontal scroll. The UI never lays out narrower than 375; if the device/window is below 375, content scrolls horizontally (no crop). Desktop resize is also floored at 375.
- **Dev size** — `W×H` in the window caption bar, left of the Kumkaj title (terracotta on navy). Controlled by `KUMKAJ_SHOW_SIZE_BADGE` in the env file (not Cargo `--release`).
- **Button label inset** — every face within an app sets layout `padding` **and** `label_walk.margin` (Makepad 2 Fill + Ellipsis otherwise pins ink to the face edge)
- **Reusable layout widgets** — see § Makepad 2 UI faces (agent guide) below (selection + truncation). Product summary: [`DESIGN.md`](DESIGN.md) § Makepad 2 faces.

## OS-native interaction (required)

**Rule:** On every supported OS, the system must be able to interact with our
app the **same way it normally interacts with other apps** on that OS —
window management, shell/task switching, global and WM shortcuts where those
apply, IME / Back / multi-window on Android, and the rest of that platform’s
ordinary conventions.

This is **cross-platform**, not desktop-only and not tied to one distro or DE:

| Platform | Expect |
|---|---|
| Desktop Linux | Behave as a normal windowed app for that compositor/DE (Wayland or X11; GNOME, KDE, others — **not** “Arch + GNOME only”) |
| Windows | Snap, Win+ shortcuts, multi-monitor move, taskbar — like other Win32-class apps |
| macOS | Spaces / Mission Control / standard window shortcuts as other Mac apps |
| Android | System Back, IME, task switcher / multi-window, PrintManager where relevant |

**Touch screens are first-class on every platform above.** A finger on Linux, Windows, macOS, or Android is a touch, not a mouse click. Both apps must stay fully usable on a touch screen of any supported OS: taps land, scrolling pages and lists follow the finger, text selection starts on a long-press, and nothing the finger needs (the control under a field, the row under a note, the button at the bottom of a page) is clipped or covered. Desktop mice and trackpads stay mice. Do not gate a gesture on “this OS is a phone.”

Makepad draws **client-side window chrome** and does not always expose full OS
window / shell integration by default. **If the stack does not provide that
naturally, we must implement or patch until the OS can treat the app as a
normal peer.** That work is product-required, not optional polish. Do not
“fix” one developer’s desktop by hard-coding that DE’s quirks as the only
truth — fix the portable integration, then keep DE-specific notes only as
regression history.

Apps (Moklanjo, Khaynjo, …) share the vendored Makepad tree; window/shell
fixes usually land in `vendor/makepad` (+ patches) and apply to every app.

## Desktop window size & maximize

Expected floating-window behaviour on **desktop** Linux, Windows, and macOS
(client-side chrome):

| Action | Expected result |
|---|---|
| First launch | Floating window **1024×768** (DSL + store defaults) |
| Later launch | Last **floating** size/position from `window_geom.json` under the app data dir |
| Max button (single click) | Toggle maximize ↔ restore |
| Caption bar double-click | **Same** toggle as the max button (not a second code path) |
| Restore with no saved float size | **1024×768** (never a monitor-fill size) |
| Resize floor | **375 css px** width and height |
| OS / WM shortcuts | Same as § OS-native interaction — snap, move across displays, etc. must work like other apps on that OS |

`window_geom.json` must **never** store maximized/fullscreen geometry. Persisting ~monitor size (e.g. `2560×1048` on an ultrawide) made the next launch look maximized while floating; the next `set_maximized` then fought the compositor on multi-monitor layouts.

### Linux compositor notes (do not regress; not “Linux = GNOME”)

Historical bugs were hit on **GNOME Mutter / Wayland**; the same maximize /
float rules must hold on other Linux DEs. Vendor patches live under
`vendor/makepad` (path-dep). Toggle logic is owned by
`Window::toggle_chrome_maximize` in `vendor/makepad/widgets/src/window.rs`.
Platform ops: `MaximizeWindow` / `RestoreWindow` in `linux_wayland.rs`.

RTL text alignment rides on two more vendor patches: `TextFlow::paragraph_align_x` in `vendor/makepad/widgets/src/text_flow.rs` (resting paragraph alignment; open table cells still override it) and `pub TextInput::label_align` in `vendor/makepad/widgets/src/text_input.rs` (patch files `patches/makepad/text_flow_paragraph_align.rs.patch` and `patches/makepad/text_input_label_align_pub.rs.patch`, applied by `scripts/setup-makepad.sh`). Each app sets them from `apply_text_alignment` while `locale::is_rtl` is active; glyph shaping and bidi ordering are Makepad’s own (rustybuzz + unicode-bidi in `draw/src/text/shaper.rs`). The window layout is not mirrored.

**Mouse cursor state** (`patches/makepad/mouse_cursor_state.rs.patch`): Wayland mapped `MouseCursor::Arrow` to the context-menu shape (focused buttons looked like a menu); it now matches Default. Widget `SetCursor` is ignored while the pointer is in the window-edge resize band so Text/Hand cannot wipe Nw/E/S/…. `View` / `Button` / `TextInput` re-assert their cursor on `FingerHoverOver` (HoverIn only fires once); focused buttons keep Hand; caption drag uses Grab. Both apps also set `ColResize` while hovering the rail divider.

**Bugs we hit (do not reintroduce)**

1. **`display_context.screen_size` is the window size** — never use it as “fills the screen ⇒ maximized”; that made every max click take the restore path.
2. **Stale maximized configure after restore** — re-latched `chrome_maximized` so later clicks only restored. Suppress compositor “adopt maximized” briefly after a chrome restore/maximize.
3. **Second `set_maximized` while Mutter still has maximized** — on a 2×2 ultrawide layout this jumped toward another output and then left maximize dead. Track `compositor_maximized` from configure `states` only; skip `set_maximized` when it is already true.
4. **Stale floating configure after restore** — the compositor often echoes the old maximized rectangle with maximized unset. Ignore a floating configure that is that exact imposed rectangle. Do not reject sizes with a pixel cutoff; that blocked snap on other displays.
5. **Caption double-click must not call `xdg_toplevel.move` on the first press** — that ate double-click and used a divergent compositor-only toggle. Defer move until pointer travels &gt;5px; on double-click emit `Event::WindowChromeMaximizeToggle` so the **same** widget toggle runs as the max button.
6. **Snap, maximize, and restore are the platform’s size.** On Wayland, bind `xdg_wm_base` new enough to receive tiled-edge states, and use the configured size whenever the state is maximized, fullscreen, or tiled. Do not code one desktop’s Super or Win+arrow chords into the app. Windows keeps a normal overlapped window so the shell can snap it.

**Code map**

| Piece | Role |
|---|---|
| `Window::toggle_chrome_maximize` | Single-click max + caption double-click |
| `Event::WindowChromeMaximizeToggle` | Wayland caption → widget toggle |
| `WaylandWindow::compositor_maximized` | Last configure maximized/fullscreen bit |
| `pending_restore_size` / `imposed_extent` | Floating size; ignore an echo of the old imposed rectangle |
| `persist_window_geom` (each app) | Skip maximized/fullscreen. Keep a large floating size. |
| Dev logs | `MaximizeWindow` / `RestoreWindow` / `xdg_toplevel configure` / `caption double-click` |

```text
❌ Persist maximized size into window_geom.json
❌ set_maximized again while compositor_maximized
❌ Treat a large configure as “the monitor” and replace it
❌ Implement one desktop’s snap keys inside the app
❌ Caption double-click toggles in wayland_state instead of the widget
❌ Start xdg_toplevel.move on the first caption press of a double-click

✅ One toggle path (widget) for max button and caption
✅ Obey maximized, fullscreen, and tiled configures
✅ A floating echo of the last imposed rectangle does not replace the restored size
✅ Skip set_maximized when already maximized per configure states
```

Manual check (any desktop Linux DE + Windows + macOS as available): max →
restore several times; caption double-click the same; **OS shortcuts** for
snap / move to another display work like peer apps; Android: Back / IME /
task switcher behave normally. A snap shortcut must change the window
the same way it does for any other window on that system.

**Add dishes → Done (do not regress).** `button.clicked` / `pressed` does not
arrive for that control while the search field is focused, so a click check
alone leaves the window open. `close_add_dishes_if_done_hit` closes on
pointer-down when the point is on the button rect or in the top-right of
`menu_panel`, and it returns before `ui.handle_event`. The face is a plain
`Button` at 140×44, not `RowSecondaryBtn`. Do not replace this with only
`clicked`.

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
6c. **Floor (and other heavy expand-to-content grids) freeze Android** — Same full-height PortalList rule as (6): Khaynjo Floor with ~44 tables drew every `FloorTile` (Overlay + 7 bg faces + icons) every frame, so the phone could not scroll (2026-11). **Rule:** fixed-pitch expand-to-content lists that are expensive per row use `page_window` / `floor_page_window` + `top_gap`/`list_box` (see `MenuList` / `FloorList`) so only on-screen rows draw; redraw on `Event::Scroll`. Also never put `Label{width: Fill, flow: Right{wrap: true}}` on those tiles (bug 7). Books/Inventory/Reports facet SQL must not re-run on every `sync_chrome` Signal — gate with `FACET_DIRTY` / empty text.
6b. **Android 8.0 (API 26) boot crash / black SurfaceView** — (1) `InputDevice.isEnabled()` is API 27+ → `NoSuchMethodError` (guard `SDK_INT >= 27`). (2) Inline `WindowInsetsAnimation.Callback` in `ResizingLayout` makes old ART fail class verify (`ClassNotFoundException`) and leave a black GL surface — keep the callback in `ImeInsetsAnimationHelper.java` and attach only via `Class.forName` when `SDK_INT >= 30` (`scripts/apply-android-activity-bridge.sh`). (3) Cargo plain `dev` on phone is too slow for first paint; use `cargo *-android-dev` (`android-dev` profile). That profile must stay **opt-level 3** with only `debug = "line-tables-only"` — full DWARF (`debug = true`) made a ~400MB `libmakepad.so` whose load alone left a black SurfaceView for many seconds on Mali-T830. (4) First drawable can precede layout settling — Makepad does a few follow-up `redraw_all`s (`android_post_layout_redraw` patch) so DeviceSetup is not stuck on `pass.clear_color`. (5) On some Huawei API 26 devices `adb exec-out screencap` captures only the window background and **misses the Makepad SurfaceView** — use a system screenshot (`adb shell input keyevent 120`) to verify paint.
6d. **Android soft keyboard ▾ hide bounces back / retap silent** — Two related Makepad IME bugs (patches `android_ime_dismiss` + `android_ime_retap_show`; Java `SHOW_FORCED` / InsetsController via `apply-android-activity-bridge.sh`). **(▾ bounce)** System hide left the `TextInput` focused; a `ShowTextIME` already in `platform_ops` (or the next draw) called `showKeyboard` again after `last_ime_config` was cleared. Fix: on `ResizeTextIME` DidHide (no physical keyboard) call `text_ime_was_dismissed()` only (mirror iOS); skip `ShowTextIME` while `text_ime_dismissed`; never re-arm the dismiss latch in `CxKeyboard::set_key_focus` when the new area is `Empty`. **Do not** `set_key_focus(Empty)` inside DidHide — that fires KeyFocusLost → `hide_text_ime` (clears the latch) and mid-gesture focus loss wrecks Android touch-scroll (ScrollY + ScrollX / `axis_lock` / `capture_overload`). App Back / outside tap still clears focus via `dismiss_soft_keyboard`. Do **not** “fix” bounce by dropping `ime_show_requested` / retap redraw — that regresses retap. **(Retap silent)** Config-only dedupe of `ShowTextIME` left retaps no-ops when OEM dismiss kept `last_ime_config` set. Fix: `request_ime_show()` on KeyFocus / FingerDown + redraw; `ShowTextIME` calls `showKeyboard` when config changed **or** `ime_show_requested`. Never treat `!last_ime_visible` alone as show-again.
6e. **Do not regress Android / desk touch-scroll when fixing IME** — Load-bearing Makepad patches: `scroll_axis_lock`, `drag_scroll_touch_only`, `scroll_touch_over_buttons`, `scroll_touch_orphan_unblock`, `text_flow_touch_scroll`, `text_input_touch_scroll`, `text_flow_selection`. Page `ScrollYView` and Floor `area_scroll` keep `mouse_capture_overload: true` + `axis_lock: true`. **Never** clear key focus from `ResizeTextIME` DidHide / keyboard-height callbacks (bug **6d**). **Never** “simplify” capture_overload off, drop axis_lock, or re-enable TextFlow / TextInput select-drag on touch FingerMove (that steals vertical swipe — Lock PIN and ClientPair fields need `text_input_touch_scroll`). **`WrapLabel` default is `selectable: true`** (both apps — [`DESIGN.md`](DESIGN.md) § Text selection & copy). Set `selectable: false` **only** inside tap targets (Kitchen KotCard, cart lines, floor tiles, snack bars, button faces, InfoHelp titles) — select-drag there steals the press / can freeze page swipe until FingerUp. Touch select starts on long-press (`text_flow_touch_scroll`); vertical swipe still scrolls. KitchenList (and other heavy expand-to-content lists) use `page_window` + `top_gap`/`list_box` so touch-move redraw does not draw every card (bug **6c**). When adding whitespace above a sideways Area strip (Floor Columns↔tables gap), put the pad **inside** that `ScrollXView` — parent `spacing` between Columns and `FloorList` is a dead zone for Area pan.
7. **Settings / CTAs freeze the UI (wrap + Fill)** — Separate from bug **7a**. Makepad turtle used to `error!`/`println!` **every frame** on `flow: Right { wrap: true }` + `width: Fill` child; that log spam freezes the app on every OS, including Windows GUI builds with no console (Khaynjo Settings → Add printer, 2026-10/11). Signature: high CPU / draw path / turtle warning. Hard stop: patch `turtle_wrap_fill_no_spam` logs **once** and still draws. Still avoid the bad layout:
   - **Button face:** any `flow: Right{wrap: true}` face must walk its label as **Fit**. Patch `button_label_center`. **Do not** “fix” by removing wrap or left-aligning labels (bug 3).
   - **Parent View:** never put `width: Fill` children inside `flow: Right{wrap: true}`. Use `ActionStack` / `flow: Down`, or Fit chips.
   - **Busy chrome:** never `set_text` / toggle `visible` on Fill+wrap Stack\* for Looking/Adding/Cancel. Cancel is `QuietFitBtn`; progress on a WrapLabel.
   - **InfoHelp:** Titled → **`width: Fill`** + **Fit title + tip + Fill trail** (ℹ hugs label on every screen); tip-only → trail hidden, **Fit**. Never `Label{width:Fill, wrap}`. One screen/section title — never two titled InfoHelps that both say the same heading (Printers regression 2026-11).
   - **CalloutTooltip (every tip):** Long tip text must **wrap** inside the bubble — never crop mid-word on the right. Shared fix in `vendor/makepad/widgets/src/callout_tooltip.rs` (readable width cap, triangle inset in label width, `flow: Right{wrap: true}` on the tip label). Do not “fix” one tip string; fix the widget.
7c. **Large face looks tappable but only a thin strip works** — e.g. `AccordionHdrBtn{width: Fit}` alone in a Fill card (Device setup / Floor, 2026-11): Makepad hit-tests **`draw_bg.area()`**, so the rest of the row is dead. **Usual fix:** keep the **control’s face** large enough for the job — accordion / hub / sidebar / checklist rows **Fill**. When an InfoHelp must sit **right after** the title: Fit caption + Fit InfoHelp + Fill `_ext`, all as **`AccordionHdrHitBtn`** (same hit metrics, **no hover/down paint**, **no `wrap: true`** — Fit+wrap expands the caption and parks ℹ at the far end) so the row reads as **one** header on the card — not three cream segments. Alone (Books): painted `AccordionHdrBtn` Fill (wrap OK). Titled InfoHelp already packs ℹ next to the label (Fit title + trail); accordion tip-rows still use **Fit caption + tip-only + `_ext`**. **Same idea, most places:** checkbox / radio / switch rows should toggle from the **label + control**, not only the 18 px box (Khaynjo `StationCheckRow` already does). **Exceptions are fine** when a full-row hit would steal another gesture or confuse the screen — e.g. sidebar **pill switch** keeps its own `SidebarToggleHit` so the row can still drag-reorder; ↑↓ on checklist rows stay their own buttons. Prefer the generous hit; carve out an exception only with a short comment why.
7a. **Settings “Not Responding” on printers place pick = `app_state` deadlock (not wrap/Fill)** — gdb on hung Khaynjo (2026-11-03): main thread `futex_do_wait` / `Mutex::lock` on `app_state::STATE` inside `set_status_line_only` ← `set_status` ← Bill/kitchen place tap. Cause: `self.set_status(cx, app_state().lock().unwrap().t(...))` — temporary `MutexGuard` lives for the whole statement, so `set_status` locks again on the same thread and never returns. Low CPU, not turtle spam. **Rule:** never pass `app_state().lock()…` into a function that locks `app_state`. Use `app_t("key")` or `{ let st = app_state().lock().unwrap(); … }` first. Cursor rule: `.cursor/rules/app-state-lock.mdc`. Do not “fix” this hang by swapping button faces. **Named guards too (2026-11-07):** a `let mut st = app_state().lock().unwrap();` still alive when a repaint helper runs is the same hang — Khaynjo Settings → Standard size (`sync_std_print_ui` under `st` while typing a digit) and Moklanjo Settings hub (`paint_header_back_chrome` at the end of `sync_settings_panels`, every section open). Scope the guard in a block (or `drop(st)`) before any `self.sync_*` / `self.paint_*` / `self.show_*` / `set_status` / `go`. Each app's `scripts/check-lock-guards.sh` now flags those names under a live `st` (only `paint_choice_pair`, which never locks, is exempt).
7d. **Bottom-nav footer: scrollbar flush, chips not cropped (2026-11-04)** — Phone chrome `bottom_nav` is a Fit `SolidView` + `ScrollXView` (`bottom_nav_scroll`) + overlay ‹ ›. Makepad paints the X bar **over** the bottom `bar_size` of the scroll view (it does not grow the Fit height). Contract (both apps; Cursor rule `.cursor/rules/bottom-nav-footer.mdc`):
   - **`bottom_nav.padding.bottom` = 0** in DSL. `lift_footer_above_system_bar` may set it to **safe-area inset only** (0 on desktop). Any extra bottom pad sits *under* the scrollbar and reads as a dead strip below the bar.
   - **Strip inside the scroll view** keeps **bottom pad ≥ `bar_size`** (today 8 with `bar_size` 4) so the bar’s lane is empty maroon/navy under the chips — zeroing that pad pins chips/‹ › to the window edge (looks sunk/cropped).
   - **`lift_footer` must only write `padding.bottom`.** Writing `padding.top` too made the Fit footer taller than its shell slot; the parent clipped the bottom (chips/‹ › sunk, empty chrome above).
   - Paint the footer X bar **dark on the footer colour** (not stock light handle) so the floor does not read as a cream strip. Keep the bar (do not “fix” by `bar_size: 0` while chevrons exist — the bar is the bottommost chrome).
7b. **UI must never wait on I/O** — Find printers, firewall, LAN discover/pair, LAN image fetch, thumbs, and **Create backup** (`VACUUM INTO`) run on worker threads and wake the UI with `SignalToUI`. Printing itself is queued to the `print` worker (`lp` / PowerShell / TCP must not run on the UI thread). A click handler may only write a small file / update state, then return. **Pair finish:** `client_pair` HTTP stays on a worker; applying the sync snapshot (`net::apply_snapshot`) must also stay off the UI thread (`LanOnce::PairApplied`) — a busy shop’s SQLite replace froze Android for seconds after “Pair now”. **Soft keyboard on ClientPair / Lock:** do not run a full `sync_chrome` (locale + Settings SQL) while a text field holds focus — that rebuild hid the IME; digits still go into the field, Pair/Unlock triggers a real sync. **Server pairing digits** in Settings (`pairing_code_value`) are a selectable `WrapLabel` (display/copy only — use **Copy pairing code**); the Client types into `client_code_input` (`FieldInput`, `is_numeric_only`).
7e. **A sheet that fills the window never dismissed (Khaynjo, 2026-11-07)** — Makepad's `Modal` closes on a release outside `content`; Add dishes and the UPI QR set `content` to Fill, so nothing was outside and the dimmed area did nothing. The app decides from the visible sheet (`sheet_press_target` / `sheet_press_closes`): a press on the dim or on Done closes at once; a press on the dishes owns the whole gesture (a scroll ending outside or over Done never closes). While a sheet is open, nav-tab drag, the rail divider, Floor area drag, sidebar drag, printer-row taps and edge swipes do not start under it; mouse Back/Forward still run and `navigate_back` closes the sheet first. A clear-backdrop dropdown modal (Floor Columns) closes on the same outside press, or it swallowed every Floor tap until the next repaint.
7f. **Fit Overlay + Fill layer = zero-height or collapsed row** — In a `flow: Overlay` view with `height: Fit`, a `height: Fill` child gets the height used *so far*. A background layer drawn before the content gets zero (no card, no ageing colour on Kitchen tickets) or a wrong height while a list scrolls (Add dishes rows drew as thick bars). Give the card a fixed height that matches its list pitch (`ProductCard` 176, `KotCard` 200 / `KOT_ROW` 208), or make the background the list item's own (`RoundedView` root; one template per state, e.g. SuggestList `Item` / `ItemSel`). A Fill layer *after* the height-giving child (a hit button) is fine.
7g. **Edge swipe vs a control in the edge strip** — The 48 px back/forward swipe arms on press, but after the UI handled the press `disarm_edge_swipe_if_claimed` drops it unless nothing, or the page's own scroll view, claimed it (`Event::pointer_claimed_area`). A button, field, card, list, or sideways strip at the edge wins: panning Moklanjo's booking desk or a tab strip from the edge used to also go Back/Forward.
7h. **Row and card taps use `finger_up_is_tap`, not `was_tap()`** — `was_tap` gives up after 0.5 s or 5 px (mouse), while the page waits 8 px before scrolling: a slow press or a 6 px drift was neither a tap nor a scroll. A tap read from raw pointer events (printer rows) needs press and release on the same row within tap travel — a release alone is the end of a scroll.
7i. **Never pre-mark a scroll handled before dispatch** — Makepad hands a wheel / trackpad delta to the innermost scroll view that can move and marks the axis handled only when it moved. Setting `handled_x` before `ui.handle_event` (`claim_hstrip_nested_scroll`, removed from Moklanjo 2026-11-02 and Khaynjo 2026-11-07) made each sideways strip move one step and then stop.
8. **Page-scroll list rows cropped / page won’t scroll** — A `PortalList` inside a page `ScrollYView` must claim **full content height** (`list_content_height` + `page_owns_scroll`) so the page owns the wheel. Guessing a short Fixed row pitch (Khaynjo Inventory `StockList` at 62 while `StockRowView` is Fit, 2026-10) clips the last rows into the next sibling (tip-pool `ReportText`) and leaves the page with almost nothing to scroll — the “missing” rows are clipped *inside* the under-tall list, not below it. The same clip hid Khaynjo’s open ticket line: a fixed 220px extra cut off Cancel / NC / Move under the note and pulled Kitchen station up over them. `cart_open_extra` is the measured details — not a constant, and not a padded blank band under the buttons. Do not auto-scroll the order page to the station control when a line opens. **Rule for every app:** Fit-row expand-to-content lists use a **generous seed + `remember_content_row`** (same pattern as `CustomerList` / Moklanjo `SaleList`); an opened or taller row reserves its **measured** height and redraws once (`CartList` settle) — never a fixed “extra” that is shorter than the details. Fixed-pitch cards (`MenuCard` 140, `DishRow` 104) keep a constant that matches the DSL. Never “fix” one screen’s crop without sweeping the same hardcoded underestimates elsewhere. Do not reintroduce nested list scroll for these lists — that eats the page wheel at the bottom edge.
9. **Nav hid peers but Day book still showed them (Waiter)** — Chrome used `nav_tab_permitted`; unlock always opened `Screen::Books` and hub rows / TODAY money stayed visible (Khaynjo 2026-11-04). **Rule:** unlock → `unlocked_home_from`; Day book hub slots + money accordions use the same caps as nav. Full table: § Staff rights vs chrome below.

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

Hard rules: never use a filled primary to mean “this navigate link is more important”; never use `ChipBtn` as a one-shot CTA (NDR / Show PIN / line Cancel → `QuietFitBtn`); exclusive closed sets (tax invoice, amend pay, paid/AP, staff role, …) → `SegBtn`, not Stack primary/secondary twins; short filters (units, catalog facets) → `ChipBtn`; `PayBtn` is ticket take-payment only. **Never** put `Row*` / `Stack*` (Fill + `wrap: true`) at `width: Fit` — wrap measures one glyph wide and stacks letters vertically (`S`/`h`/`o`/`w`…); use `QuietFitBtn` (Fit, no wrap) for inline quiet actions. **Hit targets (most cases, bug 7c):** accordion / hub / sidebar / checkbox·radio·switch rows — tap the **whole face or label row**, not a tiny glyph. When ℹ must sit beside the title, use **Fit caption + Fit InfoHelp + Fill `_ext` as `AccordionHdrHitBtn`** (invisible face, **no wrap** — one header, not painted segments), not “park ℹ at the far end.” Short labels: **Fit WrapLabel + tip-only InfoHelp**, never titled Fill InfoHelp. **Exceptions OK** when a full-row hit would break drag, steal ↑↓, or otherwise feel wrong — document the reason at the call site.

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

Implemented in `pointer_nav_drag` / `nav_drop_slot` / `nav_drag_autoscroll` (`main.rs`). Behaviour contract: tap = navigate; press on a tab + move = drag (ghost chip follows, drop target wears the active face, release swaps and persists); press on whitespace = scroll; hold a dragged tab at a bar edge = auto-scroll. Mouse (desktop) and touch (`TouchUpdate` on Android) share the same press/move/release path — phones never get MouseDown. This works today — do not “simplify” the pieces below; each one is load-bearing.

1. **Slots are positions.** `paint_nav_tabs` paints slot N with `nav_order[N]`; the order lives in books meta `ui_nav_order` (`Store::nav_order_csv` / `set_nav_order_csv`). A swap exchanges two slots and re-saves. Never hard-code tab→slot.
2. **Spatial gesture split only.** The bars keep whitespace lanes around the tabs (padding **inside** the scroll strip — not under the X scrollbar; see bug **7d**). Bottom bar today: end pad ≥ 46 (chevron clear), top ~6, bottom ~8 (≥ `bar_size`), chip gaps ~16; rail ~12px sides. The pickup hit test (`nav_slot_at`, pad 0) must stay exact — any *extra* padding on the chip itself eats the lane and lane swipes start drags instead (observed: a 5px pad turned an above-chip swipe into a swap). Drop targeting may pad (2×5px). Do not reintroduce direction- or timing-based gesture decisions (“sideways = scroll”, hold-to-pick-up timers): the natural sideways drag then never picks up, which reads as “drag is disabled”, and timing misfires on laggy remote input.
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

`apps/moklanjo/build.rs` bakes the selected env file into the binary (`KUMKAJ_ENV` wins; otherwise debug Cargo profile → development, release → production). **Cargo profile ≠ app env:** `cargo moklanjo-windows-dev` uses the **debug** Cargo profile with `development.env`. `cargo moklanjo-android-dev` / `cargo khaynjo-android-dev` use workspace profile **`android-dev`** (release-speed native code, still `android:debuggable=true` and development app id) — plain Cargo `dev` on API 26 phones often never paints (black SurfaceView). Production package scripts use `--release` (`debuggable=false`).

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
| **Web** | local `cargo moklanjo-dev` / wasm as configured | **moklanjo.com** (`apps/moklanjo/web/`, `./apps/moklanjo/scripts/package-web.sh` → `dist/web/moklanjo/`); brand **kumkaj.com** (`web/kumkaj/`); **khaynjo.com** (`apps/khaynjo/web/`) — build any site with `KUMKAJ_SITE_DIR=… ./scripts/build-site.sh` |
| **Play Store** listing | side-load / internal only | application id `com.moklanjo` (`apps/moklanjo/web/static/play-store-listing.txt`); privacy **https://moklanjo.com/privacy** |

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

**DeviceSetup (every launch):** language (platform default, immediate apply) + Server/Client. **Offline LAN:** Server Mode creates the **one** local server if none exist, otherwise opens it — no create/open/remove list (multi-server on one install is cloud-only later). **Client/Server role open (do not regress):** open books on a worker (`client_mode_job` / `server_*_job`); DeviceSetup shows **Opening… / Preparing desk…** + Cancel the whole time — including after the worker returns, while the UI drops the previous SQLite (off-thread) and runs `reload_all` / `sync_chrome`. Never clear `role_open_busy` in `role_open_finish` on success (that left a silent multi-minute freeze). Gate `sync_chrome` with `role_open_busy` like other heavy jobs. **Owner wipe:** Settings → Backup → Delete local data (business records / shop setup / screen preferences + Owner PIN), same categories as Windows uninstall / `delete-books-data.ps1`. **Emergency unlock PIN (offline LAN only — do not regress):** Team → optional duress PIN; unlock path calls `try_duress_unlock` → `wipe_trading_data_keep_structure` — books entries gone, catalogue/schema/staff/settings kept, opens as Owner looking normal (vision §17). Scope is cloudless Server/Client shops, not cloud multi-shop. Never conflate with Backup “shop setup” wipe. **Person rights ≠ DeviceRole (do not regress):** what chrome a person sees is `NavCaps` / Team rights — never strip Owner/Waiter (etc.) features because the till is Client Mode. Client/Server only changes *how* the action runs (LAN write / Server kitchen printers vs local). Khaynjo KOT “Print selected again” → Client `POST /kot_reprint`; Server prints. After staff sync/`reload_all`, refresh `session` features/rights from the roster. **Khaynjo:** after a successful first-server create (and before Owner PIN), a one-shot **Rajdhani App Template** card may appear (Apply = menu import with status + Cancel; Skip → Lock). See [`DESIGN.md`](DESIGN.md) § Launch / auth.

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

**Launcher / logo:** Master art is `apps/moklanjo/assets/moklanjo-app-icon.png`. Run `./apps/moklanjo/scripts/generate-icons.sh` (Khaynjo: `./apps/khaynjo/scripts/generate-icons.sh`) to rebuild Android mipmaps, desktop `icon_*.png` / `icon.ico`, and in-app **`kumkaj-mark.webp`** (WebP only — no PNG/AVIF brand marks). Nav icons are WebP under `resources/nav/`. Android launcher mipmaps stay PNG (platform requirement). Desktop `icon_*.png` / `.ico` stay on disk for MSI/window chrome but are **not** packed into APK assets (`prune_android_runtime_assets` in cargo-makepad via `apply-android-activity-bridge.sh`). Reinstall Android after regenerating.

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

**Install:** run `install.sh` (Linux) or double-click **`Setup.bat`** (Windows) from the package folder. On Windows, creates **Start Menu**, **Desktop**, and best-effort **taskbar** shortcuts, then **starts the app**; Start Menu also includes **Uninstall …**. Do not open `install.ps1` in Explorer (that often opens Notepad) — use `Setup.bat`.

**Uninstall:** from the menu, Settings → Apps (Windows), double-click `Uninstall.bat`, or re-run `uninstall.sh` / `uninstall.ps1`. Always prompts:

- **Keep data** (default) — books stay; run uninstall again later to delete them  
- **Remove data** — deletes the app-id data directory  

If app files are already gone but data remains, re-running uninstall still offers to remove leftover data.

**Explicit env wrappers** (no flag to remember — use these from the repo):

| | Development | Production |
|---|---|---|
| Linux install | `./apps/moklanjo/scripts/install-linux-dev.sh` | `./apps/moklanjo/scripts/install-linux.sh` |
| Linux install (Khaynjo) | `./apps/khaynjo/scripts/install-linux-dev.sh` | `./apps/khaynjo/scripts/install-linux.sh` |
| Linux uninstall | `./apps/moklanjo/scripts/uninstall-linux-dev.sh` | `./apps/moklanjo/scripts/uninstall-linux-prod.sh` |
| Windows (Moklanjo) | `apps\moklanjo\scripts\uninstall-windows-dev.ps1` | `apps\moklanjo\scripts\uninstall-windows-prod.ps1` |
| Windows (Khaynjo) | `apps\khaynjo\scripts\uninstall-windows.ps1 -Dev` | `apps\khaynjo\scripts\uninstall-windows.ps1 -Prod` |
| Android | `./apps/moklanjo/scripts/uninstall-android-dev.sh` | `./apps/moklanjo/scripts/uninstall-android-prod.sh` |
| Android (Khaynjo) | `./apps/khaynjo/scripts/uninstall-android-dev.sh` | `./apps/khaynjo/scripts/uninstall-android-prod.sh` |

(Linux install wrappers call `install-linux.sh --dev`. That rebuilds the dev package on every run, then installs it, so a leftover `dist/` folder cannot leave an old binary in place. Linux/Windows uninstall wrappers call the shared uninstaller with `--dev` / `-Dev` or `--prod` / `-Prod`. Android uses `adb uninstall` of that package id.)

Each app's installers are its own (`apps/<app>/scripts/`: `install-*.{sh,ps1}`, `uninstall-*`, `Setup.bat`, `Uninstall.bat`, `desktop-ids.sh`, `License.rtf`) — they name that app's exe, folders and labels. `scripts/stage-desktop-package.sh` copies the calling app's set into a package. (Khaynjo's Windows folder package used to stage Moklanjo's, which looked for `kumkaj-moklanjo.exe`; its MSI licence page said "Moklanjo" — fixed 2026-10-26.)

## Windows (cross from Arch / native on Windows)

**Decision (product):** Microsoft Store distribution is the **real Windows goal**.  
**Now:** per-user **MSI** (`./apps/moklanjo/scripts/msi.sh` → `dist/Moklanjo-<ver>.msi`, dev: `./apps/moklanjo/scripts/msi.sh dev`; Khaynjo: `./apps/khaynjo/scripts/msi.sh`) — single file, double-click install, **Start Menu + Desktop** shortcuts, best-effort **taskbar** pin, **launches the app** when setup finishes (skipped for silent `/qn`), uninstall from Settings → Apps. Installs to `%LOCALAPPDATA%\Programs\Moklanjo\`; **no admin**; fresh versions auto-replace old (MajorUpgrade). Uninstall keeps local data by default (`%APPDATA%\com.moklanjo\`) and asks separately whether to delete **business records**, **shop setup** (schema/databases), and **screen preferences**; Start Menu **Delete books data** offers the same categories. (Do not put prompts on the *install* path — that broke real installs earlier.) Folder + `Setup.bat` package remains the fallback.  
**Later:** wrap as **MSIX** for Store publish.

One **x86_64** PE binary covers **Windows 10 and Windows 11**. Dev and prod use the same env split as Linux/Android (`com.moklanjo.debug` vs `com.moklanjo`).

Makepad loads fonts from **`<crate>/resources/`** next to the `.exe` (crate name uses underscores: `kumkaj_moklanjo`, `kumkaj_khaynjo`). Hyphenated `kumkaj-*/resources/` will not load UI fonts.

### WorldFonts / packaged icon fonts (all desktop packages)

Floor tile actions, header eye, chevrons, and other glyph buttons use **Font Awesome** via `WorldFonts` → `icons` (`fa-solid-900.ttf`). Sidebar nav pictures are **WebP** under `resources/nav/` — those can look fine while FA buttons show empty rectangles.

| Rule | Detail |
|---|---|
| **Load path** | `WorldFonts` **must** use `self:resources/fonts/…` for icons, CJK (LXGW), and emoji — **never** `makepad_widgets:resources/…` for packaged desktop. Dev-from-source can see the widgets crate; MSI / folder / Linux packages do **not** ship a second `makepad_widgets/resources/` tree. |
| **Must ship** | At least `NotoSans-Regular.ttf`, **`fa-solid-900.ttf`**, plus the other Noto/LXGW/emoji files the app's `WorldFonts` lists. Staging: `makepad_copy_desktop_resources` in `scripts/windows-env.sh` (copies FA from vendor if missing; **errors** if FA still absent). Populate sources with `./scripts/fetch-ui-fonts.sh apps/<app>`. |
| **Symptom of a miss** | Windows/Linux install: sidebar icons OK, Floor/header action circles show blank boxes / tofu. Fix: re-fetch fonts, re-package, reinstall — not an OS theme issue. |
| **Install guards** | Each app's `install-linux.sh` / `install-windows.ps1` refuse packages missing `NotoSans-Regular.ttf` **or** `fa-solid-900.ttf`. |

Same rules for **Moklanjo and Khaynjo**, Windows **and** Linux. Android APK still packs the app's `resources/`; keep FA under the app fonts tree so all platforms share one path.

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
- **Print / share** sends through this device's printer pool when a printable
  route exists (Settings → Printers): Network `:9100`, Bluetooth SPP (`bt:`),
  USB host (`usb:`), or Standard via Android PrintManager (`android-print:` /
  blank target). Otherwise it opens an `ACTION_SEND` chooser (`shareText`).
  Either way it writes `receipts/receipt-YYYYMMDD-HHMMSS.txt` under the app
  files dir. Desktop also uses system queues (`lp` / `Out-Printer`) — see
  LAN § Printing. Bridge: `MakepadActivity-print.java` via
  `scripts/apply-android-activity-bridge.sh` + patch `android_print`.
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

The dish photo shelf (`assets/items`, ~130 MB) is not packed into the APK
(size). **Desktop packages (Windows MSI / portable, Linux) must ship both
`assets/items` and `assets/templates`** beside the app — Menu → **Import
complete template** and **Client menu** (e.g. Rajdhani) disappear if those
folders are missing. Staging: `copy_app_menu_assets_if_present` in
`scripts/windows-env.sh` (called from Khaynjo `windows-dev.sh` /
`package-linux*.sh`); MSI harvests `assets/` via `scripts/build-msi.sh`.
**Shelf** / **named client** imports run on the Server and copy each photo
into `item_images` as `custom:…`; Clients (including Android) pull missing
photos over LAN (`GET /item-image`) into a local cache. Initials show only
until that fetch lands, or when the Server itself has no file for the key.
How to add a client template: `apps/khaynjo/assets/templates/README.md`.

`arboard` and `rfd` are desktop-only dependencies — neither compiles for
Android, and `arboard` being unconditional used to break the Android build
outright.

## First-run unlock

A new server seeds an **Owner** whose PIN cannot be used: the first start
in Server Mode asks the Owner to choose a PIN (`complete_owner_setup`,
which refuses `0000`). Packages and guides must not mention a default PIN.

PINs are local SHA-256 hashes (Argon2 froze the UI on unlock; local-device PIN
threat model). Migrate does **not** rewrite staff PIN hashes and does not force
`0000`. Shops that still carry `meta.pin_reset_required = 1` from an older
reset keep the Lock-screen gate until the Owner picks a new PIN
(`Store::complete_pin_reset`, which rejects `0000`). The login-id/password
route is blocked while that flag is set. Fresh installs never set the flag:
they already have first-run Owner PIN setup.

Five consecutive wrong PINs block unlock for 30s with a countdown in the status
line. The counter and deadline live on `AppState` (in memory) — a restart clears
them, which is fine when the attacker already has the device and the real
protection is physical.

Lock screen: **Owner** | **Staff** (pick staff, then PIN), or optional **login id + password**.

Chrome peers are a long strip (phone bottom nav / desktop rail), not a More
bucket — Day book, Booking/Order, Floor/Kitchen (Khaynjo), Sales, Settings,
Lock, and the rest. Feature flags and person rights hide peers the signed-in
person cannot use. Unlock lands on the **first permitted peer**
(`unlocked_home_from`), not a hard-coded Day book.

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
`X-Kumkaj-Session` (from `/login`), `X-Kumkaj-Request` (one id per write),
`X-Kumkaj-Protocol` (both directions, see below).

**Realtime channel (`LAN_PROTOCOL` ≥ 2).** Beside HTTP, paired Clients open
a **WebSocket** on the same TCP port: `GET /ws` with the usual Device /
Protocol headers (Session optional until outbox). The Server pushes books
updates the moment `publish_books` would wake `/wait`. HTTP stays for
`/pair`, `/login`, `/hello`, `/health`, bootstrap `/sync`, photos, guest
pages, and fallback when WS cannot upgrade.

| WS frame `t` | Direction | Meaning |
|---|---|---|
| `welcome` | S→C | `{seq, rev}` — Server tip after subscribe |
| `delta` | S→C | `{seq, prev_seq, rev, full, …domains}` — apply in order |
| `need_sync` | S→C | gap or unknown Client tip — Client must `GET /sync` then resume |
| `ping` / `pong` | either | heartbeat (~15s); missed pong → reconnect |
| `outbox` | C→S | `{id, path, body, request_id}` — durable queue item over WS |
| `outbox_reply` | S→C | `{id, ok, body\|error, code?}` — same semantics as HTTP write |

**An answer on the socket is said on the screen**, exactly like an HTTP
write's: `phone::answer_from_socket` (each app's own copy) files the row
*and* pushes the job's `Done` — the Server's words, a refusal's reason,
Khaynjo's `print_problem`. Until 2026-11-07 it only filed the row, so a
device on a live link never heard "not accepted", "not printed", or got a
payment's Undo. A resend's second reply finds no row and says nothing.

**Rev + seq.** `rev` remains the snapshot content hash (304 / If-None-Match).
`seq` is a monotonic per-Server counter bumped on every published change.
Client applies a `delta` only when `prev_seq == local_seq`; otherwise
`need_sync` / HTTP `/sync`, then set `local_seq` from welcome/delta tip.
`full: true` replaces all domains; `full: false` replaces only present
domain fields (tables, KOTs, running orders, catalog, …).

**Fallback.** `/wait` + periodic `/sync` still work (older peers or failed
upgrade). Do not remove them when changing the live path.

**Do not regress.** Sole writer, idempotent `X-Kumkaj-Request`, durable
`lan_outbox`, apply off the UI thread, no shared Kumkaj sync crate — copy
per app (`lan_ws.rs` / `net.rs` / `phone.rs`).

| Status / `code` | Meaning | What the device does |
|---|---|---|
| 200 | done | queue item leaves the queue |
| 400 / 403 / 404 | refused for good, in the Server's words | queued write → "not accepted" list |
| 401 `pair_again` / `sign_in_again` | the device or sign-in is gone | queue stops, header says so |
| 426 `update_app` | the two ends speak different `LAN_PROTOCOL`s | queue **kept**, header says "Update the app…" |
| 429 `wait` | too many wrong codes / PINs | — |
| 500 | the Server panicked doing it (a bug) | refused once — never a hang-up that blocks the queue |
| 503 `retry` | "not now": books busy past the wait, disk full or failing | queue **kept**, sent again (like a lost reply); no reply recorded |

| Path | Who | Purpose |
|---|---|---|
| `GET /health`, `GET /hello` | anyone | liveness; app + venue + `server_id` (Client pair checks `/hello` against the Find row before posting the code) |
| `GET /track/{awb}` (Moklanjo) | anyone | public parcel status — no pairing |
| `GET/POST /book` (Moklanjo) | anyone on the office Wi‑Fi | corporate self-booking: priced by the chosen rate's slab and **billed to the named company's account** (never cash), audited `lan.self_book` |
| `GET /display` `/tokens` `/expo` `/kiosk` `/t/{table}` `/feedback` (Khaynjo) | anyone on the shop Wi‑Fi | guest / second-screen pages; table QR and kiosk add only dishes the guest menu offers, at most 50 per request, as pending lines staff see before Send KOT |
| `POST /pair` | anyone (rate-limited) | 6-digit pairing code (normalized, leading zeros) → device token |
| `GET /sync` | paired device | snapshot of the books — **no PIN or password hashes, no pairing code** |
| `GET /photo?kind=item\|party&file=<id>.webp` (Moklanjo) · `GET /item-image?key=…` (Khaynjo) | paired device | one photo the snapshot names only by key (`lan_images` fetches and caches it on the device) — Moklanjo takes a **bare file name only**, never a path |
| `POST /login` | paired device | staff PIN checked **on the server** → session |
| `POST /sale` `/settle` `/undo` `/void` `/refund` `/amend` | signed-in device | money writes, done as the signed-in person |
| `POST /kot` `/running` `/kot_advance` `/table_billing` (Khaynjo) | signed-in device | kitchen and table actions |
| `POST /staff_pin` `/staff_password` | signed-in device | change own credentials on the Server's books |
| `POST /books` | signed-in device | person-level shop writes (`BooksOp` — catalogue, staff hire, day close, stock, parties, settings meta, …); Client asks, Server applies |

**Person vs device (do not regress)**

```text
Person (job + rights + stations)  →  what is visible / usable
DeviceRole Server / Client        →  how this machine joins the shop LAN
Host-only OS concerns             →  firewall, pairing, local printers, …
```

**Client/Server is not a permission system.** From the owner’s point of view the
difference is almost only nature of the device:

| | Server | Client |
|---|---|---|
| Books | Listens; sole SQLite writer | Connects; sync cache |
| Printing | Direct / local printers | Uses the Server’s printers over LAN |
| Pairing / extra tills / firewall | Host controls | Join / forget *this* till |

Do **not** hide Inventory, Branches, Team hire, Settings shop rows, day close,
KOT reprint, or other person-authorized work just because `DeviceRole::Client`.
Do **not** snack “only on the shop device” for work the signed-in person’s
rights already allow — send it with `books_op` / the matching LAN route instead.

**Person rights are the real gate.** Job presets, screen chips, money/floor
rights (`StaffRights` / `check_right`), and (Khaynjo) kitchen station access
(`StaffFeatures.kot_stations`) decide visible vs hidden and allowed vs denied.
Chrome and hubs follow the signed-in **person** (`NavCaps` / rights) on Server
and Client alike.

Person-level books writes go through `AppState::books_op` → on Server,
`Store::apply_books_op`; on Client, `lan_job("/books", BooksOp, …)`. Sell/floor
paths stay on their existing routes (`/sale`, `/kot`, `/running`, `/kot_reprint`, …).
Own PIN / password stay on `/staff_pin` / `/staff_password`.

**Host-only exceptions** (`require_server_host` — this *machine* is the Server,
or this Client’s own pairing link): allow/stop extra tills, pairing code,
forget devices on this Server, firewall, autostart, choose backup folder path,
restore a backup from a local file path, local printer discovery. A Client’s
Find server / Pair / Forget *this* till’s link stays on that Client.

**Rules that keep it safe to run all day**

- **Server is the sole writer.** A device's tables are a sync cache the next
  snapshot overwrites; `Store::assert_server_writer` refuses local writes on a
  client, so no UI path can bypass it. Clients send `BooksOp` / sale / KOT
  writes to the Server instead. The `*_from_sync` writers skip the check —
  that is the server's own data arriving.
- **Each write once, and whole.** A device's write runs as **one
  transaction** (`BEGIN IMMEDIATE`, `Store::begin`), the reply record per
  `X-Kumkaj-Request` included: a retry after a lost reply — or after a
  crash between the write and its record — gets the same answer, never a
  second sale or KOT; a refused write leaves nothing half done (its
  savepoint rolls back); two devices writing to one table or the last unit
  of stock take turns. Store functions open their own unit with
  `Store::begin` / `in_transaction`, which **nests** as a savepoint — never
  `conn.unchecked_transaction()` (a nested `BEGIN` fails). Compound writes
  (`write_sale`, Khaynjo `send_kot`, Moklanjo AWB allocation) make their
  checks inside their unit, never before it. KOTs and bills for a device's
  write print only after it committed. Tests:
  `eight_devices_over_real_sockets_lose_and_double_nothing`,
  `busy_books_are_a_retry_never_a_refusal`.
- **Reads never take the write lock.** The server opens the books per
  request; `Store::open_path` migrates a file once per process and schema
  (`PRAGMA user_version` + an in-process set) — migrating each time took
  the write lock even when nothing changed. A read path that "heals" state
  (Khaynjo `list_tables` freeing paid tables) looks first and writes only
  when needed, best effort. Tests: `opening_migrated_books_takes_no_write_lock`,
  `a_sync_never_waits_on_a_writer`.
- **Versions say so.** `net::LAN_PROTOCOL` goes on every request and reply
  (`/hello` too). Bump it with any route/body/snapshot change an older
  build would misread: a mismatch is a 426 "update the app on this device /
  on the server" instead of a snapshot that fails to parse and looks like
  "offline" for ever. A request without the header is treated as the
  current protocol (devices from before it). Test
  `another_protocol_is_told_to_update_the_app`.
- **Rights and PINs are judged on the server.** A manager's approval PIN is
  sent as typed; `check_right` (per-person rights, Team → Rights) decides.
  The approver is recorded and the reason names both people.
- **Brute force waits.** 5 wrong pairing codes or PINs from one address make
  it wait 5 minutes (`Guard`); the owner can issue a **New pairing code** and
  **Forget** a device (Settings → paired devices) — it is refused at once.
- **Bind before advertise.** `ensure_server` binds the TCP port **first**,
  then starts UDP discovery / books watch. A busy port returns an error
  (close the other shop app, Allow again) — never “Extra tills on” while
  another process still owns the port. Switching the open shop’s books
  path stops the old listener and rebinds this one (`SERVER_DB_PATH`).
- **Pair the shop you picked.** `client_pair` GETs `/hello` first; when
  Find filled the host, the row’s `server_id` must match or pair refuses
  with `wrong_shop` (right code must not succeed against the wrong books
  on a stolen port). Codes go through `normalize_pair_code` (digits,
  zero-pad to 6) on both ends.
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
- Live path: WebSocket `/ws` pushes deltas (`LAN_PROTOCOL` ≥ 2). Safety net:
  `/wait` long-poll + a periodic `/sync` (default meta `sync_interval_secs`
  20s) if the socket drops. Khaynjo Floor / Kitchen / Order also refresh from
  the local cache on a short UI timer — they must not re-pull the whole book
  on the UI thread. Real-AP multi-tablet soak:
  [`PRODUCTION_STATE_FOR_OFFLINE_USAGE.md`](PRODUCTION_STATE_FOR_OFFLINE_USAGE.md)
  §3 *LAN realtime pilot checklist*.

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

**Printing (`print_share.rs`, per shop on this device).** Each local Server
(and the Client cache) keeps its own `printers.json` beside its books
(`servers/<id>/printers.json`, `client/printers.json`) — never a shared
install-root file that mixes shops. Legacy install-root `printers.json` is
copied once into a shop that has none. Settings → **Printers** edits the
open shop's pool + routes. Empty route = that feature does not print.
Khaynjo places: **Bill**, plus **KOT per kitchen station**. KOT lookup:
station’s own route (empty = do not print that station — never fall through
to Bill). Moklanjo places: **Receipt** and **AWB label**. Legacy files
migrate on load; blank fields stay empty (no invented “Default” printer).

**Printing nouns (see [`docs/PRINTER.md`](PRINTER.md)):**
- **Real printer** — OS device from Find (destination only).
- **Printer** — an OS device from Find. Find writes the list to the shop's
  `printers.json` and the running app. The OS name is the name. There is no
  virtual printer and no manual rename.
- **Bill & KOT format** — content width (shared), plus portrait/landscape and
  delivery (OS document, raw bytes, ESC/POS) separately for the bill and the
  kitchen ticket. Either job can be sent to any found printer.
- **Content width** — above the Bill and KOT tabs. One millimetre width lays
  out both. It is **not** locked to only 58/80.
- **Bill & KOT format** — slip content: field show/order/copies; **per field**
  font weight, size, and **Start / Center / End** (shop, time, staff, station,
  table, kot no, area, item/qty, title); **per KOT kind** original / duplicate /
  modified / cancelled (each kind has its own field styles + banners).
- **Start / Center / End** are drawn across the selected content width, on
  OS document and ESC/POS. Do not strip the style marks and then left-align
  the plain text — that made the chips do nothing on the printed copy.
- **Qty** is the count in the right column (`1`, `1.25 kg`). Do not prefix or
  suffix the dish with `x1` / `x1×`. `format_qty_display` uses `x2` for bills;
  a kitchen ticket must strip that before the name is printed.
- Settings → **Printers**: Find, then the OS names. No virtual printer form.
- Place attach + First/All: Kitchen stations / Bill. Station groups: discarded.
- Multi-copy KOT: one queued job; worker packs N identical ESC/POS slips into
  **one** payload (or CUPS `-n` / Windows Copies for document). Do **not** open
  N TCP/`lp` sessions and do **not** sleep between accepts — that made copy
  2/3 feel delayed after the first.

**Find printers** publishes after system queues (`lpstat`), then again
after Wi‑Fi `:9100` / BT / USB — UI merges into the pool on each Signal
(do not wait for the full /24 scan or a scroll to show names).

| Target | Document format | Thermal-style format |
|---|---|---|
| Network / BT / USB | ESC/POS raster | ESC/POS raster |
| Desktop system name / blank | OS document (`lp` / Out-Printer) | Raw ESC/POS spool |
| Android blank / `android-print:` | PrintManager PDF UI | refused — use Network/BT/USB |

**Sending a slip (both apps, 2026-11-07).** A bill and a kitchen ticket are
the same kind of slip; the route's format picks delivery (`document`: page
image; else ESC/POS) and the same encoder and width serve both. Rules the
senders keep:
- **Bands under 3 KB, one picture at a time.** `GS v 0` bands carry under
  3 KB of picture (`band_rows`), and every writer — TCP, Linux/macOS USB and
  Bluetooth files, Windows `WritePrinter` (named queue and USB), Windows
  Bluetooth serial, Android SPP and USB host — sends one picture per write
  with `PIECE_PAUSE_MS` between, and a pause before the cut. A whole ticket
  in one burst printed about three lines (the first 128-dot band). Do not
  "fix" a wide slip by clamping to 80 mm or switching it to a page image.
- **The split keeps every byte.** Bytes between two pictures (a copy's feed
  and cut, the next copy's reset) are their own piece. Dropping them ran
  copies together uncut.
- **Network: wait, then confirm.** Connect retries for ~15 s while the
  printer refuses or resets (busy with the last ticket). After the job,
  DLE EOT 1: the printer answers once it has read everything; an
  "offline" answer (paper, cover, error) is a failure. A printer that never
  answers is remembered and later jobs half-close and drain instead.
- **CUPS.** A queue whose device is `socket://host:port` is sent directly
  (paced and confirmed). Other `lp` jobs are watched for ~3 s: done → printed;
  "waiting for printer" / not connected → cancelled and reported.
- **Android Bluetooth** waits for the stack to send before `close()`, which
  otherwise drops what is still queued. The bridge lives in
  `patches/makepad/MakepadActivity-print.java`;
  `scripts/apply-android-activity-bridge.sh` swaps an older copy for it.
- **One job never rewrites the shared setup.** Test bill, Test KOT and
  `Store::print_kot` adjust a copy and call `print_route_using`; a getter
  (`kot_slip_format`) that migrates the file writes the file only
  (`write_device_printers`). Swapping the global in and out raced the LAN
  thread and Settings saves, and the first ticket after the migration printed
  nowhere.
- **Marks are for the printer.** Bill / KOT text carries a per-line style mark
  (U+E000 + size/weight/align). Clipboard, share sheet, WhatsApp, the saved
  `.txt`, the ticket status line and `ReportText` strip it
  (`strip_kot_style_marks`); only print jobs keep it.

Android-as-Server is first-class: Network, BT, USB, and document PrintManager.
Failures via `take_problems`. Client devices print at the Server.

**Backups.** The server writes a backup automatically once a day
(`auto_backup_if_due`; the last 14 are kept in the data folder's `backups/`)
and Settings → **Also copy backups to a folder…** keeps a second copy, e.g. on a USB drive.
A restore goes **into the live books through SQLite's backup API**
(`restore_from_to_path`, one step, one transaction) — never by replacing
the file: the LAN watcher and devices' requests keep connections open, a
rename over an open file fails on Windows, and on Linux those connections
kept serving the old books. Test `restore_reaches_connections_already_open`.

**Discovery.** "Find the shop's server" broadcasts on the UDP port and falls
back to scanning the /24; replies list venue + `server_id` + host so staff
pick the right shop. Tapping Use fills the host; **Pair now** re-checks
`/hello` against that `server_id`. Same Wi‑Fi trust model as before: no TLS.
Do not start discovery until TCP bind succeeded — otherwise Clients find the
real venue name, type the correct pairing code, and get “Wrong pairing code”
from whatever still holds the TCP port.

## Branches offline = separate installs (vision §5); not multi-server on one device

A branch is its own business. **Offline cloudless LAN:** each outlet runs its
own app install (or OS user profile) with **one** local Server books file —
DeviceSetup never offers a second local server. One set of books never holds
two branches. Apps may still carry a `branch_id` column and a single
built-in branch row (legacy shape; the books' own code/name used on
invoice numbers) — new code must not add in-book branch switching.
**Multi-server registry UI on one install is cloud-only (later), not offline.**

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
# Prefer the isolated harness (never :0 / never your Wayland session):
./scripts/run-ui-smoke-isolated.sh khaynjo   # or moklanjo / all
# Manual equivalent:
Xvfb :99 -screen 0 1280x900x24 &          # a private display, not the desktop
env -u WAYLAND_DISPLAY DISPLAY=:99 KUMKAJ_DATA_DIR=/tmp/kd KUMKAJ_PRINT_SINK=off \
    ./target/debug/kumkaj-khaynjo &        # X11 backend on the virtual display
DISPLAY=:99 xdotool mousemove X Y click 1  # drive it
DISPLAY=:99 import -window root shot.png   # and look
```

**Books journeys (no GUI):** `./scripts/run-user-journeys.sh` runs each app's
`tests/user_journeys.rs` — full dine-in / booking / COD / khata / day-book
paths through the same `Store` API the UI calls, with the OS printer fenced.
Also `cargo khaynjo-journeys` / `cargo moklanjo-journeys` (set
`KUMKAJ_PRINT_SINK=capture` yourself if not using the script).

**UI smoke hooks (dev only, ignored when `KUMKAJ_ENV=production`):**

- `KUMKAJ_SMOKE_AUTO_SERVER=1` — tap Server once on DeviceSetup
- `KUMKAJ_SMOKE_AUTO_UNLOCK=<pin>` — fill Owner PIN once on Lock (seed uses **1234**)
- `KUMKAJ_SMOKE_SCRIPT=peers|full|soak` — in-app scripted day (real `go_peer` /
  Settings panels / books actions), writes `need_shot` for PNGs under
  `KUMKAJ_SMOKE_OUT`, then exits with `PASS` / `FAIL`
- `KUMKAJ_SMOKE_SOAK_SECS` — after a full pass, keep peer-cycling (default 1800
  when mode is `soak`); use `86400` for a day, larger for multi-day soaks on a
  spare machine / VM with Xvfb

```bash
./scripts/run-ui-smoke-isolated.sh              # both apps, full script
SMOKE_SCRIPT=peers ./scripts/run-ui-smoke-isolated.sh khaynjo
SMOKE_SCRIPT=soak KUMKAJ_SMOKE_SOAK_SECS=86400 ./scripts/run-ui-smoke-isolated.sh moklanjo
```

Books journeys stay the fast money oracle (`./scripts/run-user-journeys.sh`).
UI smoke is the slower “what a person sees and taps” layer.

Seed realistic data first (each app's own `tests/visual_seed.rs`, opt-in) so
every state is on screen, not just the empty one. Both apps accept
`SEED_DATA_DIR=<empty folder>` (creates the local Server, Owner PIN **1234**).
Khaynjo also accepts `SEED_DB=<…/khaynjo.sqlite3>`. Moklanjo seeds cashier
Asha 2222, bookings in every desk state, logos — start with
`KUMKAJ_DATA_DIR=<same folder>`, pick Server, unlock.

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
    The patch gives `draw_caption` **no style of its own**: set the whole
    style, `draw_caption +: { color: … text_style: theme.font_regular{font_size: 10.5} }`.
    `text_style +: {font_size: …}` only patches the size onto a style with no
    font, so the caption draws nothing and a filled field shows a blank band
    above its value (Moklanjo until 2026-11-02; Khaynjo had it right). Fields
    that sit beside their own label with a "0" placeholder (cart / totals)
    turn the caption off; a placeholder must name the field ("Price", never
    "0") or the caption reads "0".
12. **xdotool cannot type into Makepad fields under X input methods** —
    launch the app with `XMODIFIERS=@im=none` on the virtual display.
13. **A screen scrolls as one page** (two separately scrolling halves were
    rejected 2026-10-24). Two list shapes:
    - **Expand-to-content** (Khaynjo Order cart / Floor grid, Moklanjo sale
      lists): full content height, `page_owns_scroll`, outer `ScrollYView`
      owns wheel **and** mouse/touch press-drag (`mouse_capture_overload`).
      Do **not** use `page_window` here — variable open/closed row heights
      made it flicker.
    - **Capped nested viewport** (long Menu catalogue, Settings pickers):
      bounded box + thin scrollbar + *scroll chaining*; may use
      `page_window` so only on-screen rows draw.
    A picker that needs the whole height (Khaynjo's menu: search, category
    tabs, dish tiles) is a `Modal` window over the screen instead — the
    page behind it holds still (2026-10-25).
14. **Sideways-scrolling tab strips get ‹ › chevrons** at the ends, each
    shown only while there is more that way (`TAB_STRIPS`,
    `sync_tab_chevrons` in each app). The chevron is *drawn* by the
    `ChevronBtn` face's shader (`draw_bg.point` 1 = ›, −1 = ‹), never a
    "‹ ›" glyph: a glyph sits on the text baseline and read low in the strip.
    Chevrons **overlay** the strip (~46px): bottom-nav content end padding must
    be ≥ that width, and chevron clicks **snap to the next tab** (or step ≈½ the
    clear viewport) — a 70% viewport jump skipped tabs on phones (2026-11).
    **Phone `bottom_nav` floor (bug 7d):** X scrollbar is the bottommost chrome
    (`bottom_nav.padding.bottom` 0 / safe-area only); strip bottom pad ≥
    `bar_size` so chips are not under the overlaid bar; `lift_footer` writes
    bottom only; bar colour matches the footer (not stock cream).
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
    Alt+←/→, edge swipe, Android system back). **Model (every OS, both apps):**
    one browser-style visit trail — not “peers without Back.”
    - **`go_peer`** (bottom nav / side rail) **and** **`go`** (drill-down) both
      push `nav_back` and clear `nav_forward`. Peer jumps also clear in-screen
      layers (Settings section, Order sheets, …) so those cannot linger.
    - **Header ‹** (`header_back_visible` in each app’s `amount_visibility.rs`):
      show whenever Back can do something — `nav_back` non-empty **or** an
      in-screen overlay (`in_screen_nested_layer`) **or** nested setup. Hide
      only on the **visit root** (first unlocked screen after onboarding /
      unlock: empty stack, no overlay). Do **not** hide ‹ on Floor just
      because it is a chrome peer.
    - **Same action everywhere:** ‹, mouse Back, Alt+←, left-edge swipe, and
      Android Back all call `navigate_back` (overlays first, then pop).
      Mouse Forward / Alt+→ / right-edge swipe call `navigate_forward`.
    - Empty-stack fallbacks: SidebarSettings → Settings; Lock / ClientPair /
      Onboard → DeviceSetup. Paint via `paint_header_back_chrome` (also after
      Settings panel open and DeviceSetup Language accordion).
    - **Setup fence:** once Back opens **Server/Client** (`DeviceSetup`), drop
      Forward and never restore **Onboard** via Forward/‹ history. Leaving
      Onboard for DeviceSetup does not push Onboard onto Forward. Unlocking
      into the main till clears the visit stack (same as finishing onboard)
      so the books are a root. Onboard is only entered by the unlock /
      first-PIN path when setup is still incomplete — not by browsing back.
    **Do not** let a focused text field swallow mouse Back / Forward on
    desktop — blur then continue the visit step (Lock PIN was the first
    report; Settings / Order / Day book fields had the same swallow). Side
    mouse buttons always consume the event and share the XF86Back debounce
    so Linux does not step twice. Web also `preventDefault`s auxclick on
    buttons 3/4 so the browser does not leave the app. Android may still use
    the first Back to dismiss the soft keyboard outside auth/setup. System ▾ hide
    latches IME dismissed without clearing key focus (bug **6d**) — never
    unfocus inside DidHide on Android (that broke touch-scroll); retap after
    dismiss must still call `request_ime_show`. See bug **6e** before any
    IME change that touches focus, capture, or insets.
    Auth screens
    hide the unlocked header; Lock stays bare but mouse/Android Back still
    returns to DeviceSetup; ClientPair / Onboard show header ‹ when nested.
    Nested in-screen layers (Settings section → hub, suggest list,
    picking-server overlay, Khaynjo line-action / move / split / pick-table)
    dismiss on Back **before** leaving the screen — never skip to Day book;
    page `ScrollYView` press-drag works with mouse and touch
    (`scroll_bar_y.mouse_capture_overload: true` on both apps).     Nested expand-to-content
    PortalLists stay touch-fling only so the page owns mouse drag over
    carts. **Capped** SuggestList / StationCheckList / LanguageList /
    CurrencyList / ServerList / PairedList / DishPickList /
    CategoryPickList (Khaynjo) set `mouse_drag_scrolling: true` so laptop
    press-drag still scrolls when there is no wheel (Makepad
    `drag_scroll_touch_only` + opt-in). Khaynjo **ProductList** (menu modal
    Fill grid — no page ScrollY behind it) also opts in. On commit the list
    `promote_finger_capture_over` so the page `ScrollYView`
    (`mouse_capture_overload`) cannot steal the gesture
    (`portal_list_mouse_drag_promote`) — without that, Settings printer
    assign and similar capped checklists only moved the page. Both apps’
    page `ScrollYView` / `ScrollXView` (and Moklanjo `booking_desk_hscroll`)
    set `axis_lock: true`. Open dropdown faces use
    `Done · {label}` (`pick_done_dd`) — no second Done under the list. Horizontal tab strips (`ScrollXView`) mouse
    press-drag on whitespace around chips (nav drag-to-swap on a chip is
    not stolen); desk panes set `scroll_bar_x.mouse_capture_overload: true`
    (Makepad patch `drag_scroll_touch_only`).
    Selectable `WrapLabel` / TextFlow yields sideways drags to an ancestor
    ScrollX (commits text-select only when the drag is mostly vertical —
    `text_flow_selection`). On **touch**, vertical swipe always scrolls the
    page; text select-drag starts on long-press only (`text_flow_touch_scroll`,
    `text_input_touch_scroll`) — otherwise WrapLabels and focused fields
    (Lock PIN, ClientPair) stole every swipe. InfoHelp titles and text
    inside tap targets stay `selectable: false`; other display WrapLabels
    stay selectable. ClientPair lists found Servers
    with **Use** (`FoundServerList`); auto-fill host only when exactly one
    answers.
    Stuck Empty scroll-blocks clear on wheel (`scroll_bar_orphan_unblock`) and
    on touch when nothing is captured (`scroll_touch_orphan_unblock`).
    **Nested scroll views of different axes** (a sideways region in a
    downward page, the Order desk's columns, the app-wide `app_hscroll`)
    set `axis_lock: true` on their bars (Makepad patch `scroll_axis_lock`;
    Khaynjo's page/strip/desk views all do): a press-drag waits 8px, then
    goes whole to the bar whose axis dominates and that can scroll — never
    to whichever saw a 2px jitter first. Same flag: Shift+wheel scrolls
    sideways (vertical bars leave it). Nested
    scroll chaining: inner list keeps the wheel only while it can still move
    that way; at the edge the page/desk takes over (DEV §13). Every
    `ScrollYView` keeps **right padding 16** so content never sits flush on
    the vertical scrollbar; desk `ScrollX` keeps **bottom padding** clear of
    the horizontal bar (same idea as phone `bottom_nav` strip bottom ≥
    `bar_size` — bug **7d**: pad is *inside* the scroll content, never under
    the bar via the SolidView’s `padding.bottom`).
    **Floor Area whitespace (Khaynjo):** the gap under Columns and the pads
    around tiles must sit **inside** `area_scroll` (`FloorList` Item) with
    `mouse_capture_overload` + `axis_lock` — parent `spacing` between
    `floor_cols_box` and `FloorList` is not Area content (sideways pan dies
    there). `floor_area_body` uses `spacing: 0`; top pad on `area_scroll`
    carries the visual Columns↔tables gap (`floor_area_block_height` must
    match).
15b. **A `width: Fill` child of a scroll view is laid out from its declared
    walk**, deferred, so a width set later in its own `draw_walk` never
    reaches the parent's measurement: `ScrollXView` saw 0 content and the
    Floor would not scroll sideways (2026-11-02). A child that sizes itself
    at draw time declares `width: Fit` (`FloorList`).
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

Before any desktop package: `./scripts/fetch-ui-fonts.sh apps/<app>` so
`resources/fonts/` includes **`fa-solid-900.ttf`** (and Noto/LXGW/emoji).
`makepad_copy_desktop_resources` refuses to finish without FA — empty
Floor/header icon boxes on Windows mean that file never reached the package
(see § WorldFonts / packaged icon fonts).

```bash
./apps/moklanjo/scripts/package-linux.sh       # prod → dist/linux/ (+ install.sh / uninstall.sh)
./apps/moklanjo/scripts/package-linux-dev.sh   # debug → dist/linux-dev/
./apps/moklanjo/scripts/package-windows.sh     # prod → dist/moklanjo-windows/ (+ Setup.bat)
./apps/moklanjo/scripts/msi.sh [prod|dev]      # MSI → dist/Moklanjo[Dev]-<ver>.msi
cargo moklanjo-windows-dev              # debug → dist/moklanjo-windows-dev/
./apps/moklanjo/scripts/package-android.sh
./apps/moklanjo/scripts/package-web.sh         # moklanjo.com → dist/web/moklanjo/
KUMKAJ_SITE_DIR=web/kumkaj ./scripts/build-site.sh              # kumkaj.com → dist/web/kumkaj/
KUMKAJ_SITE_DIR=apps/khaynjo/web ./scripts/build-site.sh        # khaynjo.com → dist/web/khaynjo/
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
- **Staff** — pick a non-owner person (Previous / Next), then their PIN. Who /
  Team / suggests use one shape for everyone: `Name (role) · username` when a
  username is set, else `Name (role)` (`StaffMember::lock_label`) — Owner is not
  a special parenthesis-only format. Display **name** may repeat; **username**
  (`login_id`, stored lowercase) is the unique human handle among active staff
  when set — not a short UUID. Team type-or-pick writes `lock_label` (never bare
  name alone) and resolves by id → username → label → unique name — never
  first-substring among duplicates.
- Optional username + password (Team can set once; only the user can change password later)
- **Client:** PIN is checked on the Server (`POST /login`). Sync never carries PIN /
  password hashes — a paired Client must not fall through to local hash check
  (that always looked like a wrong staff PIN). Offline unlock uses the PIN
  remembered after a successful online sign-in on that device
- **Client staff roster is replaced, not merged.** `replace_staff_cache_from_sync`
  deletes every non-Owner staff row then upserts the Server's list (full
  snapshot and staff deltas). Upsert-only left people from a previous shop on
  Lock — same display name, wrong id → right PIN rejected. Stale `lan_pin:` /
  `lan_session:` meta for removed ids is pruned; re-pair clears `lock_staff_id`
- **Snackbar on Lock / ClientPair:** `set_status` only skips **Info** while there
  is no session. `snack::classify` treats `"wrong"` as Error so “Wrong PIN…” /
  “Wrong pairing code…” actually open the bar (they used to classify as Info and
  never show while locked)
- More → **Team**: Owner hires by **job** (preset) + optional screen/rights tweaks; copy-from-person; **every signed-in person** updates their own PIN (Settings → My PIN / Team peer — not Owner-only). Client tills send PIN changes to the Server (`/staff_pin`). Shop admin from a Client (hire, day close, catalogue, …) uses `POST /books` (`BooksOp`) — same person rights, Server writes.
- Sale **created_by** is set on finalize and never rewritten on amend

### Staff rights vs chrome (do not regress)

Observed 2026-11-04: Waiter chrome correctly hid Books / Buy / Day close /
Reports / Parties, but unlock still opened **Day book** with every hub row
and TODAY/BILLS money visible — rights looked broken even though taps said
“Not allowed”.

**One capability story everywhere.** `NavCaps` / `nav_tab_permitted` (and
`session_may_see_today` for money) are the source of truth. When you add a
screen, hub jump, or money strip, gate it the same way as the matching
chrome peer — never “nav only.”

| Surface | Rule | Code (per app) |
|---|---|---|
| Bottom nav / side rail | Only peers `nav_tab_permitted` allows | `paint_nav_tabs` ← `current_nav_chrome_order` |
| Unlock home | First permitted peer (**not** hard-coded Day book) | `unlocked_home_from` |
| Day book hub rows (Sell/Buy/Day close/Reports/Parties) | Same rights as those peers — hide the **slot** | `books_*_slot` visibility in paint/chrome sync |
| TODAY / BILLS / bank accordions + header today bar | Owner or Sales only | `session_may_see_today` → `sync_books_sections` / `today_bar` |
| Settings chrome | Any signed-in person (`can_open_settings_shell`) — shop rows need `can_access_settings`; Waiter sees **My PIN** only | Settings hub + Team |
| Team hire/edit | `can_manage_team` — others see only Change my PIN (works on Client via `/books`) | `team_admin_block` |
| Inventory / Branches | Person caps only — **not** `!is_client` | `nav_tab_permitted` |
| Firewall / pairing / extra tills / autostart | Server **host** + Owner | `is_shop && is_owner` chrome |
| Click handlers | Still refuse in the books layer; UI hide is not enough | `go_peer_if_allowed` / `check_right` / `books_op` |
| Kitchen station on Order / Send KOT / counter pay | Person’s `kot_stations` (None = all); store resolves menu kitchen when the line has none | Team checklist + `allows_kot_station` in `send_kot` / `send_kot_filtered` / `write_sale_unit` |
| Kitchen screen view / advance | Anyone with the Kitchen screen — **not** filtered by `kot_stations` (stations gate send/fire/reprint/pay only) | `KitchenList` / `advance_kot` |
| KOT print again | Person’s `kot_stations` for each line’s kitchen — **not** `Right::Reprint`; Server loads dishes from stored `kot_id` | `reprint_selected_kots` / `kot_reprint_payload` |
| Print bill (first) | Signed-in person may print once | Order / Floor print when not yet printed |
| Print bill again | `Right::Reprint` — **bills** only, when already printed | Team rights row |
| Team hire/edit store | Same as chrome: `can_manage_team` (Owner or Team chip) — not Owner-only | `add_staff_with_features` / `set_staff_features` / … |

**Team edit UX (do not regress — Khaynjo and Moklanjo):** Hire (job + optional
copy + Add + hire-only optional login) is one card; hire Job chips use
`hire_staff_role` / `hire_features` (not the edit draft). Screens, money
rights, (Khaynjo) kitchen-station checkboxes, rename, selected-person login
(“Apply login to {name}”), copy-onto, remove, and Save live only inside
`team_selected_setup`, visible after the owner picks who to edit
(`team_staff_id`). After Add, select the **hired** person’s id (never “last
non-owner”). Never show those controls with no selection, never fall back Save
to “first non-owner”. `Right::Reprint` = bill reprint; station access is the
checklist (`StaffFeatures.kot_stations`), not that right.

**Job-first hire:** pick Waiter / Counter / Kitchen / Manager (or copy a person
into the Add form). Tweaking screens / rights / stations is edit-after-pick, not
a floating form. Do not bury self-service (own PIN) behind Owner-only screens.

**Do not:** hard-code `Screen::Books` after unlock; leave hub `HubDestBtn`s
visible “because Books is the home screen”; show money figures to Settle-denied
waiters; gate only the rail and assume the page is fine; hide Settings entirely for staff who still need My PIN; demote capable people on Client Mode for shop books (use `/books` instead); conflate Client/Server with job rights; show Team rights/stations without a selected person.

**Smoke:** unlock as Waiter (Floor + Order + Kitchen, no Sales) → home is
Floor (or Order), not Day book; Day book must not appear in nav; if somehow
opened, hub must not show Expense / Day close / Bank & P&L / Parties or
TODAY money; Settings → **My PIN** works (Server or Client). Unlock as Owner → Day book with those hubs + full Team hire.

## Publish checklist (Play / web / desktop)

1. `cargo test -p kumkaj-moklanjo`
2. `./apps/moklanjo/scripts/package-linux.sh` → smoke `dist/linux/run.sh` / `./install.sh`
3. `./apps/moklanjo/scripts/package-windows.sh` (or `msi.sh`) → `dist/moklanjo-windows/` (+ `Setup.bat`); optional `cargo moklanjo-windows-dev` for Dev
4. Clone full Makepad `main`, set `MAKEPAD_ROOT`, run `./apps/moklanjo/scripts/package-android.sh`
5. Set `KUMKAJ_ANDROID_KEYSTORE` (+ `_PASS`) for step 4 — signed APK + AAB (Android § Release signing); create Play Console listing (title, screenshots from Stitch, privacy: local-only data)
6. Sites: kumkaj.com, moklanjo.com and khaynjo.com are Makepad 2 apps under `sites/` ([`WEBSITES.md`](WEBSITES.md); wasm via each `sites/<site>/scripts/run-web.sh`). Until the Phase 3 crawl shell serves `/privacy` and `/terms`, the legal pages still come from the legacy static stubs (needs `zola` on PATH): `./apps/moklanjo/scripts/package-web.sh` → `dist/web/moklanjo/`; `KUMKAJ_SITE_DIR=web/kumkaj ./scripts/build-site.sh`; `KUMKAJ_SITE_DIR=apps/khaynjo/web ./scripts/build-site.sh`. Play listing privacy URL: **https://moklanjo.com/privacy**.
7. iOS / macOS: scaffold only until Mac signing — `./scripts/package-apple-scaffold.sh apps/moklanjo`

Platform ids: see **Platform naming** above (`com.moklanjo` / `com.moklanjo.debug`). Product site: moklanjo.com; brand: kumkaj.com.

## Manual UI smoke (shipping)

1. First Server run: choose the Owner PIN; later runs unlock with it
2. Finish onboarding — Moklanjo is locked; Fresh or Book, then Done
3. Booking desk: weigh / rate / cash or account; confirm bill totals (Actual / Applied / Rounded)
4. Owner: Day book shows today; Pay out records carrier bills & expenses; Parties settle; Day close; Reports
4b. Waiter (no Sales): unlock → Floor/Order (not Day book); nav has no Books/Buy/Reports; Day book hub must not show Expense / Day close / Bank / Parties or TODAY money
5. More → Rates: zones/services, tax rate name or %, optional photo (initials or custom — no food-menu pack)
5b. Settings → Country & tax: apply `GB` (VAT inclusive, £), then `US` (sales tax exclusive, $); save a business id and check it prints
6. Ops / COD / Riders when fleet features are used
7. Branches (offline): two installs (or OS users) as Server; export a day snapshot from each, import on the owner's device, open the roll-up — do **not** expect Create new server on one DeviceSetup
8. Extra till: pair IP + code after the shop allows extra tills → book against the shop

### Amount visibility (screen eye vs per-figure)

Both apps (`apps/moklanjo/src/main.rs`, `apps/khaynjo/src/main.rs`) share the same
rules — duplicated per app, not a shared crate.

**Scope**

- **Global across screens** for the signed-in user: one screen-eye flag and the
  two per-figure lists apply on Books, Sell, Reports, etc. alike.
- **Per user** (`owner` or staff id, same keying as LAN / favorites): owner and
  each staff member keep their own prefs. Loaded on unlock via
  `reload_amount_visibility`.

**Screen icon vs per-figure eyes**

1. **Screen icon HIDDEN** → all crucial money **values** are masked, except
   figures whose keys are in `amount_keys_peek` (still-visible exceptions).
   Labels, headings, placeholders, counts, and prose stay visible — never
   bullet them.
2. **Screen icon VISIBLE** → each figure uses its **own** hide state in
   `amount_keys_hidden`. Screen eye does **not** force everything visible; it
   means “apply individual state.”

Encoded by `amount_key_visible(screen_hidden, keys_hidden, keys_peek, key)`:

- screen hidden → `true` only if `key` is in `keys_peek`
- screen visible → `true` unless `key` is in `keys_hidden`

**Persistence** (books meta, per `user_key`)

| Pref | Meta key |
|---|---|
| Screen eye | `ui_amounts_screen_hidden:{user}` |
| Hidden while screen showing | `ui_amount_keys_hidden:{user}` |
| Visible while screen hiding | `ui_amount_keys_peek:{user}` |

Legacy unkeyed meta names are read once as fallback, then overwritten under
the per-user key on the next toggle.

**What may be masked**

- Only money **values**. Call sites must not put labels or composite prose
  through `display_sensitive` / `mask_sensitive_text` / `display_money` /
  `mask_money_tokens`.
- `paint_amount_value` is the canonical path for discrete figures (books home
  slots, etc.): it paints the value widget via `display_money` and the eye
  glyph from `amount_key_visible`, leaving the label widget alone.
- Composite strips (`today_label`, credit/floor summaries) format words and
  counts in the clear and pass only money slots through `display_money`.
- `ReportText` (Day close, Reports, and every other report card): each Kv
  **money** value gets its own eye on the right (`kv:{label}` key, same
  screen/peek/hidden rules as Books). Labels stay clear. Line/Sub prose
  masks money tokens only while the **screen** eye is hiding (no per-token
  eye). Headings and digit-free lines stay readable. Report structure is
  never erased. Callers must pass **raw** rows (pre-masking destroyed the
  original figures so a per-row eye could not reveal them).

Toggles: `toggle_amounts_screen`, `toggle_amount_key` (both write the matching
per-user meta).

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

`COUNTRY_PROFILES` holds one `CountryProfile` per country: tax system and name, pricing mode (**`{tax} inclusive`** / **`{tax} exclusive`** — e.g. GST inclusive when the total already includes GST — `TaxPricing::InsidePrice` / `AddedAtTill`; meta still stores `inclusive` / `exclusive`), seed rates, optional receipt split (India CGST/SGST), instant-pay label, business-id label, credit wording, date order, and the number style (`decimal_sep`, `thousands_sep`). `XX` is the neutral custom pack and is the only one that leaves currency/language alone. Always pair inclusive/exclusive with the tax name (GST / VAT / …).

### Date / time appearance (not storage)

Books **store** every stamp as year-first **RFC3339**
(`2026-11-05T22:47:00+05:30`) — day filters, retention, LAN sync, and
triple timestamps (`user_at` / `device_at` / `server_at`) never depend on
how the shop likes clocks to look. See [`BUSINESS_BOOKS.md`](BUSINESS_BOOKS.md)
§ Triple timestamps.

**Appearance** is owner-facing only, on Settings → Country & tax:

| Meta key | Values | Notes |
|----------|--------|-------|
| `date_order` | `dmy` · `mdy` · `ymd` | Country pack seeds it; owner may change |
| `time_format` | `24h` · `12h` | Shop preference; **survives** `apply_country_pack` |

`FiscalSettings::{format_date, format_time, format_datetime, format_now}`
are the only formatters for bills, KOT/receipts, reports, backup labels,
LAN “last seen”, and print test pages. `BooksOp::SetDateTimeAppearance`
persists both keys. Never rewrite stored timestamps to match appearance.

### Number style

`decimal_sep` and `thousands_sep` ride on the profile and are persisted as the `meta` keys of the same name. `thousands_sep` is read verbatim rather than trimmed, because a space (France, Sweden, South Africa) and an empty string (no grouping) are both real choices; `decimal_sep` falls back to `.` when blank.

- `locale::format_money_ex(minor, currency, decimal_sep, thousands_sep)` does the formatting; `locale::format_money` is the `.`/`,` default. `Money::format_localized(currency, &fiscal)` is the call site wherever fiscal settings are in hand — receipts, day close, the Sell strip, lists.
- `locale::parse_major_input` accepts every local habit (`13,5`, `1.234,50`, `1 234,50`, `1'234.50`). With both marks present the last one is the decimal; a lone mark three digits from the end is grouping, unless the currency really carries three decimals.
- Edit fields use `locale::format_major_input_ex` (local decimal mark, never grouped).

### Optional service charge

`meta.service_charge_bps` (0 = off) adds `lines_total × bps / 10000` on top of every bill, after tax, and is frozen onto `sales.service_paise`. No country pack sets one — it is a house rule, so it also survives a country switch. `FiscalSettings::service_on` is the single place the arithmetic lives, and `Store::preview_sale_total` includes it so split pay still balances.

Finalize math lives in `fiscal::{tax_on_amount, net_of_tax, gross_with_tax}`:

| Mode (Settings label) | tax | sale total (charged) | taxable value (net) |
|---|---|---|---|
| **`{tax} inclusive`** | `line × bps / (10000 + bps)` | `draft.total` (unchanged) | `draft.total − tax` |
| **`{tax} exclusive`** | `line × bps / 10000` | `draft.total + tax` | `draft.total` |

Example (GST inclusive, 18% on a ₹3000 bill): taxable value = `3000 / 1.18`, tax = remainder, guest still pays ₹3000. Receipt shows Taxable value / GST (or pack label) / Total / “(GST inclusive)”.

The cart stays tax-agnostic under both modes — `SaleDraft::assert_balanced` only checks `lines_total + rounding = total`. `Store::finalize_sale` resolves the pack, freezes `tax_pricing`/`tax_label` onto the sale row, and uses the taxed total for the payment row, the credit ledger and day totals. `Store::preview_sale_total` gives the UI (and split pay) the same figure before charging.

The instant rail is still stored as `PayMethod::Upi` (`"upi"`) in every country; only `PayMethod::label_for` changes, so historical rows and sync payloads keep parsing. `SyncSnapshot` carries the Server's `fiscal` settings and `tax_rates` so Client terminals print the same wording.

**Not claimed anywhere:** statutory e-invoicing, fiscal-printer protocols, or tax filing. Seed rates are starting points a seller edits.

### Snackbar (standard feedback)

Both apps show a leveled snackbar above the bottom chrome (`app_snackbar`),
separate from the post-charge **Undo** snackbar. Same Makepad UI on **Linux,
Windows, macOS, and Android** (phone bottom nav and desktop rail).

| Kind | Typical use | Auto-hide (base = the user's time, default 10 s) |
|---|---|---|
| Info | Neutral status | base (Khaynjo: base) |
| Success | Saved / posted / copied | base (Khaynjo: base) |
| Warn | Read-only / retry / partial | Moklanjo 1.5 × base · Khaynjo base |
| Error | Failures / not allowed | Moklanjo 2 × base · Khaynjo base |

The base is per signed-in user (Settings → Alerts, 3–60 s, books meta
`ui_snack_secs:<user>`). The old fixed 4 s went before a busy counter could
read it. Moklanjo keeps warnings and errors lingering longer (capped at 60 s).

API (per app `main.rs`): `show_snack` / `show_error` / `show_warn` / `show_ok`,
`clear_snack`. `set_status` classifies the message (`snack::classify`) and shows
a snack when unlocked (errors/warnings also on lock). Higher priority replaces a
lower one; dismiss ✕ or timeout clears. Module: `apps/*/src/snack.rs`.

**Do not regress (Android scroll):** snack / undo snack labels must keep
`selectable: false` (tap-target exception — bug **6e** / `text_flow_touch_scroll`).
Error snacks are the usual trigger (long text + bar appearing under a finger).
Display text elsewhere stays selectable per [`DESIGN.md`](DESIGN.md).

### Diagnostics (privacy-first local logs)

Both apps ship a duplicated `diag` module (`apps/*/src/diag.rs`) — **not** a shared
crate. Ships on **every supported OS** (Linux / Windows / macOS / Android):

| Concern | How |
|---|---|
| Storage | `{data_dir}/diag/` via `Cx::get_data_dir` (Android `files/`) or `dirs::data_dir` (desktop) |
| Windows prod | No console — file JSONL is the log sink (dev builds still have a console) |
| Android | Init on `ensure_store`; export uses Save dialog + `content://` write like backups |
| Share | Single-file `.kumkaj-diag.json` bundle (folder export kept for desktop convenience) |

**Collect on device:** rotating JSONL + in-memory ring. Levels `error`…`trace`;
categories `system|nav|store|lan|print|ui|sync|pay`. PIN/password/token-like text
is always redacted; long digit runs and emails are scrubbed. Panic hook writes
`last_panic.txt` + an error event. Manifest records `os` / `arch`.

**Support session (Settings → Diagnostics):** 30 minutes of Debug-level capture
without changing the saved min level permanently.

**Send / share off device:** never automatic. Owner must tap **Approve share off
this device** (single-use, short TTL), then **Create share package**, then pick
where to save the bundle (Downloads / USB / Android document picker). Local
export does not need approval. Privacy note in the package: used only to
fix/improve the product; not sold or given to third parties.

**Controls:** Settings → Diagnostics (a hub section in both apps). Clear logs; uninstall UI wipe removes the `diag/` folder.

**Tests:** `cargo test -p kumkaj-khaynjo diag::` / `kumkaj-moklanjo diag::`.
