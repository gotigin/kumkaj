# Kumkaj developer notes

Commands and paths below are from the **workspace root** (parent of `docs/`).

## Live development (Cursor)

```bash
cargo kumkaj-dev
```

Loads [`env/development.env`](../env/development.env) then `cargo run -p kumkaj -- --hot --remote`.

- `--hot` — Splash/`script_mod!` hot reload on save
- `--remote` — Tweaker via **F12**
- Makepad **2 / `main` only** — never Makepad 1 docs/examples/APIs
- Local Makepad 2 tree: `vendor/makepad`
  - First time / after clean: `./scripts/setup-makepad.sh`
  - Path-dep from `apps/kumkaj`; applies Wayland scroll-sign + dark caption button patches
  - Nested Settings lists claim wheel events so the page does not scroll with the list
- Multi-script UI fonts in `apps/kumkaj/resources/fonts/` (Noto Sans for Indian + world scripts; Font Awesome for ✕/⌫/✓; CJK/emoji from Makepad widgets). Re-fetch with `./scripts/fetch-ui-fonts.sh` if fonts are missing.
- **Mobile-first** default window (`390×844`) on desktop — seller-sized frame
- **Min layout width 320 css px** — `app_shell` uses `Fill{min: 320.}` inside a horizontal scroll. The UI never lays out narrower than 320; if the device/window is below 320, content scrolls horizontally
- **Dev size** — `W×H` in the window caption bar, left of the Kumkaj title (terracotta on navy). Controlled by `KUMKAJ_SHOW_SIZE_BADGE` in the env file (not Cargo `--release`).
- **Button label inset** — every shared face sets layout `padding` **and** `label_walk.margin` (Makepad 2 Fill + Ellipsis otherwise pins ink to the face edge)
- **Reusable layout widgets** (Makepad 2 gaps): `ChipStrip` + `ChipStripRow` + `ChipBtn` / `ChipBtnActive` (wheel/scrollbar scroll — not drag, so taps work; selected = active face via visibility swap), `ActionRow` (short labels only) + `RowPrimaryBtn` / `RowSecondaryBtn`, `ActionStack` (full-width stacked actions — prefer at 320px for multi-word CTAs)

## Development vs production env

All platform package / bundle ids are **reverse-DNS under kumkaj.com**. There is no Gotigin (or other vendor) segment in ids or data paths.

| | Development | Production |
|---|---|---|
| Env file | [`env/development.env`](../env/development.env) | [`env/production.env`](../env/production.env) |
| How | `cargo kumkaj-dev`, `cargo kumkaj-android-dev` | `./scripts/package-*.sh` |
| `KUMKAJ_APP_ID` | `com.kumkaj.app.debug` | `com.kumkaj.app` |
| `KUMKAJ_APP_LABEL` | Kumkaj Dev | Kumkaj |
| Size badge (`KUMKAJ_SHOW_SIZE_BADGE`) | on (`1`) — `W×H` in caption | off (`0`) |

`apps/kumkaj/build.rs` bakes the selected env file into the binary (`KUMKAJ_ENV` wins; otherwise debug Cargo profile → development, release → production). Android live-dev still uses Cargo `--release` for install speed, but exports `KUMKAJ_ENV=development` so the **debug** package id and badge stay on.

### Platform naming (dev vs prod)

Same `KUMKAJ_APP_ID` / `KUMKAJ_APP_LABEL` on every OS. How each platform surfaces them:

| Platform | Development | Production |
|---|---|---|
| **Android** package name / application id | `com.kumkaj.app.debug` | `com.kumkaj.app` |
| **Android** launcher label | Kumkaj Dev | Kumkaj |
| **iOS / macOS** bundle id (when signed) | `com.kumkaj.app.debug` | `com.kumkaj.app` |
| **Linux / Windows / macOS** binary name | `kumkaj` (dev run via cargo) | `kumkaj` / `kumkaj.exe` under `dist/` |
| **Desktop packager** (`Cargo.toml` `[package.metadata.packager]`) | Dev installs ignore this; scripts pass `--package-name` / env | `identifier = "com.kumkaj.app"`, `product_name = "Kumkaj"` |
| **Web** | local `cargo kumkaj-dev` / wasm as configured | site **https://kumkaj.com** (`web/`, `./scripts/package-web.sh`) — not an OS package id |
| **Play Store** listing | side-load / internal only | application id `com.kumkaj.app` (`web/play-store-listing.txt`) |

### Data directories (keyed by `KUMKAJ_APP_ID`)

Dev and prod can sit side-by-side on one machine because paths differ by id. No extra vendor folder under the id.

| Platform | Development | Production |
|---|---|---|
| Linux | `~/.local/share/com.kumkaj.app.debug/` | `~/.local/share/com.kumkaj.app/` |
| Windows | `%APPDATA%\com.kumkaj.app.debug\` | `%APPDATA%\com.kumkaj.app\` |
| macOS | `~/Library/Application Support/com.kumkaj.app.debug/` | `~/Library/Application Support/com.kumkaj.app/` |
| Android | `/data/user/0/com.kumkaj.app.debug/files/` | `/data/user/0/com.kumkaj.app/files/` |
| DB file | `kumkaj.sqlite3` in that dir | same |
| Backups | `…/<APP_ID>/backups/` | same |
| Override | `KUMKAJ_DATA_DIR` | same |

### Start template default (onboarding)

Chip strip: **None** / Counter / Courier / Food / Personal. Stable store id for None is `custom`. **None is selected by default** (active chip + meta). If the owner proceeds with Fresh / Bring my book / Done without tapping another template, None is applied automatically.
- Pay methods: Cash · instant rail (UPI / Pix / PayNow / … per country pack) · Card · **Credit** · **Split Cash+rail** (Server and paired Client)
- **UI theme:** Stitch Tactile Countertop ([`DESIGN.md`](DESIGN.md); full pack at `../../stitch_tactile_countertop_design_system/`)
  - Cool surface `#f9f9fc`, navy `#041534` / `#1b2a4a`, terracotta `#a23e18`
  - Cream glyphs only on navy window caption — never cream-on-white body text
  - PIN, Sell, and the three-total ledger share tokens with floor, kitchen, and credit

## Android (device / emulator)

```bash
cargo kumkaj-android-dev
```

Equivalent: `./scripts/kumkaj-android-dev.sh`

Requires `ANDROID_HOME` (or `~/Android/Sdk`), a connected device/emulator, and Makepad’s Android toolchain (auto-installed on first run). Builds, installs, and launches **`com.kumkaj.app.debug`** (from `env/development.env`).

**Launcher icon:** PNGs under `apps/kumkaj/resources/android/res/mipmap-*/ic_launcher.png` (required by cargo-makepad). Regenerate from `apps/kumkaj/resources/kumkaj-mark.png` with `./scripts/generate-app-icons.sh`, then reinstall.

Android has **no** Splash `--hot` like desktop — each invocation rebuilds and relaunches. Day-to-day UI iteration stays on `cargo kumkaj-dev`; use `cargo kumkaj-android-dev` to verify on device.

Release APK/AAB packaging: `./scripts/package-android.sh` → `dist/android/`.

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

Bottom tabs: **Sell** · **Items** · **More** (tables/kitchen/sales/day close/reports/inventory/branches/credit/team/backup live under More). Feature flags hide More rows the signed-in person cannot use.

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
- PINs are salted SHA-256 hashes, not plaintext, but a 4-digit PIN is brute
  forceable offline in moments. A leaked backup should be followed by new PINs.
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

## LAN Client / Server (`apps/kumkaj/src/net.rs`)

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
- Every finalize enqueues `sync_outbox` (`sale.finalize`). The queue is write-only; local billing never waits on it, and no process drains it until Kumkaj Cloud exists

## Smoke tests

```bash
cargo test -p kumkaj
```

## Packaging

```bash
./scripts/package-linux.sh
./scripts/package-windows.sh
./scripts/package-android.sh
./scripts/package-web.sh
./scripts/package-apple-scaffold.sh   # iOS/macOS deferred until Mac host
```

## Unlock model

- **Owner** — hosts this local business; unlock with Owner PIN (default `0000`)
- **Staff** — pick a non-owner person, then their PIN
- Optional login id + password (Team can set once; only the user can change password later)
- More → **Team**: Owner creates staff (role title + permission preset + PIN + features); anyone can update their own PIN
- Sale **created_by** is set on finalize and never rewritten on amend

## Publish checklist (Play / web / desktop)

1. `cargo test -p kumkaj`
2. `./scripts/package-linux.sh` → smoke `dist/linux/kumkaj`
3. `./scripts/package-windows.sh` on a Windows host (or cross toolchain)
4. Clone full Makepad `main`, set `MAKEPAD_ROOT`, run `./scripts/package-android.sh`
5. Sign Android release; create Play Console listing (title, screenshots from Stitch, privacy: local-only data)
6. `./scripts/package-web.sh` → deploy `dist/web/` to **https://kumkaj.com**
7. iOS / macOS: scaffold only until Mac signing — `./scripts/package-apple-scaffold.sh`

Platform ids: see **Platform naming** above (`com.kumkaj.app` / `com.kumkaj.app.debug`, domain kumkaj.com).

## Manual UI smoke (shipping)

1. Unlock with `0000` (or set Owner PIN on first Server run)
2. Finish onboarding — **None** is pre-selected; Fresh or Book, then Done (other templates optional)
3. Add Tea ×4; optionally edit `@`, line **Applied**, and **Note**; set order type Counter / Dine-in / Takeaway
4. Confirm bill shows three totals:
   - **Actual Total** = sum of `@ × qty`
   - **Applied Total** = sum of line Applied (equals Actual until a line is overridden)
   - **Rounded Total** = editable; defaults to Applied — edit e.g. 52 → 50 for bill round-off
5. Cash / rail / Card / Credit or Split Cash+rail; undo snackbar; Copy / Print receipt from Sales
6. More → Credit settle; Day close; Reports & expenses
7. Items: category, station, tax rate name or %, modifiers text; Inventory track/adjust/purchase
7b. Settings → Country & tax: apply `GB` (VAT, inclusive, £), then `US` (sales tax, exclusive, $) and confirm the Sell strip shows the added tax before you charge; save a business id and check it prints
8. Tables & kitchen: Send KOT, tables park carts, Kitchen station filter + Advance
9. Branches & org: add branch, switch, export snapshot, import `import_branch_snapshot.json`, consolidated report
10. Extra till: pair IP + code after the shop allows extra tills → sell against the shop

## Schema

Current meta `schema_version` **15** (entity templates, unit labels, milli qty, bill-type tag / Account vs Cash booking, cashbook / bank movements).

- v11 — org/branches, categories, kitchen stations, stock movements, purchases, sync outbox, product tax/station/category/modifiers/stock, sale order_type/tax/branch
- v12 — worldwide fiscal packs: `tax_rates` table (per branch), `products.tax_rate_id`, `sales.tax_pricing` / `tax_label` / `net_paise` / `service_paise`, and `meta` keys `country_code`, `tax_pricing`, `tax_label`, `business_tax_id`, `business_id_label`, `digital_pay_label`, `credit_label`, `date_order`, `decimal_sep`, `thousands_sep`, `service_charge_bps`. Upgrading seeds the India pack (no behaviour change for existing shops) and backfills `net_paise = total − tax` for old inclusive sales.
- v13 — `sales.invoice_no` plus `meta` keys `next_invoice_seq` and `pin_reset_required`. Rows written earlier keep an empty `invoice_no` and print the short id instead (`Sale::bill_no`).
- v15 — `money_movements` cashbook (till↔bank / bank in/out); Reports: period books, party aging, bank posts; richer receipts (party, qty units, Account/Cash booking bill type).

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

## Fiscal country packs (`apps/kumkaj/src/fiscal.rs`)

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
