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

## Deploy (khaynjo.com — Firebase)

One tree: Makepad wasm + `/privacy` `/terms` `/v1/catalogue/` + Analytics.

```bash
./sites/khaynjo/scripts/deploy-firebase.sh
./sites/khaynjo/scripts/deploy-firebase.sh --dry-run
```

## Local preview (before deploy)

```bash
# Fast native window (layout/copy; same crate as the site)
./sites/khaynjo/scripts/run-desktop.sh

# Debug wasm with hot rebuild — http://127.0.0.1:8012
./sites/khaynjo/scripts/run-web.sh

# Production-shaped release wasm + static shell (what deploy ships)
# http://127.0.0.1:8012 — use this to catch cache/layout bugs without Firebase
./sites/khaynjo/scripts/preview-web.sh
```

Deploys cache-bust `khaynjo-site.wasm` / `web_gl.js` with a build id in
`index.html` so browsers pick up new builds without a hard refresh.

Static overlays live in `static/` (catalogue under `static/v1/catalogue/`,
marketing stills under `static/media/`). Banners and the app icon come from
`apps/khaynjo/assets/` (`KhaynjoBanner-*.png`, `khaynjo-app-icon-512x512.png`)
into `resources/media/` (webp) and `static/media/`. Publish the public dish
shelf + starters with `./apps/khaynjo/scripts/publish-catalogue.sh`. Chapter
dish photos and screenshots also ship inside the wasm package via
`resources/dishes/` and `resources/media/`. Optional AI atmosphere video:
drop a `.webm` under `static/media/` (does not replace the Makepad story).
**Download CTA** opens the live Play listing:
https://play.google.com/store/apps/details?id=com.khaynjo
`/privacy` presents [`apps/khaynjo/docs/PRIVACY.md`](../../apps/khaynjo/docs/PRIVACY.md)
as **formatted** HTML (pandoc); the app embeds the same file under
Settings → Privacy. Firebase project: `khaynjo-2b2a2` (`.firebaserc`).

The small table tiles in the top bar are the chapter nav, each in the
visit colour of its chapter. `#kot` (or `#3`) in the URL opens a chapter.

## Page (one scroll, 2026-11-12 redesign)

Sticky top bar (section links ≥ 1000 px, gold **Get it free**) over bands:
**hero** (CTAs, trust lines, floor toy) → **how** (seat → KOT → kitchen →
settle → day close, Example props) → **features** → **menus** (catalogue
templates + 199-country world shelf) → **offline** → **pricing** → **faq** →
**closing** + footer. Deep links `#how` … `#faq`, `#start`; old chapter
links (`#kot`, `#3`) still land. `SITE_PREVIEW=390x844:menus` previews a
section natively.

The floor toy uses Khaynjo's own visit words and tile colours, copied into
`src/floor.rs` (Free → Occupied → In kitchen → Food ready → Served → Bill
printed → Paid → Free). As in the app, a table frees only once it is paid.

## World shelf (chapter 8)

```bash
./sites/khaynjo/scripts/build-world-shelf.sh   # from local/khaynjo/world-food-research
cargo test -p khaynjo-site                      # 199 rows, photos exist, no orphans
```

Photos load per page; reviewed rejects and sources are in
`resources/dishes/world/SOURCES.md`. Details: `apps/khaynjo/docs/KHAYNJO_WEBSITE.md` §7.

**Markdown hides nothing:** vendored Makepad `Markdown` ignores `set_visible`.
Any `Small` / `Body` / `Lede` that a layout rule hides sits inside a
`View` carrying the id (`paint_details` logs a Markdown hide target).

## Owner switches (`src/main.rs`)

| Switch | Ships | Turns on |
|---|---|---|
| `WHATSAPP_NUMBER` | `None` (**TODO owner**) | "WhatsApp for a demo" opens `wa.me` with "Khaynjo demo — restaurant type, tables, city" |
| `DOWNLOADS_LIVE` | `true` | "Get it on Google Play" → Play listing `com.khaynjo` |
| `LAN_PILOT_PASSED` | `false` | "Prints itself" (KOT auto-print) and the tablets + kitchen screen on Wi‑Fi line |

## Files

| Path | What |
|---|---|
| `src/ui.rs` | Page DSL: tokens, Baloo / Plex / Mono type, components, chapters |
| `src/main.rs` | Nav tiles, floor toy, CTAs, responsive tiers, tests |
| `src/floor.rs` | Visit order, tile words, meta line (copied + tests) |
| `src/world_shelf.rs` | **Generated** world shelf table (199 countries) — do not edit; run `scripts/build-world-shelf.sh` |
| `resources/icons/` | The app's own sidebar icons (copied from `apps/khaynjo/resources/nav/`) for the feature cards |
| `resources/dishes/world/` | Generated 600 × 400 WebP per country with a usable PD / CC0 photo + `SOURCES.md` |
| `resources/fonts/` | Baloo 2 (display), IBM Plex Sans, Liberation Mono, Noto (Gujarati, Devanagari, Tamil, Bengali, Arabic, Hebrew, Thai; chapter-8 WorldFont: Armenian, Ethiopic, Georgian, Khmer, Lao, Myanmar, Sinhala, Thaana, Tibetan, 12 KB CJK subset) — SIL OFL (`OFL-*.txt`) |

The order card, KOT, kitchen columns, bill and drawer are illustrations
marked **Example** — not app screens; no client names, menus or staff.
