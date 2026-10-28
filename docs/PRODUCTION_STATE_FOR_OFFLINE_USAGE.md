# Production state for offline (LAN) use — Moklanjo & Khaynjo

> **Home:** root `docs/` (both products). Product briefs link here:
> [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md),
> [`KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md).

Updated **2026-10-27** (v3). Research Local/LAN catch-up for both apps is
complete (§4); earlier LAN pilot items (§1–2) remain. Every line below was
checked against the code, not copied from an older assessment.

**Bottom line:** product code for offline/LAN use (including research
parity that does not need Kumkaj Cloud) is ready for a supervised pilot
(a counter device + two extras, any mix of supported platforms). What
blocks a non-technical roll-out is no longer Local/LAN feature gaps: it
is the **real-hardware run** and **installing the built packages on clean
machines** (§3), which only the owner can do.

---

## 1. Fixed / completed in this pass

| Item | What changed | Where |
|---|---|---|
| **KOTs from tablets delayed up to 20 s** (bug) | The "exponential backoff" listed as done skipped *every* worker round after a quiet one — including the round a waiter's Send KOT or a cancel woke. Now anything asked for (a KOT, a payment, a refresh, the regular pull) goes at once; only the unasked 5 s tick pauses while the Server is out of reach. Test `asked_rounds_are_never_held_back`. | both apps, `phone.rs` (`round_due`) |
| **Sync grew without limit** (bug, Khaynjo) | Kitchen tickets nobody tapped Done on (a kitchen that cooks from the paper KOT) stayed "open" for ever: at 9,000 bills every tablet's sync was **3 MB**, and every floor colour scanned them all. Tickets older than 12 h (`KOT_OPEN_HOURS`) are no longer open → **417 KB**. Test `kitchen_tickets_nobody_closed_age_out`. | `store.rs` `list_open_kots` |
| Scale testing | New tests: **10 tablets/devices at once** (KOTs or bookings + pulls, every other reply lost and re-sent): nothing lost or doubled, no "database is locked". Opt-in **months of trading** test: 9,000 bills over 30 days, 1,000 customers, 300 items — fails if the snapshot passes 600 KB or any step takes 3 s. Numbers in §5. | both apps, `net.rs` tests |
| Why writes are stuck | **Connection to the shop** now shows, per waiting write, how many tries and the last answer ("Tried 4 times · last answer: …"), translated in all 29 languages. | both apps, `main.rs`, `locale*.rs` |
| "Who changed this price" | A menu edit is audited as `menu.update` **Name · ₹120 → ₹100**; the Settings audit line now names who did each action. | both apps, `main.rs` |
| Server label leaking as the restaurant's name | A server created without a name got "Server af76697b" written as the venue name (header, lock screen, bills). No longer written; books that took it read as unnamed; other devices list such a shop by its business name. Test `generated_server_label_is_not_the_restaurant_name`. | Khaynjo `local_servers.rs`, `store.rs`, `net.rs` |
| Guides contradicted the code | `docs/LAN_QUICK_START.md` and `docs/SERVER_MIGRATION.md` rewritten from the code: the pairing code does **not** expire after 5 min; Stop extra tills does not "forget" devices; a tablet that lost power does not re-pair; the old "verify by starting a restore, then cancel" step would have replaced the live books (there is no cancel); the "zero-downtime" recipe ran two servers with the same books. | `docs/` |
| Dead code from the earlier pass | `devices_not_seen_since` (never called — the devices list already shows "last seen" / "connected now") and the unused `server_name_header` label removed. | both apps |

**2026-10-26 (packaging):** root `scripts/` now names no app; each app's
packages, installers, uninstallers, `desktop-ids.sh`, `License.rtf` and MSI
codes live in `apps/<app>/scripts/` (DEV.md § Common workspace → Scripts).
Two Khaynjo packaging bugs this exposed are fixed: its Windows *folder*
package staged Moklanjo's `Setup.bat`/`install.ps1` (they look for
`kumkaj-moklanjo.exe` — the folder install failed), and its MSI licence page
said "Moklanjo". Header names (both apps): only what the owner saved —
the shop's name (app name until saved), the business name (none shown
until saved; the old "My business" placeholder is never shown), then who is
signed in, name and role.

Also this pass (Khaynjo UI, owner's request): header shows the app name
with business name + who is signed in; the Order screen's search, category
tabs and dish grid moved into a **+ Add dishes** window (≥ 2 tiles per row);
the duplicate "On this order" tiles removed; ‹ › tab chevrons drawn and
centred. See `apps/khaynjo/docs/KHAYNJO.md` §4, §4d.

**2026-10-26 (printer + kitchen fix pass):** header labels changed from Markdown
`WrapLabel` to plain `Label` so white ink stays visible on the dark/maroon
chrome; the printer pool no longer invents a synthetic "Default" printer on
fresh install (blank target = this computer's OS printer, not a named pool
entry); legacy "Default" entries scrubbed on load, never re-created on migrate;
each Khaynjo kitchen station **gets its own independent KOT printer route**
(unknown stations no longer inherit the unassigned-items route). Tests and
`docs/DEV.md` aligned. See commit `5b54ca8`. The items from the previous
finish pass (quick-start card, tablet help, MSIs) remain in this table as
verified done.

| Item | What changed | Where |
|---|---|---|
| **Invented "Default" printer removed** | `DevicePrinters::default()` now starts empty — no printer, no routes. `add_printer` with blank target+label uses "Printer" (not "Default"). `migrate_legacy` of blank strings → empty routes (no synthetic Default). `scrub_fake_default_printers()` drops old "default" entries on deserialization. `describe(PrinterTarget::Default)` text updated. | both apps, `print_share.rs` |
| **Each kitchen station gets its own KOT route** | Stations no longer fall back to the unassigned-items (`""`) printer route. `ensure_station_routes()` called on station creation and Settings load. Station name shown in route rows. New locale keys `send_kot_to`, `kot_dest_hint`, `kot_unassigned`. | Khaynjo `print_share.rs`, `main.rs`, `locale.rs` |
| **Header white text reverting to dark on chrome** | `brand_title` and `session_label` changed from `WrapLabel` (Markdown — sometimes reverts to dark default) to `Label` (plain text with explicit `draw_text.color: #xffffff`). | both apps, `main.rs` |
| **DEV.md printer route docs** | `""` described as "unassigned items only — stations do not share a printer"; blank target = "this computer's usual OS printer — never invent a pool entry named 'Default'". | `docs/DEV.md` |
| **Quick-start card in installers** | `docs/LAN_QUICK_START.md` is staged as `Quick start.txt` in every desktop package (`stage-desktop-package.sh`) and Android package folder; MSI adds a Start Menu **Quick start** shortcut when the file is present. Rebuilt `dist/Khaynjo-0.1.0.msi` and `dist/Moklanjo-0.1.0.msi` so the card is inside the installer. Confirm by installing once on Windows (owner). | `scripts/stage-desktop-package.sh`, `scripts/build-msi.sh`, both apps' package scripts |
| **In-app help for tablets** | Settings → Extra tills shows **Set up other devices** with the short pairing steps (Server allow + code/address, client Find/Pair, Wi‑Fi drop behaviour). Fully translated in all 29 shipped languages. | both apps, `main.rs`, `locale.rs`, `locale_chrome.rs` |
| **Device clock far off the Server** | Already shipped earlier this week (header warns past 5 min via `X-Kumkaj-Time` / `clock_skew_secs`); marked complete here so §4 is empty of open code work. | both apps, `net.rs`, `main.rs`, DEV.md § Clock |

---

## 2. Already in place (verified in code, both apps unless noted)

| Capability | Where |
|---|---|
| LAN server: TCP + UDP discovery — Courier 7857/7858, Khaynjo 7847/7848 | `net.rs` |
| Pairing code + per-device token; 5 wrong codes/PINs → that address waits 5 min | `net.rs` `Guard` |
| PINs and rights judged on the Server; hashes never leave it | `net.rs`, `Store::check_right` |
| Server is the sole writer (`assert_server_writer`) | `store.rs` |
| Each write once (`X-Kumkaj-Request` reply store) | `net.rs` |
| Offline queue on the device (`lan_outbox`), survives restarts, sent in order | `phone.rs` |
| A write the Server refuses for good leaves the queue → "not accepted" list with the Server's words (no poison message blocking the queue) | `phone.rs`, `lan_outbox_refused` |
| Live updates: `/wait` long-poll + revision (304 when unchanged) + gzip | `phone.rs`, `net.rs` |
| Server found again by **server id** after an address change (every 60 s while silent); manual IP entry | `phone.rs` `refind_server` |
| Header: "Offline · N waiting", "Pair this device again", how old the screen is | `main.rs` |
| Connection panel: waiting writes (now with tries + last answer), not-accepted list | `main.rs` |
| Paired devices list with last seen / connected now, **Forget**, **New pairing code** | `main.rs` |
| Daily automatic backup (14 kept) + copy to a second folder; failures and low disk shown to the owner | `store.rs` `auto_backup_if_due`, `backup_health` |
| Every backup read back when written; restore refuses a damaged or other-app file and saves the current books first | `check_backup_file`, `restore_backup_from` |
| Keep awake while serving (Windows/Linux; Android screen on via Makepad patch) | `net::KeepAwake`, `android_keep_screen_on` |
| Start with this computer (`--serve`), Windows firewall rule from Settings | `startup.rs`, `net.rs` |
| Pull interval: 20 s default (`sync_interval_secs` meta, min 10 s — no UI, none needed) | `phone.rs` |
| Clock skew warn (≥ 5 min off Server) in the header | `net::clock_skew_secs`, `CLOCK_WARN_SECS` |
| Settings help: set up other devices (quick-start steps) | `main.rs` Settings → Extra tills |
| Quick start.txt + Start Menu shortcut in MSI / desktop packages | `stage-desktop-package.sh`, `build-msi.sh` |
| Tests never print on paper | `scripts/check-tests-never-print.sh` |

---

## 3. Still needed — only the owner can do

1. **Real-hardware run** (Windows counter + two Android tablets, the
   Rajdhani setup): KHAYNJO.md §6 lunch-service and tablet acceptance. Watch
   for: discovery on the real router (guest networks / AP isolation block
   it — manual IP is the fallback), the firewall prompt on Win 10/11,
   **Start with this computer** after a real power cut, sleep with the lid
   closed, the KOT printer through Windows.
2. **Install the built packages on clean machines.** Rebuilt on
   2026-10-26 (previous pass): `dist/Khaynjo-0.1.0.msi` (Windows),
   `dist/Moklanjo-0.1.0.msi` (Windows), folder packages under
   `dist/khaynjo-windows/` / `dist/moklanjo-windows/`, and the Android APK
   (`apps/khaynjo/scripts/package-android.sh`). Not yet installed on a
   clean machine of any supported platform. Printer pass (2026-10-26 v2) did
   not change packaging — same packages are valid.
3. **Android release signing + Play listing** (Moklanjo and Khaynjo), web
   deploy (kumkaj.com / khaynjo.com). iOS/macOS stay scaffold-only until a
   Mac exists.

---

## 4. Still open in code

**Worth doing next**

| Item | Why / how |
|---|---|
| *(none in Local/LAN product code)* | Research catch-up for both apps is **Have** for everything that does not need Kumkaj Cloud — see [`RESTAURANT_RESEARCH.md`](../apps/khaynjo/docs/RESTAURANT_RESEARCH.md) §8 and [`COURIER_RESEARCH.md`](../apps/moklanjo/docs/COURIER_RESEARCH.md) §7. **Emergency unlock PIN** (Owner duress wipe of trading history, structure kept) shipped 2026-10-27 in both apps. Remaining product work is Cloud/Net connectors (optional stubs only) or owner actions below. |

**Still needs the owner (not code)**

| Item | Why |
|---|---|
| Real-hardware pilot | Windows counter + tablets on the shop LAN (§3) |
| Clean-machine install of built MSI/APK | Confirm Start Menu, firewall, Quick start |
| Android release signing + Play / web listing | Distribution |

**Deliberately not done (decisions, not gaps)**

| Item | Reason |
|---|---|
| Encrypted backups / DB at rest | DEV.md § Backup format: a local-POS backup is a plain copy the owner controls; encryption belongs with Kumkaj Cloud. A lost USB stick exposes the books — say so to the owner. |
| mDNS / `server.txt` / IPv6 / other subnets | Broadcast + /24 scan + find-by-server-id + manual IP cover a shop LAN; manual IP also works across routed subnets. Revisit only if a real site fails. |
| Tamper-evident (hash-chained) audit | Vision §18 asks for attributable, append-only history — that exists (audit events, sale revisions, cancel-not-delete). A cryptographic seal only matters where a tax regime demands it. |
| Zero-downtime server migration | One Server is the only writer by design; migration is a few minutes outside service (SERVER_MIGRATION.md). |
| Colour dot for link health | The header says it in words; colour alone would break the "never carry meaning in colour alone" rule (DEV.md). |
| GSTR-1/3B, e-invoice IRN/QR, tax-period lock, FIFO valuation, **live** cloud features | Vision §23 / later phases; not claimed anywhere. Optional Net stubs (aggregators, card outbox, WhatsApp share) must never gate the counter. SMS: local outbox Have; Local SMS Gateway + Cloud SMS Planned — [`SMS_MESSAGING.md`](SMS_MESSAGING.md). |

---

## 5. Measured (2026-10-25, this machine, release build)

Opt-in test `months_of_trading_stay_quick` — 9,000 bills over 30 days
(300 today), 1,000 customers, 300 items:

| Step | Khaynjo | Moklanjo |
|---|---|---|
| Snapshot built | 18 ms | 16 ms |
| Snapshot size (before gzip) | 417 KB (was 3,083 KB) | 330 KB |
| `/sync` answered | 20 ms | 20 ms |
| Today's totals | 0.3 ms | 0.3 ms |
| Day close | 10 ms | 9 ms |
| Day report | 1 ms | — |
| Backup | 77 ms | 73 ms |

Not measured: low-end Android memory and battery while serving or syncing
all day — do it during the hardware run.

---

## 6. Re-check

```bash
cargo test -p kumkaj-khaynjo        # ~229 #[test] markers in src
cargo test -p kumkaj-moklanjo        # ~181 #[test] markers in src
apps/khaynjo/scripts/check-tests-never-print.sh
apps/khaynjo/scripts/check-lock-guards.sh
cargo test -p kumkaj-khaynjo --release --lib -- --ignored months_of_trading --nocapture
cargo test -p kumkaj-moklanjo --release --lib -- --ignored months_of_trading --nocapture
```

Re-run after **2026-10-27** research Local/LAN catch-up: both packages'
unit/integration tests green (prior printer pass 2026-10-26 still holds);
never-print + lock-guard clean. Research status:
[`RESTAURANT_RESEARCH.md`](../apps/khaynjo/docs/RESTAURANT_RESEARCH.md) §8,
[`COURIER_RESEARCH.md`](../apps/moklanjo/docs/COURIER_RESEARCH.md) §7.

| | Moklanjo | Khaynjo |
|---|---|---|
| Rust source | ~47k+ lines | ~54k+ lines |
| `#[test]`s | ~181 in `src/` | ~229 in `src/` |
| LAN guides | `docs/LAN_QUICK_START.md`, `docs/SERVER_MIGRATION.md` (shared) | same |
| MSI | `dist/Moklanjo-0.1.0.msi` (Quick start) | `dist/Khaynjo-0.1.0.msi` (Quick start) |
| Research Local/LAN | **Have** (§7); Cloud/Net/owner left | **Have** (§8); Cloud/Net/owner left |
| Real hardware | not yet (any platform) | not yet (any platform) |
