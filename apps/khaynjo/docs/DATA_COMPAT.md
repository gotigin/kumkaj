# Khaynjo — published books compatibility

Khaynjo is shipping on **Play Store**, **Microsoft Store (MSIX)**, and **Arch AUR**.
From the first public build, every shop’s SQLite books are a **forever contract**:
an upgrade must open them, migrate forward safely, or refuse with a clear
message — never crash, never pin a newer schema down, never delete client
data to “fix” an upgrade.

Canonical code: `apps/khaynjo/src/store.rs` (`SCHEMA_VERSION`, `bring_up_to_date`,
`backup_to` / `restore_from_to_path`, `check_backup_file`).

## Hard rules

1. **Forward-only.** `meta.schema_version` (and `PRAGMA user_version`) only
   increase. A build that understands schema *N* opens *1…N*. Schema *N+1*
   from a newer Khaynjo is **refused** and the file is left untouched.
2. **Bump = migrate step + tests.** Raising `SCHEMA_VERSION` requires
   `migrate_N_to_N+1`, a pre-migrate snapshot, and tests that an *N* file
   with a sale still opens and keeps that sale.
3. **No silent “pin to current”.** Never rewrite a higher `schema_version`
   back to an older number. That was pre-release behaviour; it is forbidden
   after publish.
4. **Additive within a version only for repairs.** Schema **1** may still
   `CREATE IF NOT EXISTS` / `ALTER TABLE … ADD COLUMN` to repair incomplete
   pre-release files. **New product features** that need columns or tables
   ship as schema **2+**, not as unversioned alters.
5. **Pre-migrate snapshot.** Before applying *N→N+1*, write
   `backups/pre-migrate-vN-to-vM-<timestamp>.sqlite3` via `VACUUM INTO` and
   `check_backup_file`. If the snapshot fails (disk full, permission),
   **abort** the upgrade — do not half-migrate.
6. **Integrity before migrate.** Existing books get `PRAGMA quick_check`
   before any schema write. Damage → tell the owner to restore; do not upgrade.
7. **Backups are verified.** Auto / manual backup uses `VACUUM INTO` then the
   same checks restore uses (SQLite quick_check, Khaynjo tables, `meta.app`
   / `entity_template`, schema not newer than this build). A bad write is
   deleted, never left looking like a backup.
8. **Restore is atomic.** Restore copies into the live DB through SQLite’s
   backup API (not rename-over), then opens + migrates. Wrong app / newer
   schema / damaged file never replaces working books.
9. **App mark.** Live books and backups carry `meta.app = khaynjo`. Other
   Kumkaj products’ files are rejected on restore.
10. **Store / AUR versioning.** One **product** version (Cargo
    `version` / `version_name` / MSI / MSIX / AUR `pkgver`) for every
    platform of the same release. Platform-only integers (Play
    `version_code`, AUR `pkgrel`, MSIX fourth digit) may differ but must
    still move **forward** on each upload. Play `version_code` is a small
    manual counter in Cargo (**start at `1`**; **+1** per AAB) — never
    timestamp `auto` / YYYYMMDDHH (see
    [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md)). A schema bump is always
    accompanied by a product version bump so clients are not stuck on a
    build that cannot open their books.

## Intentional history clear (not corruption)

Owner-controlled **trading history** wipe is separate from schema upgrades:

| Action | Code path | Effect |
|--------|-----------|--------|
| Settings → Backup → **Clear history now** | `purge_trading_history_keep_setup` | Full clear → empty books / zero defaults |
| History retention **on day close** | same purge after day close | same |
| Emergency unlock PIN at login | `wipe_trading_data_keep_structure` → **same purge** | same; opens as Owner, looks normal |
| Retention **after N hours** (e.g. 12) | `purge_trading_older_than_hours` | Deletes rows **older than N hours** only (recent bills can remain) |

Full clear deletes sales/ledgers/expenses/purchases/KOTs/open floor work/audit/vouchers/outboxes, resets tracked stock / loyalty / wallet / invoice & KOT sequences to starting defaults (mostly **zero**), and **keeps** schema, menu/prices, parties, tables, staff, settings, and the emergency PIN. Distinct from Settings uninstall wipe (which can also remove shop setup).

## What the owner should still do

Local reliability does not replace off-device copies:

- Automatic daily backups under the data dir’s `backups/` (last 14 kept).
- Settings → **Also copy backups to a folder…** (USB / synced folder).
- After `COPY_DUE_DAYS` with no off-device copy, Backup settings warns.

A device that dies takes on-device backups with it.

## Agent / developer checklist (every schema change)

- [ ] Bump `SCHEMA_VERSION` in `store.rs`
- [ ] Implement `migrate_N_to_N+1` (transactional; no `DROP` of client tables
      without a copy/transform plan)
- [ ] Wire it in the `bring_up_to_date` match ladder
- [ ] Test: refuse schema `SCHEMA_VERSION + 1`
- [ ] Test: open a schema-*N* fixture (or freshly made *N* DB) on the new
      build; sale / invoice / lines still present
- [ ] Test: backup from *N* restores on the new build
- [ ] Update this doc’s “Current schema” line
- [ ] Never change `meta.app` / Identity / package id in a way that orphans
      existing `%APPDATA%\com.khaynjo\` / Android files / Linux data dirs

## Current schema

| Version | Meaning |
|--------:|---------|
| **1** | First public Khaynjo books shape (restaurant / KOT / tables / books spine), shipped with product **1.0.0**. |

Release index: [`RELEASE_1_0_0.md`](RELEASE_1_0_0.md). Publish: [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md), [`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md), [`PUBLISH_AUR.md`](PUBLISH_AUR.md).

## Platform publish notes (Khaynjo)

| Channel | Package / id | Data dir (typical) |
|---------|----------------|--------------------|
| Play Store | `com.khaynjo` | app-scoped files via Makepad |
| Microsoft Store (MSIX) | Identity `Kumkaj.Khaynjo` | `%APPDATA%\com.khaynjo\` (packaged classic / mediumIL) |
| MSI sideload | same app id | same AppData path |
| Arch AUR | desktop package → `com.khaynjo` | `~/.local/share/com.khaynjo/` (via `dirs::data_dir`) |

Uninstall must **keep** books by default (Play, Store, MSI, pacman). Wipe is
an explicit owner action (in-app Backup wipe or MSI uninstall checkboxes).
