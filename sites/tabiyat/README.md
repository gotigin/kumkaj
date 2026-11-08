# tabiyat.shop — Makepad 2 product site

Package `tabiyat-site`. Copied from `sites/kirano` / `sites/khaynjo`, then
diverged (no shared site engine). Story and claims:
[`apps/tabiyat/docs/TABIYAT_WEBSITE.md`](../../apps/tabiyat/docs/TABIYAT_WEBSITE.md);
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md).

Pharmacy **counter** BOS — medicines, barcode/salt, batch/expiry owner
bookkeeping. No Floor, Kitchen, KOT or tables. Domain **tabiyat.shop**.
Never claims Drugs Act compliance on this site.

## Run

```bash
./sites/tabiyat/scripts/run-desktop.sh   # native window
./sites/tabiyat/scripts/run-web.sh       # wasm → http://127.0.0.1:8014
```

Top-bar tiles jump chapters. `#search` (or `#3`) in the URL opens a chapter.

## Story (8 chapters — Phase 1)

**Pharmacy counter** (bill toy) → Counter sell → Barcode & salt → Batch &
expiry → Khata → Extra till → Free forever → Start / download.

The bill toy walks Ready → lines → settle → Paid (`src/counter.rs`).

## Owner switches (`src/main.rs`)

| Switch | Ships | Turns on |
|---|---|---|
| `WHATSAPP_NUMBER` | `None` (**TODO owner**) | "WhatsApp for a demo" opens `wa.me` with "Tabiyat demo — pharmacy, city" |
| `DOWNLOADS_LIVE` | `false` | "Download free" opens `DOWNLOAD_URL` (`https://tabiyat.shop/download`) |
| `LAN_PILOT_PASSED` | `false` | Extra-till / multi-device Wi‑Fi lines |

## Files

| Path | What |
|---|---|
| `src/ui.rs` | Page DSL: Tabiyat teal/lime tokens, chapters |
| `src/main.rs` | Nav, bill toy, CTAs, responsive tiers, tests |
| `src/counter.rs` | Bill step order, example lines, rupees helper |
| `resources/fonts/` | Baloo 2, IBM Plex Sans, Liberation Mono, Noto Gujarati / Devanagari — SIL OFL |

Bill, search card, batch slip and khata card are illustrations marked
**Example** — not app screens.
