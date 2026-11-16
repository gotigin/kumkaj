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
| A1 | OS documents at real size on the printer's paper; paper per printer (Find fills it); preview of the job on any printer's paper | **Done 2026-11-16** | Was `lp -o fit-to-page` / a Windows page fitted to fixed margins: an 80 mm receipt printed page-wide. `docs/PRINTER.md` § Paper and preview, § Moklanjo. 18 paper tests; Xvfb checked (A4 driver paper, 101.6 mm roll, 80 mm ESC/POS label cut at the roll's edge) |
| A2 | In-app privacy policy (Settings → Privacy) from `docs/PRIVACY.md`, as Khaynjo's `privacy_policy.rs` | Open — *needs owner OK* (a new Settings section) | `PRIVACY.md` written 2026-11-16 for the site and the Play listing; legal details are owner inputs (O2) |
| A3 | Settings → Printers "Where things print": Khaynjo's pattern — each job's printers a checklist dropdown (✓ rows, Find printers row when none), no Add / Up / Down / Remove last / Clear / **Save printers** buttons (formats save themselves) | Open — *needs owner OK* | The old route controls are still always shown. DEV.md § Dropdowns rules 9–10, DESIGN.md "Formats save themselves" |
| A4 | Hard-coded English left in printer screens: "(none yet — tap Find printers)", "this computer's usual printer (blank address)" in `courier_printers_pool_rows` | **Done 2026-11-16** | The paper rows replaced that list; the function and its call are gone (a blank address reads `printer_target_default`, translated) |
| A5 | `docs/DEV.md` § Manual UI smoke listed a Khaynjo step ("4b. Waiter … Floor/Order") in Moklanjo's checklist | **Done 2026-11-16** | Now a cashier check (seed Asha 2222) against MOKLANJO.md § Rights match chrome |
| A6 | Store cert (`./scripts/product-cert.sh moklanjo --tier=store`) | Run 2026-11-16 — see § Cert runs | |
| A7 | Version for the first public release (0.1.0, Play code 1 today; Khaynjo is 1.0.10 / 8) | *Owner decides* | `apps/moklanjo/Cargo.toml` `version`, `[package.metadata.makepad.android]` |
| A8 | Export template (Khaynjo 2026-11-14) | Later — owner: Khaynjo first, then the other three | `APP_MATURITY.md` owner queue |

## Website (moklanjo.com)

| # | Item | Status | Notes |
|---|---|---|---|
| W1 | `/privacy` and `/terms` served with the Makepad site (one deploy tree, as khaynjo.com) | Open | Today only the legacy zola stubs (`apps/moklanjo/web/content/privacy.md`, `terms.md`, with `TODO_OWNER` lines). Source of truth for privacy: `apps/moklanjo/docs/PRIVACY.md` |
| W2 | Deploy pipeline like Khaynjo's: release wasm, cache-busting build id, static shell, `preview-web.sh`, `deploy-firebase.sh --dry-run` | Open | Khaynjo: `sites/khaynjo/scripts/{deploy-firebase,preview-web,patch-web-shell}.sh`, `firebase.json`. Hosting project is an owner input (O3) |
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
