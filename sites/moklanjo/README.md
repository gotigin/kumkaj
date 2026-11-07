# moklanjo.com — Makepad 2 product site

Package `moklanjo-site`. Copied from `sites/kumkaj`, then diverged (no
shared site engine). Story and claims:
[`apps/moklanjo/docs/MOKLANJO_WEBSITE.md`](../../apps/moklanjo/docs/MOKLANJO_WEBSITE.md);
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md).

## Run

```bash
./sites/moklanjo/scripts/run-desktop.sh   # native window
./sites/moklanjo/scripts/run-web.sh       # wasm → http://127.0.0.1:8011
```

The tracking rail under the top bar is the chapter nav: beads behind the
parcel turn yellow, the current one orange. `#weigh` (or `#2`) in the URL
opens that chapter.

## Story (9 chapters — "one parcel, from counter to settled")

Walk-in → **Weigh it right** (volumetric toy) → Price it → Label it → Hand
over → Track and prove → Money comes home → Internet down → Start.

The weight toy uses the app's rules, copied (not imported) into
`src/weight.rs`: L × B × H ÷ 5000 domestic / ÷ 6000 international, the
higher of actual and volumetric, rounded up to 0.5 kg. Its tests pin the
app's own worked examples from `apps/moklanjo/src/booking.rs`.

## Owner switches (`src/main.rs`)

| Switch | Ships | Turns on |
|---|---|---|
| `WHATSAPP_NUMBER` | `None` (**TODO owner**) | "WhatsApp for a demo" opens `wa.me` with "Moklanjo demo — city, parcels a day" |
| `DOWNLOADS_LIVE` | `false` | "Download free" opens `DOWNLOAD_URL` (after clean-machine installs pass) |
| `LAN_PILOT_PASSED` | `false` | "Second counter and rider phone joined over the office Wi‑Fi" |

Until then the buttons explain themselves in the note under them.

## Files

| Path | What |
|---|---|
| `src/ui.rs` | Page DSL: tokens, Anek / Plex / Mono type, components, chapters |
| `src/main.rs` | Rail, toy wiring, CTAs, responsive tiers, tests |
| `src/weight.rs` | Chargeable weight (copied rules + tests) |
| `resources/fonts/` | Anek Latin (display), IBM Plex Sans, Liberation Mono, Noto Devanagari / Gujarati — SIL OFL |

Rate cards, labels and rider cash on the page are illustrations marked
**Example** — not app screens and not real numbers.
