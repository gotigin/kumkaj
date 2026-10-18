# Kumkaj developer notes

Commands and paths below are from the **workspace root** (parent of `docs/`).

**Product focus:** [Kumkaj Courier](KUMKAJ_COURIER.md) — courier BOS (books +
booking desk). Package id: production `com.kumkaj.courier` · development
`com.kumkaj.courier.debug`.

## Live development (Cursor)

```bash
cargo courier-dev
```

Loads [`apps/courier/env/development.env`](../apps/courier/env/development.env) then `cargo run -p kumkaj-courier -- --hot --remote`.

- `--hot` — Splash/`script_mod!` hot reload on save
- `--remote` — Tweaker via **F12**
- Makepad **2 / `main` only** — never Makepad 1 docs/examples/APIs
- Local Makepad 2 tree: `vendor/makepad`
  - First time / after clean: `./scripts/setup-makepad.sh`
  - Path-dep from `apps/courier`; applies Wayland scroll-sign + dark caption button patches
  - Nested Settings lists claim wheel events so the page does not scroll with the list
  - Page `ScrollYView` keyboard: ↑/↓ step, PgUp → top, PgDn → bottom (`App::handle_scroll_key`). Skipped while a TextInput has key focus.
- Multi-script UI fonts live in **one** place: `apps/courier/resources/fonts/` (Noto scripts, FA icons, LXGW CJK, Noto Color Emoji). Desktop packages copy only `kumkaj-courier/resources/` — not a second `makepad_widgets/resources/` font tree. Re-fetch with `./scripts/fetch-ui-fonts.sh` if fonts are missing.
- **Desktop window size & maximize** — see § below.
- **Windows console** — production builds (`KUMKAJ_ENV=production` / `package-windows.sh`) use the GUI subsystem (no terminal). Dev builds keep a console for logs.
- **Min layout width 320 css px** — `app_shell` uses `Fill{min: 320.}` inside a horizontal scroll. The UI never lays out narrower than 320; if the device/window is below 320, content scrolls horizontally
- **Dev size** — `W×H` in the window caption bar, left of the Kumkaj title (terracotta on navy). Controlled by `KUMKAJ_SHOW_SIZE_BADGE` in the env file (not Cargo `--release`).
- **Button label inset** — every shared face sets layout `padding` **and** `label_walk.margin` (Makepad 2 Fill + Ellipsis otherwise pins ink to the face edge)
- **Reusable layout widgets** — see § Makepad 2 UI faces (agent guide) below (selection + truncation). Product summary: [`DESIGN.md`](DESIGN.md) § Makepad 2 faces.

## Desktop window size & maximize

Expected behaviour on **Linux (Wayland/GNOME), Windows, and macOS** for Kumkaj Courier client-side chrome:

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
| `apps/courier` `persist_window_geom` | Skip fullscreen / monitor-fill sizes |
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

Use this when changing bottom nav, choice toggles, chips, or any button label. Same UI code ships to **Linux / Windows / Android** — fix faces once in `apps/courier`, not per platform.

Cursor rule (always apply when editing kumkaj UI): `.cursor/rules/makepad-layout-faces.mdc`.

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
5. **Item / rate photos** — Courier has no built-in food-menu art pack. Thumbnails are name initials or a user-picked custom photo under the store’s item-images dir (not a packaged `resources/menu/` catalogue).

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

**Labels (no right truncation)**

| Kind of label | Face |
|---|---|
| Short chip in a horizontal strip | `ChipStrip` + `ChipBtn` / `ChipBtnActive` (Fit max 160, ellipsis OK) |
| Ultra-short side-by-side (≤ ~4 chars) | `ActionRow` + `RowPrimaryBtn` / `RowSecondaryBtn` |
| Multi-word / translated CTA | `ActionStack` + `StackPrimaryBtn` / `StackSecondaryBtn`, or `NavBtn` / `PayBtn` / `MoreRow` |
| Side-by-side fields (no CTAs) | `FormFieldRow` + `FieldInput` (equal `Fill`) |

Full-width CTA faces: `width: Fill`, `height: Fit`, `flow: Right{wrap: true}`, label `width: Fill`, **no** `max_lines: 1`, **no** `Ellipsis`. Always `padding` **and** `label_walk.margin`.

Never invent a one-off button type (`LangSourceBtn`, etc.) when Stack/Nav already cover it. Never put multi-word phrases in `ChipBtn` / `ActionRow`.

**FormFieldRow (no flex-shrink):** Makepad will crush a Fit CTA beside Fill inputs until wrap stacks each letter. Example regression: **Add rate** as a vertical navy pill overlapping Unit. Always:

```text
FormFieldRow{ price FieldInput; unit FieldInput }
ActionStack{ New item StackSecondaryBtn; Add rate StackPrimaryBtn }
```

### Code map (`apps/courier/src/main.rs`)

| Piece | Role |
|---|---|
| `TabBtn` / `TabBtnActive` | Bottom-nav idle / selected faces |
| `NavBtn`, `PayBtn`, `MoreRow`, `Stack*`, `Row*` | Full-width CTAs (wrap, no ellipsis) |
| `FormFieldRow` | Side-by-side fields only — never CTAs |
| `ChipBtn` / `ChipBtnActive` | Short strip chips only |
| `nav_tab_for` | `Screen` → which bottom peer is active |
| `paint_nav_tabs` | Exactly one footer peer visible-as-selected |
| `paint_choice_pair` / `clear_button_animators` | Dual-choice visibility + animator clear |
| `paint_choice_state` | All dual choices after chrome sync |
| `locale::en` + `english_chrome_never_returns_raw_keys` | Human labels, never snake_case keys |

```text
❌ Idle TabBtn color_hover = selected navy
❌ Rely on focus/hover as “selected”
❌ NavBtn / MoreRow with Ellipsis; ChipBtn for “Tax invoice” / language source
❌ FieldInput + FieldInput + NavBtn{width: Fit} in one Right row
❌ Patch one screen only — faces are shared across Linux / Windows / Android

✅ Overlay idle+active + paint from Screen + clear animators
✅ Wrap on full-width CTAs; ChipBtn only for short chips
✅ FormFieldRow for fields; ActionStack for multi-word CTAs under them
✅ Fix shared face definitions once (all platforms)
```

## Development vs production env

All platform package / bundle ids are **reverse-DNS under kumkaj.com**. Concrete products under the brand may use further ids (e.g. a courier face) still under that domain — see vision §1.

| | Development | Production |
|---|---|---|
| Env file | [`apps/courier/env/development.env`](../apps/courier/env/development.env) | [`apps/courier/env/production.env`](../apps/courier/env/production.env) |
| How | `cargo courier-dev`, `cargo courier-android-dev`, `cargo courier-windows-dev` | `./scripts/package-*.sh` |
| `KUMKAJ_APP_ID` | `com.kumkaj.courier.debug` | `com.kumkaj.courier` |
| `KUMKAJ_APP_LABEL` | Kumkaj Courier Dev | Kumkaj Courier |
| Size badge (`KUMKAJ_SHOW_SIZE_BADGE`) | on (`1`) — `W×H` in caption | off (`0`) |
| Windows console | yes (logs) | no (GUI subsystem) |

`apps/courier/build.rs` bakes the selected env file into the binary (`KUMKAJ_ENV` wins; otherwise debug Cargo profile → development, release → production). **Cargo profile ≠ app env:** `cargo courier-windows-dev` builds the **debug** profile with `development.env` (Dev app id / console). Android live-dev still uses Cargo `--release` for install speed, but exports `KUMKAJ_ENV=development` so the **debug** package id and badge stay on.

### Platform naming (dev vs prod)

Same `KUMKAJ_APP_ID` / `KUMKAJ_APP_LABEL` on every OS. How each platform surfaces them:

| Platform | Development | Production |
|---|---|---|
| **Android** package name / application id | `com.kumkaj.courier.debug` | `com.kumkaj.courier` |
| **Android** launcher label | Kumkaj Courier Dev | Kumkaj Courier |
| **iOS / macOS** bundle id (when signed) | `com.kumkaj.courier.debug` | `com.kumkaj.courier` |
| **Linux** binary | `cargo courier-dev` (host) · `package-linux-dev.sh` | `dist/linux/` + `install.sh` via `package-linux.sh` |
| **Windows** (Win10 + Win11 x86_64) | `cargo courier-windows-dev` → `dist/windows-dev/` (exe + resources) | `./scripts/package-windows.sh` → `dist/windows/` |
| **macOS** binary (when built on Mac) | `cargo courier-dev` | package script / packager |
| **Desktop packager** (`Cargo.toml` `[package.metadata.packager]`) | Dev installs ignore this; scripts pass `--package-name` / env | `identifier = "com.kumkaj.courier"`, `product_name = "Kumkaj Courier"` |
| **Web** | local `cargo courier-dev` / wasm as configured | site **https://kumkaj.com** (`web/`, `./scripts/package-web.sh`) — not an OS package id |
| **Play Store** listing | side-load / internal only | application id `com.kumkaj.courier` (`web/play-store-listing.txt`) |

### Data directories (keyed by `KUMKAJ_APP_ID`)

Dev and prod can sit side-by-side on one machine because paths differ by id. No extra vendor folder under the id.

| Platform | Development | Production |
|---|---|---|
| Linux | `~/.local/share/com.kumkaj.courier.debug/` | `~/.local/share/com.kumkaj.courier/` |
| Windows | `%APPDATA%\com.kumkaj.courier.debug\` | `%APPDATA%\com.kumkaj.courier\` |
| macOS | `~/Library/Application Support/com.kumkaj.courier.debug/` | `~/Library/Application Support/com.kumkaj.courier/` |
| Android | `/data/user/0/com.kumkaj.courier.debug/files/` | `/data/user/0/com.kumkaj.courier/files/` |
| Local servers | `servers.json` + `servers/{id}/courier.sqlite3` (one DB per business/server) | same |
| Client cache | `client/courier.sqlite3` | same |
| Backups | `…/<APP_ID>/backups/` | same |
| Override | `KUMKAJ_DATA_DIR` | same |

**DeviceSetup (every launch):** language (platform default, immediate apply) + Server/Client. Server Mode creates a unique local server if none exist, otherwise create-new or open-existing. See [`DESIGN.md`](DESIGN.md) § Launch / auth.

### Product / template default (onboarding)

**Kumkaj Courier** should open with courier defaults (Account/Cash booking, kg
rates language). The workspace may still contain other template chips for
later brand products; do not make “None” the story for courier demos.
- Pay methods: Cash · instant rail (UPI / Pix / PayNow / … per country pack) · Card · **Credit** · **Split Cash+rail** (Server and paired Client)
- **UI theme:** Stitch Tactile Countertop ([`DESIGN.md`](DESIGN.md); full pack at `../../stitch_tactile_countertop_design_system/`)
  - Logo palette: surface `#D6E0EC`, navy `#02030B` / `#072058`, orange `#C44E10`, yellow `#F7B043` (see [`DESIGN.md`](DESIGN.md))
  - White glyphs only on navy chrome — never light ink on light wells / placeholders
  - PIN, Booking, and the three-total ledger share the same tokens

## Android (device / emulator)

```bash
cargo courier-android-dev
```

Equivalent: `./scripts/courier-android-dev.sh`

Requires `ANDROID_HOME` (or `~/Android/Sdk`), a connected device/emulator, and Makepad’s Android toolchain (auto-installed on first run). Builds, installs, and launches **`com.kumkaj.courier.debug`** (from `apps/courier/env/development.env`).

**Launcher / logo:** Master art is `assets/courier-app-icon.png`. Run `./scripts/generate-app-icons.sh` to rebuild Android mipmaps, desktop `icon_*.png` / `icon.ico`, and in-app **`kumkaj-mark.webp`** (WebP). Reinstall Android after regenerating.

Android has **no** Splash `--hot` like desktop — each invocation rebuilds and relaunches. Day-to-day UI iteration stays on `cargo courier-dev`; use `cargo courier-android-dev` to verify on device.

Release APK/AAB packaging: `./scripts/package-android.sh` → `dist/android/`.

## Desktop install / uninstall (Linux + Windows)

Packages ship **Install** and **Uninstall** so owners never hunt data paths.
Dev and prod stay separate (debug suffix on id + install folder).

| | Development | Production |
|---|---|---|
| Env | `apps/courier/env/development.env` | `apps/courier/env/production.env` |
| App id | `com.kumkaj.courier.debug` | `com.kumkaj.courier` |
| Label | Kumkaj Courier Dev | Kumkaj Courier |
| Linux package | `./scripts/package-linux-dev.sh` → `dist/linux-dev/` | `./scripts/package-linux.sh` → `dist/linux/` |
| Windows package | `cargo courier-windows-dev` → `dist/windows-dev/` | `./scripts/package-windows.sh` → `dist/windows/` |
| Linux install dir | `~/.local/lib/kumkaj-courier-debug/` | `~/.local/lib/kumkaj-courier/` |
| Linux command | `kumkaj-courier-dev` | `kumkaj-courier` |
| Windows install dir | `%LOCALAPPDATA%\Programs\Kumkaj Courier Dev\` | `%LOCALAPPDATA%\Programs\Kumkaj Courier\` |
| Data dir | `…/com.kumkaj.courier.debug/` | `…/com.kumkaj.courier/` |

**Install:** run `install.sh` (Linux) or `install.ps1` (Windows) from the package folder. Adds Start Menu / app menu entries, including **Uninstall …**.

**Uninstall:** from the menu, Settings → Apps (Windows), or re-run `uninstall.sh` / `uninstall.ps1`. Always prompts:

- **Keep data** (default) — books stay; run uninstall again later to delete them  
- **Remove data** — deletes the app-id data directory  

If app files are already gone but data remains, re-running uninstall still offers to remove leftover data.

Shared helpers: `scripts/desktop-ids.sh`, `scripts/stage-desktop-package.sh`.

## Windows (cross from Arch / native on Windows)

One **x86_64** PE binary covers **Windows 10 and Windows 11**. Dev and prod use the same env split as Linux/Android (`com.kumkaj.courier.debug` vs `com.kumkaj.courier`).

| | Development | Production |
|---|---|---|
| Command | `cargo courier-windows-dev` | `./scripts/package-windows.sh` |
| Output | `dist/windows-dev/` (copy whole folder) | `dist/windows/` (copy whole folder) |
| Cargo profile | **debug** (`dev`) | **release** |
| Env | `apps/courier/env/development.env` | `apps/courier/env/production.env` |
| Data on Windows | `%APPDATA%\com.kumkaj.courier.debug\` | `%APPDATA%\com.kumkaj.courier\` |

**Cross-compile from Arch Linux** (this host):

```bash
sudo pacman -S mingw-w64-gcc
rustup target add x86_64-pc-windows-gnu
cargo courier-windows-dev          # Dev package → copy whole dist/windows-dev/ folder
./scripts/package-windows.sh      # Prod package → copy whole dist/windows/ folder
# Then on Windows: powershell -ExecutionPolicy Bypass -File .\install.ps1
# Makepad: MAKEPAD_PACKAGE_DIR=. with kumkaj-courier/resources/ beside the .exe
```

Linker is set in [`.cargo/config.toml`](../.cargo/config.toml) (`x86_64-w64-mingw32-gcc`). Day-to-day UI work stays on `cargo courier-dev` (Linux); Windows scripts only produce `.exe` packages to test on a real Windows box (or Wine if you choose).

**On a Windows machine with Rust:** the same scripts pick `x86_64-pc-windows-msvc` when available and build natively.

### Receipts on Android — known limitation

Makepad exposes the clipboard (`Cx::copy_to_clipboard` → `ClipboardManager`).
`open_url` is still unimplemented. Kumkaj adds two activity methods, applied
by `scripts/apply-android-activity-bridge.sh` before an Android build:

- **Copy receipt** puts the text on the system clipboard.
- **Print / share** opens an `ACTION_SEND` chooser (`shareText`) and also writes
  `receipts/receipt-YYYYMMDD-HHMMSS.txt` under the app files dir. Desktop still
  gets `lp`, then `xdg-open`/`open`/`start`.
- A gallery `content://` URI is read with `readContentUri` and saved as a real
  file before it becomes an item photo or venue icon.

Because the file is under `getFilesDir`, another app cannot open it without a
`FileProvider`; the clipboard is the working share path on Android today.

`arboard` and `rfd` are desktop-only dependencies — neither compiles for
Android, and `arboard` being unconditional used to break the Android build
outright.

## First-run unlock

Default seeded **Owner** PIN: `0000`

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

Bottom tabs: **Books** · **Sell** · **Buy** · **More** (Items, tables/kitchen/sales/day close/reports/inventory/branches/credit/team/backup live under More). Feature flags hide More rows the signed-in person cannot use. Unlock lands on **Books**.

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

## LAN Client / Server (`apps/courier/src/net.rs`)

Server listens on **TCP port 7847** when role = Server (pairing code in Settings / Lock hint).

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` | Liveness |
| GET | `/pair?code=` | Confirm code + venue |
| GET | `/sync?code=` | Snapshot: products, customers, ledger, tables, sales, KOTs, staff, day totals |
| POST | `/sale?code=` | Finalize sale (customer_name / table_name OK) |
| POST | `/settle?code=` | Credit settlement |
| POST | `/undo?code=` | Quick undo |
| POST | `/void?code=` | Void |
| POST | `/refund?code=` | Refund |
| POST | `/kot_advance?code=` | Advance kitchen ticket |
| POST | `/amend?code=` | Amend a sale (draft + reason + actor) |
| POST | `/kot?code=` | Send a kitchen ticket from a terminal's cart |

**Server is the sole writer.** A Client's tables are a sync cache that the next
snapshot overwrites, so writing them locally silently loses the edit. Every till
op a terminal can do goes over LAN — sell, settle, undo, void, refund, amend,
send KOT — and everything else is refused.

`Store::assert_server_writer` is the backstop: catalogue, stock, purchases,
staff, branches, tables, customers, expenses, fiscal settings and `amend_sale`
all return `"<action> on the Server — this device is a Client"` when
`device_role = client`, so no UI path can bypass it. The `*_from_sync` writers
skip the check — that is the Server's own data arriving. Screens that only
mutate (Items, Inventory, Branches, Team, Settings) still open on a Client for
lookups and show a read-only banner.

Credit on a Client sends `customer_name`, never a locally minted id, so the
Server resolves the customer in its own book; inventing an id here billed credit
against a customer the Server had never heard of.

Same Wi‑Fi trust model (no TLS). Sync payload includes categories, stations,
branch/org names and invoice numbers.

## Organization / branches (local)

- Meta: `org_id`, `org_name`, `active_branch_id`
- Tables: `branches`, `categories`, `kitchen_stations`, `stock_movements`, `purchases`, `purchase_lines`, `sync_outbox`
- Operational rows (products, sales, tables, KOTs, customers, expenses) carry `branch_id`
- **Export:** More → Branches → Export → `backups/branch_snapshot_{CODE}_{YYYY-MM-DD}.json` (+ clipboard)
- **Import:** place file as `backups/import_branch_snapshot.json`, then Import
- **Consolidated:** live day on this device + imported snapshots (stale/missing called out)
- Every finalize enqueues `sync_outbox` (`sale.finalize`). The queue is write-only; local billing never waits on it, and no process drains it until Kumkaj Cloud exists. When a drain marks an event synced, `mark_outbox_synced` also stamps `sales.server_at` (triple timestamps).

## Smoke tests

```bash
cargo test -p kumkaj-courier
```

## Packaging

```bash
./scripts/package-linux.sh            # prod → dist/linux/ (+ install.sh / uninstall.sh)
./scripts/package-linux-dev.sh        # debug → dist/linux-dev/
./scripts/package-windows.sh          # prod → dist/windows/ (+ install.ps1)
cargo courier-windows-dev              # debug → dist/windows-dev/
./scripts/package-android.sh
./scripts/package-web.sh
./scripts/package-apple-scaffold.sh   # iOS/macOS deferred until Mac host
```

Desktop install/uninstall: see **Desktop install / uninstall** above.

## Unlock model

- **Owner** — hosts this local business; unlock with Owner PIN (default `0000`)
- **Staff** — pick a non-owner person, then their PIN
- Optional login id + password (Team can set once; only the user can change password later)
- More → **Team**: Owner creates staff (role title + permission preset + PIN + features); anyone can update their own PIN
- Sale **created_by** is set on finalize and never rewritten on amend

## Publish checklist (Play / web / desktop)

1. `cargo test -p kumkaj-courier`
2. `./scripts/package-linux.sh` → smoke `dist/linux/run.sh` / `./install.sh`
3. `./scripts/package-windows.sh` → `dist/windows/` (+ `install.ps1`); optional `cargo courier-windows-dev` for Dev
4. Clone full Makepad `main`, set `MAKEPAD_ROOT`, run `./scripts/package-android.sh`
5. Sign Android release; create Play Console listing (title, screenshots from Stitch, privacy: local-only data)
6. `./scripts/package-web.sh` → deploy `dist/web/` to **https://kumkaj.com**
7. iOS / macOS: scaffold only until Mac signing — `./scripts/package-apple-scaffold.sh`

Platform ids: see **Platform naming** above (`com.kumkaj.courier` / `com.kumkaj.courier.debug`, domain kumkaj.com).

## Manual UI smoke (shipping)

1. Unlock with `0000` (or set Owner PIN on first Server run)
2. Finish onboarding — Kumkaj Courier is locked; Fresh or Book, then Done
3. Booking desk: weigh / rate / cash or account; confirm bill totals (Actual / Applied / Rounded)
4. Day book shows today; Pay out records carrier bills & expenses; More → Parties settle; Day close; Reports
5. More → Rates: zones/services, tax rate name or %, optional photo (initials or custom — no food-menu pack)
5b. Settings → Country & tax: apply `GB` (VAT, inclusive, £), then `US` (sales tax, exclusive, $); save a business id and check it prints
6. Ops / COD / Riders when fleet features are used
7. Branches & org: add branch, switch, export snapshot, import `import_branch_snapshot.json`, consolidated report
8. Extra till: pair IP + code after the shop allows extra tills → book against the shop

## Schema

Current meta `schema_version` **1** (pre-release: one create, no upgrade ladder).

Fresh DBs get the full books shape in a single `CREATE TABLE` pass in `Store::migrate`:

- org / branches, categories, products/rates (tax, category, modifiers, stock, units), sales (invoice, tax, money lane, order type, authorship, **triple timestamps**), sale lines (milli qty), payments, revisions, customers / credit + payable ledgers, purchases / purchase lines, expenses, money movements, stock movements, tax rates, units + conversions, sync outbox, staff features; kitchen stations / tables / KOT tables remain in the shared schema for a future food product but are not exposed in Courier UI
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

## Fiscal country packs (`apps/courier/src/fiscal.rs`)

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
