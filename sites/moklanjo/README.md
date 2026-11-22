# moklanjo.com — Makepad 2 product site (courier operating system)

Package `moklanjo-site`. Copied from `sites/kumkaj`, then diverged (no
shared site engine). Story and claims:
[`apps/moklanjo/docs/MOKLANJO_WEBSITE.md`](../../apps/moklanjo/docs/MOKLANJO_WEBSITE.md);
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md).

## Run

```bash
./sites/moklanjo/scripts/run-desktop.sh   # native window
./sites/moklanjo/scripts/run-web.sh       # debug wasm → http://127.0.0.1:8011

# Production-shaped tree (wasm + /privacy /terms /account + OG) before Firebase
./sites/moklanjo/scripts/preview-web.sh   # http://127.0.0.1:8011
./sites/moklanjo/scripts/deploy-firebase.sh --dry-run
```

Auth: email + Google on `/account` — see `docs/FIREBASE_AUTH_SETUP.md`.

Domain **moklanjo.com** is registered. Firebase Hosting project:

- Project id: **`moklanjo-com`** (display name Moklanjo) — see
  `sites/moklanjo/.firebaserc`
- Canonical public URL: **`https://moklanjo.com`** (and paths under it)
- Do **not** use `moklanjo-com.web.app` / `*.firebaseapp.com` in OAuth
  redirect URIs, Auth authorized domains, Play / store privacy links,
  canonicals, or any callback we publish — attach `moklanjo.com` in
  Firebase Console → Hosting, then use the custom domain everywhere

Privacy source:
[`apps/moklanjo/docs/PRIVACY.md`](../../apps/moklanjo/docs/PRIVACY.md).

The tracking rail under the top bar is the chapter nav: beads behind the
parcel turn yellow, the current one orange. `#weigh` (or `#2`) in the URL
opens that chapter.

## Scroll: free-scroll + rail in sync

Wheel, trackpad, and finger-drag **free-scroll** the `ScrollYView` at the
user’s own speed. The old “one viewport per stroke” page-step was removed
(2026-11-22) — it locked scrolling to a fixed ease.

**What stays:** the horizontal parcel-journey rail (orange fill + beads +
`1 / 9 · …` pill) must track where you are in the story as you scroll.
That is independent of page-steps.

Implementation in `src/main.rs`:

1. **No wheel claim** — `ScrollYView` owns free-scroll. Do not reintroduce
   `handled_y` page-steps.
2. **Rail sync** — on `Event::Scroll` and when Draw sees scroll Y move,
   `request_chrome_sync` → `NextFrame` → `paint_scroll_chrome` /
   `snap_rail_fill` from `journey_progress` (never mutate rail width
   mid-Draw — wasm text smear).
3. **Last chapter (Start)** — `at_page_end` is
   `scroll_pos ≥ max_scroll − 4` **only when** scroll metrics are ready
   (`total`/`visible` real and `max > 4`). Never treat “not laid out yet”
   as end — that filled the whole orange rail on Walk-in (2026-11-22).
   Mid-viewport (~55%) also promotes Start at the bottom. Section tops
   must span ≥ 80 px before progress leaves 0.
4. **Bead taps / deep links** may ease to a chapter; PageUp/PageDown/Space
   jump one viewport **immediately**; arrows nudge 64 px.

### Wasm text ghosting — do not regress

| Cause | Rule |
|---|---|
| Eased rail width via perpetual `NextFrame` | Snap rail width; no rail ease loop |
| `paint_chrome` / `set_text` on beads every anim tick | Mid bead-jump: scroll + `snap_rail_fill` only; full chrome on settle |
| `snap_rail_fill` on every `Event::Draw` | Sync on **NextFrame** after scroll |

Unit tests pin `journey_progress_from` / `active_chapter_from`.

Hard-refresh after a wasm rebuild (`?v=preview-…`); stale tabs can keep an
old binary (`RuntimeError: unreachable`).

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
