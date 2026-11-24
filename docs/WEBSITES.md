# Kumkaj websites — Makepad 2 fullpage apps

> **Status:** Five Makepad 2 site crates — `sites/kumkaj`, `sites/moklanjo`,
> `sites/khaynjo`, `sites/kirano`, `sites/tabiyat` — each native + wasm.
> **Domains registered.** **Maturity (2026-11-18):** every site is a
> **scroll SPA** (Khaynjo/Wadhu pattern) with `selection_root` /
> `select_from_gaps`, dual-paint Markdown (`draw_selection` + `font_color`),
> `MIN_WIDTH` 320 + site-shell.css, in-app Privacy/Terms links, and
> preview/deploy scripts staging `/privacy` `/terms` + OG. Only
> **khaynjo-2b2a2** exists on Firebase today; the other four need a project
> id before first deploy (§8). WhatsApp numbers and some download switches
> remain owner steps. Next: Phase 2 3D chapters; finish Phase 3 (Firebase
> projects + custom domains + Analytics).
> **Scope:** Kumkaj-wide approach for all five domains. Product stories:
> [`MOKLANJO_WEBSITE.md`](../apps/moklanjo/docs/MOKLANJO_WEBSITE.md),
> [`KHAYNJO_WEBSITE.md`](../apps/khaynjo/docs/KHAYNJO_WEBSITE.md),
> [`KIRANO_WEBSITE.md`](../apps/kirano/docs/KIRANO_WEBSITE.md),
> [`TABIYAT_WEBSITE.md`](../apps/tabiyat/docs/TABIYAT_WEBSITE.md).
> **Architecture:** each site **is** a Makepad 2 app (wasm in the
> browser, the same crate native on desktop) — the whole page, not a
> canvas inside an HTML page.

------------------------------------------------------------------------

## Language catalogue (same as apps — hard rule)

Every Kumkaj **app** ships one shared 115-language catalogue
(`LANGUAGES` + translation tables). **No app may omit a language another
app has.** Websites must not invent a smaller set: when a site claims
languages or draws native names / samples, it uses that same catalogue and
the script faces those strings need (`WorldFont`). Gate:
`./scripts/check-language-parity.sh`.

**The sites speak it too (2026-11-23).** All five Makepad sites translate
their own copy: the top-bar chip (it shows the code in use — `EN`, `HI`,
`UR`) opens lang-nav.js's 115-language picker, which writes `#l=<code>`
(kept beside `#auth=` / sections) and localStorage; the wasm reads the hash
and repaints the page, the account and shop pages repaint through
`site-i18n.js`. Arabic-script and Hebrew languages right-align. No
preference-only picker: a language whose words are not in the site's packs
does not ship (site tests). Packs are fetched per language
(`/i18n/<code>.tsv`), not baked into the wasm. Since 2026-11-24 all 110
table languages are written for every site (aliases such as pt-BR / es-MX
fold to their parent). Since the same day lines wrap at Unicode line break
opportunities (Makepad patch `line_break_uax14`): Thai, Lao, Khmer and
Burmese break between words (their packs mark words with U+200B,
`scripts/mark-words.mjs`), no line starts with ، ۔ । or a dash, and RTL
paragraphs are laid out right to left (`rtl_paragraph_direction`), so Latin
names keep their place in an Urdu or Hebrew sentence. How it works and how
to add copy: [`DEV.md`](DEV.md) § Languages → Websites.

------------------------------------------------------------------------

## Firebase deploy policy (quota / cost — hard rule for **agents**)

**Owner** may deploy normally:

```bash
./sites/<site>/scripts/deploy-firebase.sh
```

No extra flag. Running the script yourself is approval.

**AI agents** (Cursor sets `CURSOR_AGENT=1`) must **not** upload Hosting on
every small change. They:

| Step | Required |
| --- | --- |
| 1 | Fix + test **locally** (`preview-web.sh` / `run-web.sh` / desktop) until the change is complete |
| 2 | Wait for **explicit owner approval** for that deploy |
| 3 | Only then deploy with an approval flag |

```bash
./sites/<site>/scripts/deploy-firebase.sh --i-approve-firebase-deploy
# or
GOTIGIN_APPROVE_FIREBASE_DEPLOY=1 ./sites/<site>/scripts/deploy-firebase.sh
```

`--dry-run` is always allowed (build/stage only; no upload). Agents must
default to local preview and **must not** invent approval.

Shared gate: [`scripts/require-firebase-deploy-approval.sh`](../scripts/require-firebase-deploy-approval.sh)
— blocks only when `CURSOR_AGENT=1` or `GOTIGIN_AI_AGENT=1`.

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
5. **Name order is always Trademark → Company → Brand → Product:**
   **Gotigin** (trademark) → **GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED**
   (company) → **Kumkaj** (brand) → product (Moklanjo / Khaynjo / …).
   Footer line example:
   "*Gotigin · GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED · Kumkaj · Khaynjo.*"
   No grievance address until the owner supplies one (§8).
6. **Name no competitor**, carrier or aggregator in copy or visuals;
   describe our own mechanics.
7. **Examples say so.** Illustrative props (rate cards, labels, tickets,
   bills, drawers) carry an *Example* tag; they are not app screens and
   carry no client names, menus, staff or real numbers. Real app
   captures replace them when device screens arrive (Phase 2).

Brand map: **Gotigin** (trademark) → **GOTIGIN SOFTWARE & HARDWARE PRIVATE
LIMITED** (company) → **Kumkaj** (BOS brand) → **Moklanjo** / **Khaynjo** /
**Kirano** / **Tabiyat** (products).

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

**Every site is a scrolling page (2026-11-18; khaynjo first on 2026-11-12):**
`ScrollYView` bands + sticky top bar, section deep links — not a
`SlidesView` deck (phones and short laptops no longer hide body copy).
Rows still switch side-by-side / stacked by content width (`ROWS`, `PROPS`).
`page` is the selection root (`select_from_gaps: true`). Sections are
measured from band rects when scrolling to them, so a deep link lands once
layout settles.

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

### Body type size (phone feel)

**Normal body** across Makepad product/brand sites targets **15.5px**
desk size (Khaynjo `Body` / Moklanjo `Body`), then phone `type_scale`
(~0.93–0.95). Lede/subhead stays larger (~18–19). Do not shrink display
headlines to “fix” body feel — change `Body` / `Lede` only. Documented
after Android Chrome phone pass 2026-11-22 (kumkaj was too big at Body 16
+ Lede 20).

### Accepted device limit — Khaynjo on old Mali (AGS2)

**Huawei AGS2-W09 / Mali-T830 / Chrome 101:** `khaynjo.com` can paint a
blank cream canvas after wasm load — WebGL `compile_fail.vertex` /
`Missing shader in FromWasmAllocVao` in CDP. Sibling Makepad sites
(kumkaj, moklanjo, kirano, tabiyat, wadhu, gotigin) paint on the same
tablet when opened one-tab-at-a-time. Cause is site-specific first-wave
WebGL pressure (long page + floor toy + world-shelf Noto faces + parade
textures), not the VV-resize freeze. Sprint 2026-11-22 tried: plain shelf
`Image` (no SDF), flat LampBand/Point/TicketRule (no unique `pixel`
shaders), and deferred reveal of floor/menus/shelf after hero warmup —
AGS2 stayed cream-blank (~20+ `compile_fail` in one-tab Chrome). A real
fix needs fewer simultaneous Makepad programs on that GPU (or a
`patches/makepad/` WebGL budget change); treat as an **accepted residual**
for that class of GPU. Prefer a current phone Chrome for Khaynjo QA.
Evidence: `Internal/sprint/shots/L4/mali/`.

### Threads

Build with **`--no-threads`** so ordinary hosting does not need
COOP/COEP / `SharedArrayBuffer`. Revisit only if a chapter truly needs
workers.

### Vendor patches the sites rely on

`vendor/makepad` is gitignored and rebuilt by `scripts/setup-makepad.sh`
from `patches/makepad/`. The sites need these patches (tracked since
2026-11-07; before that they lived only in the vendor tree and would
have vanished on the next setup):

| Patch | What |
|---|---|
| `slides_view_story_axis` | `SlidesView` `axis: Vertical`; tap / swipe / wheel flip one chapter (OS momentum ignored); presenter keys ↓ → PgDn Space, ↑ ← PgUp, Home / End; `goal_slide()` / `slide_count()` |
| `web_init_window_order` | wasm `ToWasmInit` runs `Event::Startup` (which creates window 0) **before** applying the window geometry — without it the canvas sticks on "Loading.." with an IdPool panic |
| `web_aux_buttons_stay` | mouse side buttons stay in-app; canvas context menu is Copy / Select all; PageUp–arrows/Space not stolen by the shell |
| `web_mobile_layout_width` | wasm: on touch, clamp canvas layout to `min(inner, outer, screen)` — Android Chrome **Desktop site** fakes `innerWidth` ~980 on a ~393 phone and made every Makepad site unusable (2026-11-21). Debounced `visualViewport` **resize** only (no VV **scroll**); skip no-op WebGL buffer realloc — URL-bar animation used to freeze scroll and smear text on Android Chrome / Firefox / Samsung Internet (2026-11-22). Clamp keys off `fullpage` **or** class `full_canvas` (sites use the class). Sets canvas + html/body **width/maxWidth** only — do **not** set body height (that raced `mobile-viewport-guard.js` and intermittently zoomed/cropped while scrolling, 2026-11-22). Keep in sync with `scripts/web/mobile_layout_size.js`. Gate: `./scripts/check-web-mobile-layout.sh` |
| `text_selection` | one selection model for apps and sites: drag selects across blocks like a browser page (from a margin too on sites, `select_from_gaps`; wheel and edge auto-scroll while dragging), double-click word, long-press / 0.5 s hold on touch; `View { selection_root: true }` (the site's `page`) answers Ctrl/Cmd+A and Copy of what is highlighted; on wasm the end of a touch selection opens the same Copy / Select all menu (`FromWasmShowClipboardActions`) |

**Site text selection is on (2026-11-13)** with that one model — see
[`DESIGN.md`](DESIGN.md) § Text selection & copy. The earlier
`widget_steals_document_selection` / `view_document_*` patches (a
cross-block drag host that left some cards dead) are gone. Still open:
14 world-shelf photos “Photo coming” ([`sites/khaynjo/README.md`](../sites/khaynjo/README.md) § Remaining).

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
  (Subset at staging since 2026-11-21 — see § The shell around the wasm.)
- Verified natively at 390×844, 820×1180, 1024×768, 1280×720, 1280×800
  and 1920×1080 (every chapter of every site fits one screen); wasm
  served by each `run-web.sh` and rendered in headless Chromium at
  1280×800 and 390×844, including `#chapter` deep links; toys clicked
  natively (floor visit steps, weight steppers and divisor chips,
  cut-the-internet, chapter nav).
- **Known limit:** landscape phones (height < 480) clip the tallest
  chapters — a slide does not scroll. Portrait is the phone layout.

### The shell around the wasm (2026-11-21, all five sites)

Nothing can run before the wasm has arrived, and link crawlers never run it
at all, so three things live in the HTML. Each site owns its copy under
`sites/<site>/shell/`.

- **Start-up screen** — `shell/boot.{css,html,js}` + `shell/logo.webp`,
  inlined into `index.html` by `scripts/patch-boot-screen.sh` (called by
  `patch-web-shell.sh`, so preview and deploy show the same thing). It
  replaces cargo-makepad's grey "Loading.." bar: the product's own tile in a
  ring that fills with the **real** wasm download (the page counts the bytes
  as they stream; the size is written in at build time), the name, a line
  that changes, then "Starting" until Makepad has painted, then a fade. It
  says so after 15 s on a slow line, and when the page cannot start it
  offers Reload and the plain pages. Colours are the site's own; change
  the lines in `boot.html`.
- **Link preview** — `static/media/link-preview.jpg`: a 1200×630 JPEG of the
  real first screen, named by the Open Graph and Twitter tags in
  `shell/head.html` (home) and in the privacy, terms and account pages. One
  set of tags serves WhatsApp, X, LinkedIn, Facebook, Telegram, Discord,
  Slack, iMessage, Instagram link stickers, and Google’s large-image preview
  (`robots` `max-image-preview:large` on the home shell). Required tags:
  `og:title` / `description` / `url` / `type` / `site_name` / `image` (+
  `type` `width` `height` `alt`), `twitter:card=summary_large_image` +
  title / description / image / `image:alt`, plus `canonical` and meta
  description — all absolute HTTPS, in the first HTML (no wasm). JPEG
  because some apps cannot read WebP; under 300 KB because WhatsApp drops
  bigger ones. After the first screen changes:
  `./sites/<site>/scripts/preview-web.sh` in one terminal,
  `./sites/<site>/scripts/make-link-preview.sh http://127.0.0.1:<port>/` in
  another, commit the picture, deploy. On the home page its address carries
  a hash of the file, because those apps keep a preview for days. Gate:
  `./scripts/check-link-previews.sh` (live) or `--local` against preview
  ports.
- **Head tags in one place** — `shell/head.html` (title, description,
  canonical, icons, preview tags, `WebSite` JSON-LD) is read by both
  `deploy-firebase.sh` and `preview-web.sh`; it used to be pasted in each.
  Every site now has `/favicon.png` and a touch icon (the browser's request
  for a missing `/favicon.ico` fetched a 21 KB error page on every visit).

**What a first visit downloads** (empty cache, 390×844, measured with
`Internal/tools/site-shots/netlog.mjs` — outside this repo — before this
change):

| Site | Requests | On the wire | of which wasm | of which fonts |
|---|---|---|---|---|
| kumkaj.com | 21 | 2.05 MB | 1.30 MB | 0.64 MB (8 files) |
| khaynjo.com | 64 | 3.58 MB | 1.32 MB | 1.39 MB (22 files) |
| moklanjo.com | 20 | 1.94 MB | 1.31 MB | 0.46 MB (5 files) |
| kirano.shop | 22 | 2.13 MB | 1.30 MB | 0.66 MB (8 files) |
| tabiyat.shop | 21 | 2.07 MB | 1.30 MB | 0.60 MB (7 files) |

- **The wasm is the Makepad runtime.** cargo-makepad already builds it with
  `opt-level=z`, one codegen unit, `build-std` with `panic_abort`, and strips
  the name section (9.7 MB → 6.4 MB → 1.3 MB brotli). A site's own code
  adds almost nothing, so the five are the same size.
- **`--split` does not help** (tried on gotigin.com, same Makepad): 5,130 of
  22,605 functions move to a second file, but "secondary remains on the
  startup path", so both halves load before the first frame and the total
  is 1.45 MB, not 1.33 MB.
- **`wasm-opt` does not help either** (Binaryen 128, `--all-features -Os`
  and `-Oz`, measured on the same wasm): the file is 10–15% smaller raw
  (6.40 → 5.76 / 5.46 MB) but 2–3% **larger** once brotli has had it (1.33
  → 1.36 / 1.37 MB), and time to first paint is the same (2.08 s both, four
  runs each). Leave the `--wasm-opt` flag off.
- **First-paint fonts are Latin (+ FA icons) only.** English page chrome
  uses `DisplayFont` / `StrongFont` / `TextFont` without Indic/CJK members
  (2026-11-23). Script samples must use a **separate** `WorldFont` (or
  equivalent) that is only referenced by those widgets — Khaynjo's deferred
  shelf, Kumkaj §06 `WordChip`. A `FontFamily` downloads every member on
  first use, so putting Noto in Text/Strong pulls those TTFs before the hero
  paints. **Regression rule:** never delete Noto faces from a site while that
  site still draws non-Latin copy; never point WordChips at Latin-only
  families. Doing either returns empty boxes (tofu).
- **Fonts are cut to the site's own characters when the tree is staged** —
  `sites/<site>/scripts/subset-fonts.sh`, called by `preview-web.sh` and
  `deploy-firebase.sh` (needs `pyftsubset` from fonttools; without it the
  fonts stay whole and the site works the same). It rewrites the staged
  copies only — `resources/fonts/` stays whole for the native build — to
  every character found in `src/*.rs` plus plain Latin, keeps the shaping
  tables for what stays, and replaces the font names with placeholders
  (IBM Plex and Liberation reserve their names for the unmodified font).
  The same script then **drops unused `makepad_widgets` faces** from staging
  (NewCMMath ~1.2 MB, NotoColorEmoji, JetBrains Mono, unused Plex italics,
  widget icon SVGs, duplicate `fa-solid` when the site ships its own) — those
  were copied by cargo-makepad but never declared by the sites.
  Variable display faces are pinned to the `ui.rs` weight before subset
  (kumkaj Fraunces `wght=620`, khaynjo Baloo 2 `wght=720`; moklanjo Anek).
- **Versioned and cached:** deploy gives the wasm and its loader
  `?v=<build id>`, and `firebase.json` now serves `*.wasm` `immutable`, so a
  returning visitor downloads it once per release.
- **Not shipped any more:** the `.br` twin cargo-makepad writes beside
  every file (Firebase compresses on its own and never serves them), and
  on khaynjo.com the old banner/screenshot stills (multi‑MB PNG/WebP) that
  sat in `resources/media/` and `static/media/` with nothing drawing them —
  originals stay in `apps/khaynjo/assets/` if needed. Cleared again
  2026-11-23 (`static/media/` keeps only link-preview + touch icon + small
  app-icon.webp). Indic faces stay on **khaynjo** `WorldFont` and **kumkaj**
  `WorldFont` (languages WordChips). Moklanjo / Kirano / Tabiyat chrome stays
  Latin-only unless they grow script samples.
- **Images are not inside the `.wasm`.** The release wasm is almost entirely
  the Makepad runtime (~6.2 MB raw → ~1.3 MB brotli). Fonts and icons are
  separate HTTP fetches; world-shelf photos already use a placeholder
  (`soon`) until their WebP arrives; feature-card icons load over HTTP after
  first-paint warmup (no `crate_resource` on the hot path). Prefer more
  separate downloads over growing the wasm — PNGs were never inlined.
- **Product tiles** are 192 px WebP with transparent corners, made from the
  masters in `apps/<app>/assets/` and `assets/images/`. The masters are a
  rounded tile on an opaque white or black square (Kirano's also has a
  black bar down its right edge), which showed as a box on every coloured
  header and card. Moklanjo's sites were still showing the pre-2026-11-05
  tile that reads "courier".

------------------------------------------------------------------------

## 3. Repo layout

```text
sites/kumkaj/      kumkaj-site    → kumkaj.com     (wasm :8010)
sites/moklanjo/    moklanjo-site  → moklanjo.com   (wasm :8011)
sites/khaynjo/     khaynjo-site   → khaynjo.com    (wasm :8012)
sites/kirano/      kirano-site    → kirano.shop    (wasm :8013)
sites/tabiyat/     tabiyat-site   → tabiyat.shop   (wasm :8014)

sites/<site>/
  src/ui.rs                 the whole page: tokens, type, components, chapters (script_mod!)
  src/main.rs               chrome, toys, links, responsive tiers, owner switches, tests
  src/weight.rs | floor.rs | counter.rs  toy rules copied from the app (copy, not import) + tests
  resources/fonts/          the faces this site draws, with their OFL licences
                            (include **fa-solid-900.ttf** — same WorldFonts `icons`
                            fallthrough as the apps; shelf play/pause etc. use FA
                            codepoints, never bare Unicode ⏸/▶ which paint tofu.
                            Populate with `./scripts/fetch-ui-fonts.sh sites/<product>`)
  shell/                    start-up screen + head tags inlined into index.html
  static/                   crawl shell overlays (terms, css; catalogue on khaynjo)
  scripts/run-web.sh        debug wasm + local server
  scripts/run-desktop.sh    native window
  scripts/preview-web.sh    release wasm + shell on the same port (pre-deploy)
  scripts/deploy-firebase.sh  stage public/ + firebase hosting (needs project id)
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
| kirano.shop | Cream `#FDF7EA`, orange `#CD380E` / `#F96303`, leaf `#1C460E` | Baloo 2 (display) · IBM Plex Sans · Liberation Mono | Top-bar tiles | Neighbourhood grocery counter; bill toy |
| tabiyat.shop | Cool white `#FBFCFC`, teal `#023637` / `#017B85`, lime `#37BE28` | Baloo 2 (display) · IBM Plex Sans · Liberation Mono | Top-bar tiles | Pharmacy counter; bill toy — never claims regulatory compliance |

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
   desktop) and grab the window by pid. Check **320, 360, 390, 600, 820,
   1280 and 1920** wide, scrolled top to bottom: nothing may touch or run
   off a side edge, and no word may break into letters. **Also on 1280 /
   1920:** Fit chip toolbars (play / speed / pager / − value +) must be
   **one row** when they fit — stacking that wastes horizontal space is
   not “responsive” (DESIGN § Always responsive — graceful growth). Do
   **not** blindly flatten intentional stacks (user-owned density and
   other mostly-obvious exceptions — DESIGN; if unsure leave alone).
   **Android Chrome:** also verify with **Desktop site** on and after a
   soft-keyboard open — layout width must stay ≈ `screen.width`, never the
   fake ~980 `innerWidth` (`web_mobile_layout_width` +
   `mobile-viewport-guard.js`; `./scripts/check-web-mobile-layout.sh`).
   360 is
   the common Android phone; the layout floor is 320 (`MIN_WIDTH` and
   `static/css/site-shell.css` — it was 375 until 2026-11-21, so every
   360 px phone scrolled sideways with the top bar cut off).
   Three faults that passed `cargo check` and tests and only a screenshot
   showed (2026-11-21):
   - **A `Markdown` ignores `visible`.** `x := Body{visible: false …}` logs
     "property visible not defined" and stays on screen. An owner switch or a
     width rule must hide a `View` around the text. kumkaj.com showed its
     LAN-pilot sentence this way while `LAN_PILOT_PASSED` was false.
   - **A `Markdown` with no `width` inside a `Fit` container draws nothing.**
     Give it `width: Fit`. The "Planned" and "Example" pills were empty.
   - **A fixed-width child beside a `Fill` text starves the text.** Two chips
     beside a title left the title 20 px and it broke one letter a line.
     Stack the row under a content width (`ROWS`), or narrow the fixed
     columns on phones.
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
7. **Selectable copy.** Browser-like selectable Markdown on every site
   ([`DESIGN.md`](DESIGN.md) § Text selection & copy): `page` is
   `selection_root` + `select_from_gaps`; Markdown uses `draw_selection`
   + `font_color`. Plain `Label` / `selectable: false` only for tap
   targets and chrome marks (top-bar brand, beads, floor tiles, buttons).
8. **Claims check.** Before publishing, re-read each cited row; gated
   sentences stay behind the owner switch.
9. **Wasm scroll / chrome — do not ghost text.** On Makepad wasm,
   rewriting labels or mutating `walk.width` every `NextFrame` / `Draw`
   while `scroll_pos` is moving leaves previous glyphs visible for a
   beat. Moklanjo
   ([`sites/moklanjo/README.md`](../sites/moklanjo/README.md) § Scroll)
   is the worked example for a continuous chapter rail:
   - Prefer **normal free-scroll** (do not claim the wheel into fixed
     page-steps — that locks the user to one speed).
   - Keep the **horizontal rail / beads in sync** with vertical scroll:
     `snap_rail_fill` when the chapter is unchanged; re-sync chrome on
     **NextFrame** after `Event::Scroll` / scroll-Y change on Draw
     (layout tops lag `set_scroll_pos`; never mutate rail width mid-Draw).
   - Bead / deep-link jumps may use a short timed ease; mid-anim:
     `set_scroll_pos` + rail snap only — **no** `set_text` on beads every
     tick. Full `paint_chrome` on settle or when the active section changes.
   - Short last sections need an `at_page_end` (or mid-viewport) promote
     so the last nav item lights when the footer is already on screen.
     Prefer **`scroll_pos ≥ max_scroll − ε`** over footer-rect flush (wasm
     often never reports flush).
   - After a wasm rebuild, hard-refresh; stale `?v=` tabs can keep an old
     binary (`RuntimeError: unreachable`).

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
./sites/kirano/scripts/run-web.sh      # http://127.0.0.1:8013
./sites/tabiyat/scripts/run-web.sh     # http://127.0.0.1:8014

# Production-shaped release wasm + /privacy /terms (same tree deploy stages)
./sites/kumkaj/scripts/preview-web.sh     # :8010
./sites/moklanjo/scripts/preview-web.sh   # :8011
./sites/khaynjo/scripts/preview-web.sh    # :8012
./sites/kirano/scripts/preview-web.sh     # :8013
./sites/tabiyat/scripts/preview-web.sh    # :8014
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
| **2 — 3D chapters + budgets** | PBR / glTF / 3D text chapters inside each site; real app captures on device screens; size budgets (font subsetting done 2026-11-21; `--split` and `wasm-opt` measured the same day: no gain, see § The shell around the wasm) | Next |
| **3 — Thin crawl shell + legal** | Static `/privacy` `/terms`, OG meta, `<noscript>` around the wasm; Play privacy URL; retire legacy zola stubs | **Done 2026-11-21** — all five hosted on their own domains; link-preview picture, Twitter tags, icons and a start-up screen in every shell |
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
| 3 | Firebase project ids (and Analytics web apps) for kumkaj / moklanjo / kirano / tabiyat; custom domains already registered | Hosting; `product_url` / `kumkaj_url` already point at the public domains. Shipped: `khaynjo-2b2a2`, `kumkaj-b32abc`, `moklanjo-com`. Attach **moklanjo.com** in Console → Hosting (prefer custom domain over `*.web.app`) |
| 4 | WhatsApp Business number per product | `WHATSAPP_NUMBER` in `sites/moklanjo/src/main.rs`, `sites/khaynjo/src/main.rs` (ship `None`, marked TODO) |
| 5 | Installers / store listing | Khaynjo: `DOWNLOADS_LIVE` + Play URL live (`com.khaynjo`). Moklanjo: still `false` until its listing/installers ship |
| 6 | Real-hardware LAN pilot passes | `LAN_PILOT_PASSED` in all three sites (ships `false`; shows the multi-device sentences) |
| 7 | Windows code-signing; Play developer account | Packaging |
| 8 | Native-speaker review for Hindi / Gujarati copy | Phase 4 |
| 9 | Real shop photos / testimonials with consent — or none | Never invented |
| 10 | Dish photos on the product site | Khaynjo: photo shelf under `/v1/catalogue/public/items/` + curated chapter media (owner OK). Shop templates, including Rajdhani, are rows on the same public `/v1/catalogue` index. |

------------------------------------------------------------------------

## 9. kumkaj.com home story (shipped chapters)

| # | Chapter | Says / does | Sources |
|---|---|---|---|
| 1 | The promise | "The internet went down. The counter didn't." + "this counter, right now" status card; *Choose your trade* / *Watch it keep working* | vision §6, §20 |
| 2 | Built for your trade | Moklanjo, Khaynjo, Kirano and Tabiyat choice cards — each with its spine and three shipped facts — linking to the product sites | MOKLANJO.md; KHAYNJO.md; KIRANO.md; TABIYAT.md |
| 3 | Your books live in your shop | On your device · no account with us · backup copies you keep; the extra-counters line waits for `lan-pilot` | vision §6, §21 |
| 4 | Try it offline | Toy: cut the internet, the counter's work stays "Working" | vision §6, §20 |
| 5 | Free forever on your devices | ₹0 on your own devices; Kumkaj Cloud *planned* as yearly Steady / Swift / Priority / Custom ([`CLOUD_PLANS.md`](CLOUD_PLANS.md)) | vision §21 |
| 6 | Speaks your staff's language | Catalogue languages (India + world), RTL; Khaynjo's real "Food ready" in Hindi, Gujarati, Tamil, Bengali, Urdu | `locale.rs` + `translations/*.tsv` |
| 7 | Who makes it | Trademark → Company → Brand → Product map; the choice again | — |

Code: `sites/kumkaj/src/ui.rs`, `sites/kumkaj/src/main.rs`.
