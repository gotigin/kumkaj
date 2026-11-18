# Moklanjo — production ready, app and website (tracker)

Owner ask (2026-11-16): "Make moklanjo as production ready too, the way we
made khaynjo but be careful not to include khaynjo specific things … khaynjo
is for restaurants and likes and courier is for courier business and likes
like self setup or franchises … make moklanjo production ready for both app
and website."

Rules for this work:

- **Khaynjo's maturity, Moklanjo's own product.** Port the *engineering*
  (correct printing, previews that do not lie, tests, cert, packaging, legal
  pages, deploy) — never Khaynjo's restaurant concepts (tables, KOTs,
  kitchen, menus, dishes, dine-in, waiters). Moklanjo's users: courier
  counters, franchise / agency counters, own-fleet offices
  (`MOKLANJO.md` § Product shape).
- **Discuss before adding a setting or a screen** (owner rule). A row marked
  *needs owner OK* is proposed, not built.
- An agent continues from the first open row and keeps the rows honest
  (what was verified, how).

## App

| # | Item | Status | Notes |
|---|---|---|---|
| A1 | OS documents at real size on the printer's paper; paper per printer (Find fills it); preview of the job on any printer's paper | **Done 2026-11-16** | Was `lp -o fit-to-page` / a Windows page fitted to fixed margins: an 80 mm receipt printed page-wide. `docs/PRINTER.md` § Paper and preview, § Moklanjo. 18 paper tests; Xvfb checked (A4 driver paper, 101.6 mm roll, 80 mm ESC/POS label cut at the roll's edge). **Reworked 2026-11-17** with the other apps: the job prints the standard way on the printer's own page (no paper named by the app), each printer has a typed *virtual* paper and a read-only *reported* paper, the receipt and the AWB label each have a 3 × 3 place, and Windows printing / USB / Bluetooth use the Windows API with no helper process. Checked on Xvfb at 1440 and 420 px; not tried on a real printer or an Android device |
| A2 | In-app privacy policy (Settings → Privacy) from `docs/PRIVACY.md`, as Khaynjo's `privacy_policy.rs` | Open — *needs owner OK* (a new Settings section) | `PRIVACY.md` written 2026-11-16 for the site and the Play listing; legal details are owner inputs (O2) |
| A3 | Settings → Printers "Where things print": Khaynjo's pattern — each job's printers a checklist dropdown (✓ rows, Find printers row when none), no Add / Up / Down / Remove last / Clear / **Save printers** buttons (formats save themselves) | Open — *needs owner OK* | The old route controls are still always shown. DEV.md § Dropdowns rules 9–10, DESIGN.md "Formats save themselves" |
| A4 | Hard-coded English left in printer screens: "(none yet — tap Find printers)", "this computer's usual printer (blank address)" in `courier_printers_pool_rows` | **Done 2026-11-16** | The paper rows replaced that list; the function and its call are gone (a blank address reads `printer_target_default`, translated) |
| A5 | `docs/DEV.md` § Manual UI smoke listed a Khaynjo step ("4b. Waiter … Floor/Order") in Moklanjo's checklist | **Done 2026-11-16** | Now a cashier check (seed Asha 2222) against MOKLANJO.md § Rights match chrome |
| A6 | Store cert (`./scripts/product-cert.sh moklanjo --tier=store`) | **Not passed** — 2026-11-16 verdict FAIL | fast, ui, visual, resilience and xcomp pass. The Linux package check was skipped because there is no package (A9). The Android launch check needs a device attached. See § Cert runs |
| A7 | Version for the first public release (0.1.0, Play code 1 today; Khaynjo is 1.0.10 / 8) | *Owner decides* | `apps/moklanjo/Cargo.toml` `version`, `[package.metadata.makepad.android]` |
| A8 | Export template (Khaynjo 2026-11-14) | Later — owner: Khaynjo first, then the other three | `APP_MATURITY.md` owner queue |
| A9 | Release packages in `dist/`: Linux, Android (AAB + APK), Windows (MSIX + MSI) | Open — never built | `dist/` holds only Khaynjo's, from 2026-11-15. Needs A7. A signed Play upload needs O1; a Store MSIX needs O5 |
| A10 | Play Store and Microsoft Store requirements, checked item by item against Khaynjo's `PUBLISH_PLAY_STORE.md` / `PUBLISH_WINDOWS_STORE.md` | Open — not reviewed | Moklanjo has no publish docs and no store listing yet. The privacy URL that Play needs is W1 |
| A11 | Settings → Printers "Where things print" at phone width (420 px): the job's name wraps one letter per line and the printer text is cut off at the card's edge | Done 2026-11-17 | The value never shrank, so a long one left the label a few px. `ReportText` now draws such a row with the label above and the value under it, both wrapping (`kv_stacks`, DEV.md bug 7j; all four apps). Seen on the isolated screen at 420 and 1440 px; money rows unchanged. A3's checklist dropdowns would still replace this list |

## Website (moklanjo.com)

| # | Item | Status | Notes |
|---|---|---|---|
| W1 | `/privacy` and `/terms` served with the Makepad site (one deploy tree, as khaynjo.com) | **Done 2026-11-18** (local) | `sites/moklanjo/scripts/{preview-web,deploy-firebase}.sh` present `apps/moklanjo/docs/PRIVACY.md` via pandoc + `static/terms/`. Legacy zola stubs remain only until Phase 3 retire |
| W2 | Deploy pipeline like Khaynjo's: release wasm, cache-busting build id, static shell, `preview-web.sh`, `deploy-firebase.sh --dry-run` | **Done 2026-11-18** (scripts) | Same shape as Khaynjo under `sites/moklanjo/`. Hosting project id still owner input (O3) — domain moklanjo.com is registered |
| W3 | Owner switches: `WHATSAPP_NUMBER` (None), `DOWNLOADS_LIVE` (false), `LAN_PILOT_PASSED` (false) in `sites/moklanjo/src/main.rs` | *Owner inputs* | `docs/WEBSITES.md` § 8 |
| W4 | Crawl shell: OG meta, `<noscript>` chapter text | Open (Phase 3 in `WEBSITES.md`) | |

## Owner-only inputs

| # | Input |
|---|---|
| O1 | Play developer account listing for `com.moklanjo`; Android release keystore (`KUMKAJ_ANDROID_KEYSTORE`) |
| O2 | Gotigin legal / contact / grievance details for `PRIVACY.md` and the site footer |
| O3 | Hosting for moklanjo.com (Firebase project id or other) |
| O4 | WhatsApp Business number for "WhatsApp for a demo" |
| O5 | Windows code signing / Microsoft Store identity (`MearajBhagad.Moklanjo`) |
| O6 | Real-hardware LAN pilot (second counter + rider phone) before the LAN claims go live |

## Cert runs

- 2026-11-16 `--tier=fast`: PASS (259 + 33 + 9 tests).
- 2026-11-16 `--tier=store`, first run: fast PASS, ui PASS, **visual FAIL**
  (`000a_rail_collapsed` AE 52 293 > 50 000). Not a regression: the five
  baselines (pinned 2026-11-14) predate the owner's 2026-11-15 changes —
  unlock lands on the first nav item (Booking, not Day book), the nav's
  default order is dictionary order, Settings is one list of nav items with
  Enable / Disable All — plus the random shop id. Each old / new pair
  checked by eye; the five refreshed from that run's green `ui/peers` shots
  (they match its visual shots within 178 AE).
- 2026-11-16 `--tier=store`, second run (`SUMMARY-20261116-145824`):
  fast, ui, visual, resilience and xcomp PASS. linux **SKIP**: there is no
  `dist/moklanjo-linux`, because it was never built (`CERT_BUILD_LINUX=1`
  builds it). android **FAIL**: no device attached. **Verdict: FAIL.**
