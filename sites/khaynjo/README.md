# khaynjo.com — Makepad 2 product site

Package `khaynjo-site`. Copied from `sites/kumkaj`, then diverged (no
shared site engine). Story and claims:
[`apps/khaynjo/docs/KHAYNJO_WEBSITE.md`](../../apps/khaynjo/docs/KHAYNJO_WEBSITE.md);
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md).

## Run

```bash
./sites/khaynjo/scripts/run-desktop.sh   # native window
./sites/khaynjo/scripts/run-web.sh       # wasm → http://127.0.0.1:8012
```

The small table tiles in the top bar are the chapter nav, each in the
visit colour of its chapter. `#kot` (or `#3`) in the URL opens a chapter.

## Story (10 chapters — "one table, one service")

**Lunch rush** (floor toy) → Order taken → The KOT → Kitchen in step →
Service changes → Settle → Close the day → Your menu → Internet down →
Start.

The floor toy uses Khaynjo's own visit words and tile colours, copied into
`src/floor.rs` (Free → Occupied → In kitchen → Food ready → Served → Bill
printed → Paid → Free). As in the app, a table frees only once it is paid.

## Owner switches (`src/main.rs`)

| Switch | Ships | Turns on |
|---|---|---|
| `WHATSAPP_NUMBER` | `None` (**TODO owner**) | "WhatsApp for a demo" opens `wa.me` with "Khaynjo demo — restaurant type, tables, city" |
| `DOWNLOADS_LIVE` | `false` | "Download free" opens `DOWNLOAD_URL` (after clean-machine installs pass) |
| `LAN_PILOT_PASSED` | `false` | "Prints itself" (KOT auto-print) and the tablets + kitchen screen on Wi‑Fi line |

## Files

| Path | What |
|---|---|
| `src/ui.rs` | Page DSL: tokens, Baloo / Plex / Mono type, components, chapters |
| `src/main.rs` | Nav tiles, floor toy, CTAs, responsive tiers, tests |
| `src/floor.rs` | Visit order, tile words, meta line (copied + tests) |
| `resources/fonts/` | Baloo 2 (display), IBM Plex Sans, Liberation Mono, Noto Gujarati / Devanagari — SIL OFL |

The order card, KOT, kitchen columns, bill and drawer are illustrations
marked **Example** — not app screens; no client names, menus or staff.
