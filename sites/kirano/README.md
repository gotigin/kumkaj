# kirano.shop — Makepad 2 product site

Package `kirano-site`. Copied from `sites/khaynjo`, then diverged (no
shared site engine). Story and claims:
[`apps/kirano/docs/KIRANO_WEBSITE.md`](../../apps/kirano/docs/KIRANO_WEBSITE.md);
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md).

Grocery / kirana **counter** BOS — Books · Sell · Buy · Items. No Floor,
Kitchen, KOT or tables. Domain **kirano.shop** (not kirano.com).

## Run

```bash
./sites/kirano/scripts/run-desktop.sh   # native window
./sites/kirano/scripts/run-web.sh       # wasm → http://127.0.0.1:8013
```

Top-bar tiles jump chapters. `#sell` (or `#2`) in the URL opens a chapter.

## Story (8 chapters — Phase 1)

**Busy counter** (bill toy) → Counter sell → Khata → Buy & stock → Day
book → Extra till → Free forever → Start / download.

The bill toy walks Ready → lines → settle → Paid (`src/counter.rs`).

## Owner switches (`src/main.rs`)

| Switch | Ships | Turns on |
|---|---|---|
| `WHATSAPP_NUMBER` | `None` (**TODO owner**) | "WhatsApp for a demo" opens `wa.me` with "Kirano demo — shop type, city" |
| `DOWNLOADS_LIVE` | `false` | "Download free" opens `DOWNLOAD_URL` (`https://kirano.shop/download`) |
| `LAN_PILOT_PASSED` | `false` | Extra-till / multi-device Wi‑Fi lines |

## Files

| Path | What |
|---|---|
| `src/ui.rs` | Page DSL: Kirano orange/leaf tokens, chapters |
| `src/main.rs` | Nav, bill toy, CTAs, responsive tiers, tests |
| `src/counter.rs` | Bill step order, example lines, rupees helper |
| `resources/fonts/` | Baloo 2, IBM Plex Sans, Liberation Mono, Noto Gujarati / Devanagari — SIL OFL |

Bill, khata card, purchase slip and drawer are illustrations marked
**Example** — not app screens.
