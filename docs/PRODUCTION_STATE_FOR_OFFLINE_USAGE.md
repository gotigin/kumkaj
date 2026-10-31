# Production state for offline (LAN) use — Moklanjo & Khaynjo

> **Home:** root `docs/` (both products). Product briefs link here:
> [`MOKLANJO.md`](../apps/moklanjo/docs/MOKLANJO.md),
> [`KHAYNJO.md`](../apps/khaynjo/docs/KHAYNJO.md).

Updated **2026-10-31** (v5): a LAN hardening and correctness pass (§1,
first table) — concurrent writes, transient errors, version mismatch,
restore on Windows, Moklanjo self-booking money, Khaynjo floor control —
and the test suites green again (on 2026-10-29 v4 said green while 7
Khaynjo and 3 Moklanjo tests failed). Research Local/LAN catch-up for both
apps is complete (§4). Every line below was checked against the code, not
copied from an older assessment.

**Bottom line:** product code for offline/LAN use (including research
parity that does not need Kumkaj Cloud) is ready for a supervised pilot
(a counter device + two extras, any mix of supported platforms). What
blocks a non-technical roll-out is no longer Local/LAN feature gaps: it
is the **real-hardware run** and **installing the built packages on clean
machines** (§3), which only the owner can do.

---

## 1. Fixed / completed in this pass

**2026-10-31 (v5) — LAN hardening.** Found by reading the LAN path end to
end and by the suites (which were red); every fix has a test.

| Item | What changed | Where |
|---|---|---|
| **Two tablets sending to one table lost KOTs** (bug, Khaynjo) | A device's write ran as many separate statements: two waiters' Send KOT on the same table interleaved — one got "Nothing new to send" (and dropped it as refused) while the other's KOT carried both waiters' lines. The 8-devices test showed it. Every device write is now **one transaction**, reply record included; `send_kot` and `write_sale` make their checks inside their own unit. | both apps `net.rs` (`answer_inner`), `store.rs` (`Tx`, `begin`, `in_transaction` nests as a savepoint) |
| **Two tills could get the same AWB** (bug, Moklanjo) | AWB pool allocation read `next_n` then wrote `next_n + 1` outside any transaction (and stock was checked before the sale's transaction). Now inside the booking's unit. | Moklanjo `store.rs` `write_sale` |
| **"Database is locked" was a permanent refusal** (bug) | Any server error — including a busy database or a full disk — came back 400, so the tablet moved the KOT / payment to "not accepted" for good, and the refusal was even recorded as the answer for retries. Now "not now" errors are **503 `retry`**: not recorded, the device keeps the write and sends it again. A panic in a request is a 500, never a hang-up that blocks the device's queue. | both apps `net.rs` (`answer`, `is_transient`, `lan_error`) |
| **Every read took the write lock** (bug) | Opening the books (once per LAN request — `/sync`, `/wait`, discovery) ran the migration's no-op `UPDATE`s, and `list_tables` (every snapshot, every floor refresh) ran an `UPDATE` each time: each device's read queued behind any write and failed after 5 s. Books are now migrated once per schema (`PRAGMA user_version`); heals look first and write only when needed. | both apps `store.rs` (`open_path`, `lan_device_for_token`), Khaynjo `list_tables` |
| **Mixed app versions looked "offline" for ever** (gap) | No protocol version: a newer Server's snapshot failed to parse on an older tablet and read as "no reply". `LAN_PROTOCOL` now travels on every request and reply; a mismatch is 426 **"Update the app — this device and the shop's server have different versions"** in the header (translated, 29 languages), waiting writes kept. | both apps `net.rs`, `phone.rs` (`needs_update`), header |
| **Restore failed on Windows while serving** (bug) | Restore renamed a file over the live books, but the LAN watcher keeps them open: on Windows the rename fails; on Linux the watcher kept serving the old books. Restore now goes into the live database through SQLite's backup API, one step, one transaction. | both apps `store.rs` `restore_from_to_path` |
| **Open `/book` page wrote fake cash** (bug, Moklanjo) | Anyone on the office Wi‑Fi could post the self-booking form: it booked as the Owner, **paid in cash**, at whichever rate came first, ignoring weight. Now the company names itself (required), picks a service, is priced by that rate's slab like the desk, and is **billed on account**; audited `lan.self_book`. | Moklanjo `net.rs` `lan_book_*`, `store_courier_ops.rs` `lan_self_book` |
| **Tables could not be removed** (gap, Khaynjo) | A fresh shop seeds T1..T24 and the UI could only add tables — Rajdhani's D1–D20 + P1–P20 floor would carry 24 unused tables for ever (and deleting them in code brought them back on the next start). Floor → ADD TABLES → **Remove these tables** (`T1-T24` or a name; a table in use is kept and named); seeded once; a table with an open order cannot be deleted; a tablet drops tables the Server no longer has. | Khaynjo `store.rs` (`remove_tables_named`, `ensure_seed_data`, `replace_tables_from_sync`), floor UI |
| **Guest pages** (Khaynjo) | The table-QR / kiosk endpoints (no pairing) could add raw materials and sold-out dishes, in any number; form decoding could panic on non-ASCII and garbled Gujarati. Now only what the guest menu shows, ≤ 50 per request, all or nothing; UTF-8-safe decoding; menu JSON safe inside `<script>`. Pages listed for owners in `LAN_QUICK_START.md` §6. | Khaynjo `net.rs`, `store_ops.rs` |
| **Unbounded threads** | One thread per connection with no cap: past 64 at once the Server now answers 503 at once. | both apps `net.rs` `serve` |
| **Tests and checks were red / blind** | 10 failing tests fixed (1 real race, 9 stale after deliberate changes — each checked against the code); 865 + 277 missing translations added (Moklanjo's 2026-10-29 AWB-pool / TDS / consignor labels shipped untranslated); the locale test now lists every gap at once. `check-lock-guards.sh` scanned only `main.rs`, which the shell split emptied — it now scans `src/app/` and `src/widgets/`, and Moklanjo has its own copy. KOT print capture in tests now behaves like a printer took the job. | both apps |

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
2. **Install the built packages on clean machines.** Windows rebuilt on
   **2026-10-31** with the v5 fixes: `dist/Khaynjo-0.1.0.msi`,
   `dist/Moklanjo-0.1.0.msi`, folder packages under `dist/khaynjo-windows/`
   / `dist/moklanjo-windows/`. The **Android APKs in `dist/` are from
   2026-10-26 and predate v5** — rebuild them before the pilot
   (`apps/khaynjo/scripts/package-android.sh`, Moklanjo's likewise; they
   patch the vendored Makepad, so run them on a settled tree). **Every device
   in the shop must run the same build** — v5 adds the LAN protocol check,
   and an older tablet would be told to update. Not yet installed on a
   clean machine of any supported platform.
3. **Android release signing + Play listing** (Moklanjo and Khaynjo), web
   deploy (kumkaj.com / khaynjo.com). iOS/macOS stay scaffold-only until a
   Mac exists.

---

## 4. Still open in code

**Worth doing next**

| Item | Why / how |
|---|---|
| *(none in Local/LAN product code)* | Research catch-up for both apps is **Have** for everything that does not need Kumkaj Cloud — see [`RESTAURANT_RESEARCH.md`](../apps/khaynjo/docs/RESTAURANT_RESEARCH.md) §8 and [`COURIER_RESEARCH.md`](../apps/moklanjo/docs/COURIER_RESEARCH.md) §7 (C Trunk Local gaps closed **2026-10-29**: AWB pools, bulk status, TDS/cheque settle, bill chase, stamp/QR, GST month CSV, cargo pieces, mix report; Khaynjo stamp/QR + aggregator drop-file). **Emergency unlock PIN** shipped 2026-10-27. Mouse cursor state patch ships with Makepad setup. Remaining product work is Cloud/Net connectors (optional stubs only), polish (coupons / variation matrix), or owner actions below. |

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

## 5. Measured (2026-10-31, this machine, release build)

Opt-in test `months_of_trading_stay_quick` — 9,000 bills over 30 days
(300 today), 1,000 customers, 300 items:

| Step | Khaynjo | Moklanjo |
|---|---|---|
| Snapshot built | 30 ms | 17 ms |
| Snapshot size (before gzip) | 484 KB (limit 600) | 398 KB |
| `/sync` answered | 31 ms | 17 ms |
| Today's totals | 0.3 ms | 0.3 ms |
| Day close | 14 ms | 9 ms |
| Day report | 1 ms | — |
| Backup | 73 ms | 75 ms |

(2026-10-25: Khaynjo snapshot 417 KB / 18 ms, Moklanjo 330 KB / 16 ms —
the growth is features added since, not this pass.)

Not measured: low-end Android memory and battery while serving or syncing
all day — do it during the hardware run.

---

## 6. Re-check

```bash
cargo test -p kumkaj-khaynjo --no-fail-fast   # 247 lib + 11 bin (2026-10-31)
cargo test -p kumkaj-moklanjo --no-fail-fast  # 210 lib + 5 bin
apps/khaynjo/scripts/check-tests-never-print.sh
apps/moklanjo/scripts/check-tests-never-print.sh
apps/khaynjo/scripts/check-lock-guards.sh
apps/moklanjo/scripts/check-lock-guards.sh
cargo test -p kumkaj-khaynjo --release --lib -- --ignored months_of_trading --nocapture
cargo test -p kumkaj-moklanjo --release --lib -- --ignored months_of_trading --nocapture
```

Re-run **2026-10-31**: both packages green with `--no-fail-fast` (without
it cargo stops at the first red binary and the rest never run — that is
how the bin tests went unseen); never-print and lock-guard clean in both
apps. Research status:
[`RESTAURANT_RESEARCH.md`](../apps/khaynjo/docs/RESTAURANT_RESEARCH.md) §8,
[`COURIER_RESEARCH.md`](../apps/moklanjo/docs/COURIER_RESEARCH.md) §7.

| | Moklanjo | Khaynjo |
|---|---|---|
| Rust source | ~62k lines | ~72k lines |
| Tests passing | 210 lib + 5 bin | 247 lib + 11 bin |
| LAN guides | `docs/LAN_QUICK_START.md`, `docs/SERVER_MIGRATION.md` (shared) | same |
| MSI | `dist/Moklanjo-0.1.0.msi` (Quick start) | `dist/Khaynjo-0.1.0.msi` (Quick start) |
| Research Local/LAN | **Have** (§7); Cloud/Net/owner left | **Have** (§8); Cloud/Net/owner left |
| Real hardware | not yet (any platform) | not yet (any platform) |
