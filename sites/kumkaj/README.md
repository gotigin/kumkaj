# kumkaj.com — Makepad 2 brand site

Package `kumkaj-site`. A fullpage Makepad 2 app: wasm in the browser, the
same crate native on desktop (preview and offline presenter). Plan and
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md).

## Run

```bash
./sites/kumkaj/scripts/run-desktop.sh   # native window
./sites/kumkaj/scripts/run-web.sh       # wasm → http://127.0.0.1:8010
```

Scroll, swipe up, tap, or press ↓ / → / PgDn / Space for the next chapter;
↑ / ← / PgUp go back; Home / End jump; the top-bar dots jump to a chapter.
`http://127.0.0.1:8010/#trade` (or `#2`) opens a chapter directly.

## Story (7 chapters)

The promise (with a "this counter, right now" status card) → Built for
your trade (Moklanjo / Khaynjo choice cards) → Your books live in your
shop → **Try it offline** (cut-the-internet toy) → Free forever on your
devices → Speaks your staff's language (Khaynjo's real "Food ready" in
five scripts) → Made by Gotigin (brand map, the choice again).

## Files

| Path | What |
|---|---|
| `src/ui.rs` | The whole page: tokens, type, components, chapters (`script_mod!`) |
| `src/main.rs` | Chrome, toy, links, responsive tiers, owner switches, tests |
| `resources/fonts/` | Fraunces (display), IBM Plex Sans, Noto Devanagari / Gujarati / Tamil / Bengali / Arabic — all SIL OFL (`OFL-*.txt`) |

## Owner switches (`src/main.rs`)

- `LAN_PILOT_PASSED` — shows "extra counters and tablets join over your
  own Wi‑Fi" once the real-hardware LAN pilot passes. Ships `false`.

## Links

Product cards open `https://moklanjo.com/` / `https://khaynjo.com/`. A page
served from 127.0.0.1 / localhost opens the local product sites instead
(:8011 / :8012); natively, `SITE_LINKS=local` does the same.

## Native preview

`SITE_PREVIEW=390x844:4 cargo run -p kumkaj-site` opens a 390×844 window on
chapter 4 — the loop agents use to check phone, tablet and desk layouts.
