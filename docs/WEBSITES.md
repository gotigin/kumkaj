# Kumkaj websites — Makepad 2 fullpage apps

> **Status:** Phase 1 live in-tree as a local demo (2026-11-07) — three
> Makepad 2 site crates, each running native and as wasm in the browser:
> `sites/kumkaj` (`kumkaj-site`, 7 chapters), `sites/moklanjo`
> (`moklanjo-site`, 9 chapters + chargeable-weight toy), `sites/khaynjo`
> (`khaynjo-site`, 10 chapters + floor toy). Not deployed anywhere;
> domains, hosting, WhatsApp numbers and download links are owner steps
> (§8). Next: Phase 2 3D chapters and the Phase 3 crawl shell (§7).
> **Scope:** Kumkaj-wide approach for kumkaj.com, moklanjo.com and
> khaynjo.com. Product stories live with each product:
> [`MOKLANJO_WEBSITE.md`](../apps/moklanjo/docs/MOKLANJO_WEBSITE.md),
> [`KHAYNJO_WEBSITE.md`](../apps/khaynjo/docs/KHAYNJO_WEBSITE.md).
> **Architecture:** each site **is** a Makepad 2 app (wasm in the
> browser, the same crate native on desktop) — the whole page, not a
> canvas inside an HTML page.

------------------------------------------------------------------------

## 1. Three sites, three jobs

| Site | Who arrives | Its job | Main action |
|---|---|---|---|
| **kumkaj.com** | Owners comparing systems, partners, press; people looking up Gotigin | Make the promise felt — *the counter keeps working when the internet stops; free forever on your own devices; built for your trade* — then send each visitor to their trade | **Choose your trade** → moklanjo.com / khaynjo.com / kirano.shop / tabiyat.shop |
| **moklanjo.com** | Courier / logistics agency owners, franchise counters, own-fleet operators | Show a **courier desk**, not a till: book → label → hand over → track → settle | **Download free** · **WhatsApp for a demo** |
| **khaynjo.com** | Restaurant, café, QSR, cloud-kitchen, bakery owners and managers | Show a **service running** on the floor and in the kitchen, internet or not | **Download free** · **WhatsApp for a demo** |
| **kirano.shop** | Kirana / grocery / neighbourhood shop owners | Show a **grocery counter**: sell → khata → stock → day book | **Download free** · **WhatsApp for a demo** |
| **tabiyat.shop** | Chemist / pharmacy counter owners | Show a **pharmacy counter**: medicine → batch/expiry → bill (owner Rx flags, not fake compliance) | **Download free** · **WhatsApp for a demo** |

Rules (vision §1, §21, §24, §31):

1. **Design per product.** Sites do not share a look, layout or motion
   language. Palette comes from each app's icon / `DESIGN.md` (§4).
2. **No Kumkaj core.** Each site crate owns its code. Copy a sibling
   site to start a new one, then diverge. Never extract a shared
   "website engine" — the three crates today share only the workspace
   and `vendor/makepad`.
3. **Claim only what ships.** Feature sentences cite Ship / Have / Done
   rows in the product docs (claim ids sit in comments next to each
   chapter). Planned work is labelled *planned*. A claim with a gate
   (`lan-pilot`) is in the page but hidden behind an owner switch (§8).
4. **Free forever locally; cloud by usage** is the only pricing message.
5. **Gotigin** owns the brand. Footer line:
   "*Moklanjo / Khaynjo is a Kumkaj product by Gotigin.*" No legal
   entity name, address or grievance contact until the owner supplies
   them (§8).
6. **Name no competitor**, carrier or aggregator in copy or visuals;
   describe our own mechanics.
7. **Examples say so.** Illustrative props (rate cards, labels, tickets,
   bills, drawers) carry an *Example* tag; they are not app screens and
   carry no client names, menus, staff or real numbers. Real app
   captures replace them when device screens arrive (Phase 2).

Brand map: **Gotigin** (company) → **Kumkaj** (BOS brand) →
**Moklanjo** / **Khaynjo** (concrete products).

------------------------------------------------------------------------

## 2. Architecture — Makepad first

**Decision:** the public site is a **fullpage Makepad 2 wasm app**,
built and served the Makepad way (cargo-makepad from
`vendor/makepad-full`, see the run scripts):

```bash
cargo makepad wasm --no-threads run -p kumkaj-site     # :8010 via run-web.sh
```

The same crate runs native (`cargo run -p kumkaj-site`) for fast
preview loops and as an offline presenter. Chapters are a vertical
`SlidesView`; 3D chapters (PBR props, glTF, 3D text) grow inside the
same app later — not as a separate HTML page that hosts a canvas.

**khaynjo.com is the exception (2026-11-12):** one scrolling page
(`ScrollYView` bands + sticky top bar, section deep links) instead of a
`SlidesView` deck — a product page with templates, pricing and FAQ outgrew
"one chapter per screen" (phones and short laptops had to hide content).
Rows still switch side-by-side / stacked by content width (`ROWS`, `PROPS`);
nothing is clipped any more, so `DETAIL_*` hide lists are gone there.
Sections are measured from the first band when scrolling to them (rects of
one draw pass), so a deep link lands exactly once layout settles.

```text
Browser / desktop
  └─ Makepad Window (fullpage)
       ├─ top bar: brand + chapter nav (dots / tracking rail / table tiles)
       └─ SlidesView (axis: Vertical) — story chapters
            ├─ copy, cards, toys (native Makepad widgets + small shaders)
            └─ later: 3D scene chapters (draw_pbr, glTF, draw_text_3d)
```

### Why Makepad-first

- One stack with the products: same widgets, text shaping (Gujarati,
  Devanagari, Arabic), motion and agent edit loop the apps already use.
- Presenter mode is free: arrow keys / PgDn / clicker flip chapters; the
  same crate is the sales deck on Windows / Linux / Android.
- "The Makepad way" for web is `cargo makepad wasm` — fullpage by
  default for a windowed app.

### What still needs a thin HTML shell (Phase 3, not the product)

Search engines, OG/WhatsApp link previews and Play's website check want
crawlable text and a privacy URL. That is a **thin static wrapper around
the wasm build** (title, meta, `/privacy` and `/terms`, canonical links,
a `<noscript>` copy of each chapter's words) — not the site
architecture. Until then the legal pages for the Play listing come from
the legacy static stubs under `web/kumkaj/`, `apps/*/web/` (built with
`scripts/build-site.sh`); nothing else is built there any more.

### Threads

Build with **`--no-threads`** so ordinary hosting does not need
COOP/COEP / `SharedArrayBuffer`. Revisit only if a chapter truly needs
workers.

### Vendor patches the sites rely on

`vendor/makepad` is gitignored and rebuilt by `scripts/setup-makepad.sh`
from `patches/makepad/`. The sites need three patches (tracked since
2026-11-07; before that they lived only in the vendor tree and would
have vanished on the next setup):

| Patch | What |
|---|---|
| `slides_view_story_axis` | `SlidesView` `axis: Vertical`; tap / swipe / wheel flip one chapter (OS momentum ignored); presenter keys ↓ → PgDn Space, ↑ ← PgUp, Home / End; `goal_slide()` / `slide_count()` |
| `web_init_window_order` | wasm `ToWasmInit` runs `Event::Startup` (which creates window 0) **before** applying the window geometry — without it the canvas sticks on "Loading.." with an IdPool panic |
| `web_aux_buttons_stay` | mouse side buttons stay in-app; canvas context menu is Copy / Select all; PageUp–arrows/Space not stolen by the shell |
| `widget_steals_document_selection` | only Button / TextInput / HtmlLink block document select (Labels/Images do not) |
| `view_document_selection` | View document-host select (experimental; **not wired on sites**) |
| `view_document_chrome` | PageUp/Home/End + doc key focus (pairs with document select) |

**Site canvas text selection is OFF (2026-11-12).** Document-host select
and per-Markdown gestures both left some cards live and neighbors dead
(feature grid / KOT). Keep site Markdown `selectable: false` until one
approach works everywhere — see [`DESIGN.md`](DESIGN.md) § Text selection
and [`sites/khaynjo/README.md`](../sites/khaynjo/README.md) § Remaining
(also: 14 world-shelf photos still “Photo coming”).

Two vendor behaviours the sites work around in their own DSL (vendor
untouched, apps unaffected):

- **Light text on dark grounds** shows faint glyph boxes: the SDF text
  shader adds `sdf_luma_bias` coverage to the whole glyph quad for
  light ink. Each site sets `sdf_luma_bias = 0.0` on `Label` / `Button`.
- **A `Fit`-height plain `View` does not paint `show_bg`.** Bars and
  cards that need a background are `RoundedView`s.

### Measured (2026-11-07, this machine)

- **kumkaj-site release** (`cargo makepad wasm --no-threads --strip
  --brotli build -p kumkaj-site --release`): wasm **6.1 MB → 1.33 MB
  brotli** — the same as the bare minimal-app baseline (6.3 MB → 1.32 MB),
  so seven chapters of widgets and shaders cost almost nothing. Debug
  builds are 9.5 MB per site.
- **Fonts are the real budget:** kumkaj's own faces are 657 KB brotli
  (Fraunces alone 199 KB; Noto Devanagari / Arabic / Gujarati / Bengali /
  Tamil 33–82 KB each); moklanjo and khaynjo carry 1.2 / 1.8 MB raw TTF.
  Nothing is subset yet (Phase 2: subset Latin display faces; load Indic
  faces only for the languages chapter).
- Verified natively at 390×844, 820×1180, 1024×768, 1280×720, 1280×800
  and 1920×1080 (every chapter of every site fits one screen); wasm
  served by each `run-web.sh` and rendered in headless Chromium at
  1280×800 and 390×844, including `#chapter` deep links; toys clicked
  natively (floor visit steps, weight steppers and divisor chips,
  cut-the-internet, chapter nav).
- **Known limit:** landscape phones (height < 480) clip the tallest
  chapters — a slide does not scroll. Portrait is the phone layout.

------------------------------------------------------------------------

## 3. Repo layout

```text
sites/kumkaj/      kumkaj-site    → kumkaj.com     (wasm :8010)
sites/moklanjo/    moklanjo-site  → moklanjo.com   (wasm :8011)
sites/khaynjo/     khaynjo-site   → khaynjo.com    (wasm :8012)

sites/<site>/
  src/ui.rs                 the whole page: tokens, type, components, chapters (script_mod!)
  src/main.rs               chrome, toys, links, responsive tiers, owner switches, tests
  src/weight.rs | floor.rs  toy rules copied from the app (copy, not import) + tests
  resources/fonts/          the faces this site draws, with their OFL licences
  scripts/run-web.sh        wasm build + local server
  scripts/run-desktop.sh    native window
  README.md
```

- Crates are workspace members; `default-members` stays the apps, so
  plain `cargo build` does not compile sites.
- A site crate depends on `vendor/makepad` **only** — never on an app
  crate (apps pull SQLite, sockets, printers). It copies colours, words
  and small rules from its product and pins them with tests.
- Served from 127.0.0.1 / localhost, each site links to the others on
  their local ports; anywhere else it links to the public domains.

------------------------------------------------------------------------

## 4. Look per site

| Site | Palette | Type | Chapter nav | Feel |
|---|---|---|---|---|
| kumkaj.com | Navy `#041534`, terracotta `#A23E18` (clay `#E8875C` for text on navy), ink `#F9F9FC` | Fraunces (display) · IBM Plex Sans | Dots + "n / 7" | Calm house for the products; navy ground with faint ledger rules and a warm glow |
| moklanjo.com | Desk light `#D6E0EC`, navy `#072058` / `#0A3A8C`, orange `#DF5E16` / `#C44E10`, yellow `#F7B043` | Anek Latin (display) · IBM Plex Sans · Liberation Mono (AWB, weights) | The parcel's tracking line: beads passed (yellow), here (orange), ahead (hollow) | Speed and precision; the icon's motion streaks |
| khaynjo.com | Cream `#F9F4F0`, deep red `#9A0E10`, maroon `#69090C`, gold `#F69D2C` | Baloo 2 (display) · IBM Plex Sans · Liberation Mono (KOT) · Noto scripts | Sticky red bar: section links + gold "Get it free"; visit-colour key in "How a service runs" | Warm lunch-rush energy; lamp-lit hero, cream / paper bands, deep-red offline + closing bands |

Contrast follows `DESIGN.md`: dark ink on light wells, white ink only on
deep chrome (navy / deep red), never grey-on-grey.

------------------------------------------------------------------------

## 5. How agents build the presentation

1. **Story in the app.** Chapters live in the site's `src/ui.rs`
   (`script_mod!`); one place for words, layout and motion. Claim ids
   sit in a comment above each chapter.
2. **Native first.** Edit → `cargo run -p <site>` → iterate; then
   `./sites/<site>/scripts/run-web.sh`. Note: DSL mistakes surface at
   **runtime** as `[E] … ui.rs:line:col` log lines, not in
   `cargo check` — read the log every run.
3. **See every tier.** `SITE_PREVIEW=390x844:4 cargo run -p <site>` opens
   a phone-sized window on chapter 4. On a headless box run it under
   Xvfb with `WAYLAND_DISPLAY` unset (else the window opens on the
   desktop) and grab the window by pid. Check 390×844, 820×1180,
   1280×720 and 1280×800: every chapter must fit one screen — a slide
   does not scroll, overflow is clipped.
4. **Responsive in Rust, not CSS.** `App::apply_responsive` picks a tier
   for type (480 / 720 / 1024 widths, `Wide` from 1600 × 900; short
   windows step down) and scales each label from its DSL desk size.
   Rows go side by side only when the **content width** fits them
   (`ROWS`: each row's minimum; `PROPS`: fixed-width cards that go full
   width when their row stacks) — width-based, because a 560 px card
   beside a headline needs ~1024 px on any device. `DETAIL_WHEN_STACKED`,
   `DETAIL_ON_PHONE` and `DETAIL_WHEN_SHORT` (under 860 px tall and 1200 px wide)
   are evaluated together on every resize.
5. **DSL overrides need direct children.** `Fact{head.text: …}` only
   reaches `head` if it is a direct child of the component; give every
   chapter and every looked-up widget a **unique** id (a rail line named
   `track` once shadowed the `track` chapter).
6. **Props in code first.** Cards, labels, tickets and floor tiles are
   widgets and small shaders; 3D props (CSG / glTF) come in Phase 2;
   device screens show real app captures, never mock UI.
7. **Selectable copy.** Goal remains browser-like selectable Markdown
   ([`DESIGN.md`](DESIGN.md) § Text selection & copy). **khaynjo.com
   currently ships `selectable: false`** after incomplete document-host
   / per-card approaches — open: README § Remaining. Set copy with
   `body:` (and `child.body:` on composites). Plain `Label` only inside
   tap targets (floor tiles, nav chips, choice faces, button faces).
8. **Claims check.** Before publishing, re-read each cited row; gated
   sentences stay behind the owner switch.

------------------------------------------------------------------------

## 6. Local preview

From the workspace root:

```bash
# Native windows (fastest; also the offline presenter)
./sites/kumkaj/scripts/run-desktop.sh
./sites/moklanjo/scripts/run-desktop.sh
./sites/khaynjo/scripts/run-desktop.sh

# Browser — Makepad wasm servers (each blocks; one terminal each)
./sites/kumkaj/scripts/run-web.sh      # http://127.0.0.1:8010
./sites/moklanjo/scripts/run-web.sh    # http://127.0.0.1:8011
./sites/khaynjo/scripts/run-web.sh     # http://127.0.0.1:8012

# Khaynjo: production-shaped release wasm (same tree deploy stages) before Firebase
./sites/khaynjo/scripts/preview-web.sh # http://127.0.0.1:8012
```

Scroll, swipe up, tap, or press ↓ / → / PgDn / Space for the next
chapter; ↑ / ← / PgUp go back; Home / End jump; the top-bar nav jumps
to a chapter. `#<chapter id>` or `#<n>` in the URL opens a chapter
(`http://127.0.0.1:8011/#weigh`). With all three servers up, kumkaj's
trade cards open the local product sites and the product footers link
back. Natively, `SITE_LINKS=local` does the same.

`run-web.sh` uses `cargo-makepad` from `vendor/makepad-full` so
packaging matches this workspace's Makepad. A newer global
`cargo-makepad` may require a font-assets custom section this tree does
not emit yet.

One-time toolchain (if missing):

```bash
rustup target add wasm32-unknown-unknown
rustup component add rust-src --toolchain nightly
```

------------------------------------------------------------------------

## 7. Roadmap

| Phase | Work | Status |
|---|---|---|
| **0 — Brand spike** | `sites/kumkaj` SlidesView brand story; workspace member; run scripts | **Done** 2026-11-07 |
| **1 — Product site crates** | Brand house deepened; `sites/moklanjo`, `sites/khaynjo` copied then diverged; trade palettes, type and nav; story chapters from the product docs; one toy per site; Download / WhatsApp CTAs behind owner switches | **Done as a local demo** 2026-11-07 (native + wasm) |
| **2 — 3D chapters + budgets** | PBR / glTF / 3D text chapters inside each site; real app captures on device screens; font subsetting; `--release --brotli --split --strip` size budgets | Next |
| **3 — Thin crawl shell + legal** | Static `/privacy` `/terms`, OG meta, `<noscript>` chapter text around the wasm; Play privacy URL; then retire the legacy static stubs | — |
| **4 — Presenter + Indic** | Offline presenter after one visit (service worker / packaged native); Hindi / Gujarati copy with native review | — |
| **5 — Growth** | Tools pages (volumetric weight, GST bill), case studies, Cloud pages when Cloud exists | — |

**Launch gates** (from `PRODUCTION_STATE_FOR_OFFLINE_USAGE.md`):
`/download` links go live only after clean-machine install passes;
LAN-pilot claims only after the real-hardware pilot. Both are code
switches today (§8).

**Catalogue on the product site** (shop templates as static files on the
**same** Firebase deploy as the Makepad wasm site):
`https://{product}/v1/catalogue/`. Khaynjo:
`sites/khaynjo/static/v1/catalogue/` +
`./sites/khaynjo/scripts/deploy-firebase.sh` (wasm + shell + catalogue).
Contract: [`apps/khaynjo/docs/CATALOGUE_URLS.md`](../apps/khaynjo/docs/CATALOGUE_URLS.md).

------------------------------------------------------------------------

## 8. Owner inputs (not agent work)

| # | Input | Where it goes |
|---|---|---|
| 1 | Kumkaj mark (only product icons exist; kumkaj.com uses a typographic wordmark) | `sites/kumkaj` top bar |
| 2 | Gotigin legal / contact / grievance details | Footers, Phase 3 legal pages |
| 3 | Domains and where the wasm builds are published | Hosting; `product_url` / `kumkaj_url` already point at the public domains |
| 4 | WhatsApp Business number per product | `WHATSAPP_NUMBER` in `sites/moklanjo/src/main.rs`, `sites/khaynjo/src/main.rs` (ship `None`, marked TODO) |
| 5 | Installers / store listing | Khaynjo: `DOWNLOADS_LIVE` + Play URL live (`com.khaynjo`). Moklanjo: still `false` until its listing/installers ship |
| 6 | Real-hardware LAN pilot passes | `LAN_PILOT_PASSED` in all three sites (ships `false`; shows the multi-device sentences) |
| 7 | Windows code-signing; Play developer account | Packaging |
| 8 | Native-speaker review for Hindi / Gujarati copy | Phase 4 |
| 9 | Real shop photos / testimonials with consent — or none | Never invented |
| 10 | Dish photos on the product site | Khaynjo: public shelf under `/v1/catalogue/public/items/` + curated chapter media (owner OK). Private client packs stay off the public index. |

------------------------------------------------------------------------

## 9. kumkaj.com home story (shipped chapters)

| # | Chapter | Says / does | Sources |
|---|---|---|---|
| 1 | The promise | "The internet went down. The counter didn't." + "this counter, right now" status card; *Choose your trade* / *Watch it keep working* | vision §6, §20 |
| 2 | Built for your trade | Moklanjo and Khaynjo choice cards — each with its spine and three shipped facts — linking to the product sites | MOKLANJO.md B rows; KHAYNJO.md §4b/§4d |
| 3 | Your books live in your shop | On your device · no account with us · backup copies you keep; the extra-counters line waits for `lan-pilot` | vision §6, §21 |
| 4 | Try it offline | Toy: cut the internet, the counter's work stays "Working" | vision §6, §20 |
| 5 | Free forever on your devices | ₹0 on your own devices; Kumkaj Cloud *planned*, usage-based | vision §21 |
| 6 | Speaks your staff's language | Catalogue languages (India + world), RTL; Khaynjo's real "Food ready" in Hindi, Gujarati, Tamil, Bengali, Urdu | `locale.rs` + `translations/*.tsv` |
| 7 | Made by Gotigin | Brand map Gotigin → Kumkaj → Moklanjo / Khaynjo; the choice again | — |

Code: `sites/kumkaj/src/ui.rs`, `sites/kumkaj/src/main.rs`.
