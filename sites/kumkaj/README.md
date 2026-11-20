# kumkaj.com — Makepad 2 brand site

Package `kumkaj-site`. A fullpage Makepad 2 app: wasm in the browser, the
same crate native on desktop (preview and offline presenter). Plan and
rules: [`docs/WEBSITES.md`](../../docs/WEBSITES.md). Domain **kumkaj.com**
(registered; Firebase project is an owner step before first deploy).

## Run

```bash
./sites/kumkaj/scripts/run-desktop.sh   # native window
./sites/kumkaj/scripts/run-web.sh       # debug wasm → http://127.0.0.1:8010

# Production-shaped tree (wasm + /privacy /terms + OG) before Firebase
./sites/kumkaj/scripts/preview-web.sh   # http://127.0.0.1:8010
./sites/kumkaj/scripts/deploy-firebase.sh --dry-run   # project: the default in .firebaserc
```

One scrolling page (Khaynjo/Wadhu maturity): drag-select across blocks,
Ctrl/Cmd+A, Copy / Select all menu; dots and `#trade` jump to sections.
Footer opens `/privacy` and `/terms`.

## Story (scroll sections)

The promise (offline-first BOS demo card) → Built for your trade
(Moklanjo / Khaynjo / Kirano / Tabiyat) → Your books stay with you →
**Try it offline** → Free forever on your devices → Speaks your staff's
language → Who makes it (Trademark → Company → Brand → Product) →
Privacy / Terms footer.

## Files

| Path | What |
|---|---|
| `src/ui.rs` | The whole page: tokens, type, components, chapters (`script_mod!`) |
| `src/main.rs` | Chrome, toy, links, responsive tiers, owner switches, tests |
| `PRIVACY.md` | Brand privacy — presented at `/privacy` by preview/deploy |
| `static/terms/` | Terms page for the crawl shell |
| `resources/fonts/` | Fraunces (display), IBM Plex Sans, Noto scripts — SIL OFL |

## Owner switches (`src/main.rs`)

- `LAN_PILOT_PASSED` — shows "extra counters and tablets join over your
  own Wi‑Fi" once the real-hardware LAN pilot passes. Ships `false`.

## Links

Product cards open the public domains. A page served from 127.0.0.1 /
localhost opens the local product sites instead (:8011 / :8012 / :8013 /
:8014); natively, `SITE_LINKS=local` does the same.

## Native preview

`SITE_PREVIEW=390x844:4 cargo run -p kumkaj-site` opens a 390×844 window on
chapter 4 — the loop agents use to check phone, tablet and desk layouts.
