# moklanjo.com — Makepad 2 product site (courier operating system)

Package `moklanjo-site`. Copied from `sites/kumkaj`, then diverged (no
shared site engine). Story and claims:
[`apps/moklanjo/docs/MOKLANJO_WEBSITE.md`](../../apps/moklanjo/docs/MOKLANJO_WEBSITE.md);
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md).

## Run

```bash
./sites/moklanjo/scripts/run-desktop.sh   # native window
./sites/moklanjo/scripts/run-web.sh       # debug wasm → http://127.0.0.1:8011

# Production-shaped tree (wasm + /privacy /terms + OG) before Firebase
./sites/moklanjo/scripts/preview-web.sh   # http://127.0.0.1:8011
./sites/moklanjo/scripts/deploy-firebase.sh --dry-run   # needs Firebase project
```

Domain **moklanjo.com** is registered. Firebase project id is an owner
step (`FIREBASE_PROJECT` or `sites/moklanjo/.firebaserc`). Privacy source:
[`apps/moklanjo/docs/PRIVACY.md`](../../apps/moklanjo/docs/PRIVACY.md).

The tracking rail under the top bar is the chapter nav: beads behind the
parcel turn yellow, the current one orange. `#weigh` (or `#2`) in the URL
opens that chapter.

## Scroll: viewport page-steps (not chapter snap)

Wheel / trackpad / PageUp·PageDown / Space do **not** free-scroll the
`ScrollYView`. A small intentional stroke eases about **one viewport**
(~92% of visible height, ~0.5 s ease-out cubic). The orange rail fill
moves with that motion. Arrow keys still nudge 64 px for fine reading;
finger-drag on the page still free-scrolls (escape hatch). Bead taps
animate to that chapter.

Implementation lives in `src/main.rs`:

1. **Claim the wheel before the scroll view** — `handle_wheel_page` runs
   on `Event::Scroll` *before* `ui.handle_event`, sets `handled_y`, and
   accumulates delta (threshold 28). Ignore `ScrollPhase::Momentum` so a
   flick does not race several pages. Web usually sends `phase: None`.
2. **Time-based page anim** — `scroll_anim_from` / `scroll_anim_to` /
   `scroll_anim_t0` + `NextFrame` (~0.52 s, ease-out cubic). Not an
   asymptotic lerp that crawls forever at the end.
3. **Last chapter (Start)** — Start + footer are shorter than the
   viewport, so Start’s top never crosses the sticky spy mark. 
   `at_page_end` / mid-viewport promote force chapter 9 when the footer
   is flush or Start is well into view. Without that, Offline stays lit
   and Start stays hollow at the bottom.

### Wasm text ghosting — do not regress

On Makepad wasm, **mutating layout or label text every animation /
Draw frame** leaves previous glyphs visible for a beat (smear / trail).
We hit this when:

| Cause | Why it smears | Rule |
|---|---|---|
| Eased **rail width** via perpetual `NextFrame` | `walk.width` thrash + full redraw mid-scroll | Snap rail width; no rail ease loop |
| `paint_chrome` / `set_text` on beads **every anim tick** | Glyphs redrawn while scroll_pos is moving | Mid-flight: `set_scroll_pos` + `snap_rail_fill` only; full `paint_chrome` on settle (and when chapter flips) |
| `paint_scroll_chrome` / `snap_rail_fill` on **every `Event::Draw`** | Mid-draw layout mutation | Chrome updates from Scroll / anim tick / chapter change — not blanket Draw |
| Slow asymptotic scroll lerp (many tiny frames) | More partial paints | Prefer timed ease (~0.5 s), then stop |

**Safe mid-anim:** `page.set_scroll_pos` + `snap_rail_fill` + redraw
`page` / `track_row`. **On complete:** `paint_scroll_chrome` (beads +
step pill). Do not reintroduce continuous `tick_rail_fill` easing or
label rewrites inside `tick_scroll_anim`.

Hard-refresh the preview after a wasm rebuild (`?v=preview-…` in
`index.html`); an old tab can keep a previous build and even hit
`RuntimeError: unreachable`.

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
