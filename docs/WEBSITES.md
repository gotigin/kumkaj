# Kumkaj websites — kumkaj.com, moklanjo.com, khaynjo.com

> **Status:** plan, 2026-11-05. Only today's one-page `web/index.html`
> exists; everything else below is to build.
> **Scope:** Kumkaj-wide (docs/README.md rule 1) — the approach all
> three sites share, plus kumkaj.com itself (§13). Each product site's
> own story, pages and 3D chapters live with its app:
> [`MOKLANJO_WEBSITE.md`](../apps/moklanjo/docs/MOKLANJO_WEBSITE.md),
> [`KHAYNJO_WEBSITE.md`](../apps/khaynjo/docs/KHAYNJO_WEBSITE.md).
> **Goal:** three responsive sites whose animated, 3D presentation is
> built with Makepad 2 and produced with AI agents — without giving up
> fast first paint, search, accessibility or honesty about what ships.

------------------------------------------------------------------------

## 1. Three sites, three jobs

| Site | Who arrives | Its job | Main action |
|---|---|---|---|
| **kumkaj.com** | Owners comparing systems, partners, press, future trades; people looking up Gotigin | Make the promise felt — *the counter keeps working when the internet stops; free forever on your own devices; built for your trade* — then send each visitor to their trade | **Choose your trade** → moklanjo.com / khaynjo.com |
| **moklanjo.com** | Courier / logistics agency owners, franchise counters, own-fleet operators, account managers selling it | Show a **courier desk**, not a till: book → label → hand over → track → settle | **Download free** · **WhatsApp for a demo** |
| **khaynjo.com** | Restaurant, café, QSR, cloud-kitchen, bakery owners and managers | Show a **service running** on the floor and in the kitchen, internet or not | **Download free** · **WhatsApp for a demo** |

Rules carried over from the vision — they bind the sites as much as the apps:

1. **Design per product** (vision §24). The three sites do not share a
   look, layout or motion language. Each takes its palette from its
   own app icon (§5). Similar or different is fine; the best page for
   *that* trade is the goal.
2. **No Kumkaj core** (vision §1). The sites share no code: templates,
   stage loader JS and the Makepad stage crate are copied per site and
   diverge. Only the workspace, vendored Makepad and root build scripts
   that name no site are shared.
3. **Claim only what ships** (vision §21, §31). Every feature sentence
   cites a **Ship / Have / Done** row in its app's doc (§6 step 5).
   Planned work (Kumkaj Cloud, SMS delivery, iOS/macOS) is labelled
   *planned* or left out.
4. **Free forever locally; cloud by usage, same rates for all**
   (vision §21) is the only pricing message.
5. **Gotigin** owns the brand. Footer on every site:
   "*Moklanjo / Khaynjo is a Kumkaj product by Gotigin.*" kumkaj.com
   carries the company page.

------------------------------------------------------------------------

## 2. What the current site gets wrong (fix in Phase 1)

Found in `web/index.html` and its packaging, 2026-11-05:

1. **Promises trades that do not ship** ("tea stall, clothes shop").
   Only Moklanjo and Khaynjo ship (vision §1, §31).
2. **Download section is developer commands**
   (`./apps/moklanjo/scripts/package-android.sh`), not downloads.
3. **Publishes the default Owner PIN `0000`** on a public page. Keep it
   in the install guide and first-run screen only.
4. **Moklanjo's site is two places at once:**
   `web/play-store-listing.txt` says *Website: moklanjo.com*, while
   `DEV.md` (platform table "Web" row, publish checklist step 6) and
   `apps/moklanjo/scripts/package-web.sh` publish Moklanjo at
   kumkaj.com. **Decision:** moklanjo.com is Moklanjo's site,
   kumkaj.com is the brand's. The listing moves to
   `apps/moklanjo/web/`; the brand site gets its own build (§4).
5. **No privacy-policy URL** exists for any product — Google Play
   requires one, so this blocks a Play listing. Ship `/privacy` on all
   three domains first.

------------------------------------------------------------------------

## 3. Architecture — HTML first, Makepad 3D stages on top

**Decision:** every page is static, semantic HTML that is complete
with no JavaScript and no WebGL. The animated 3D parts are **Makepad
stages**: one Makepad 2 wasm canvas per page, loaded *after* the page
is readable, driven by the page's own scrolling, and swapped for a
pre-rendered clip or still on devices that should not run it.

Why not build the whole site as one Makepad canvas:

- Google, WhatsApp/Facebook link previews and Play's website check
  read HTML, not canvas pixels.
- Screen readers, **browser translate** (many Hindi/Gujarati readers
  rely on it), find-in-page and text selection need real text.
- An owner on a mid-range Android over 4G must read the promise in
  under 2 s; a canvas-only page shows a loader until the wasm arrives.
- The stage still delivers the 3D presentation — it is just not the
  only path to the words.

```text
page.html (Zola)  — headings, copy, images, links, forms: works everywhere
  └─ stage.js (~5 KB, copied per site) decides once per visit:
       ├─ live   → stage.wasm (Makepad 2, WebGL2) in one <canvas>
       │           page → wasm: {chapter, progress 0..1, theme, motion}
       ├─ clip   → <video> per chapter, rendered from the same scene
       └─ still  → AVIF/WebP per chapter, rendered from the same scene
```

**One scene, every output.** The scene that runs live also renders,
offscreen, the fallback clips, the poster stills, each page's
OG/social image, the Play Store feature graphic and WhatsApp-status
clips for sales. Words and pictures come from one story file (§6), so
they cannot drift.

### 3.1 What Makepad 2 gives us (checked in the tree, 2026-11-05)

| Need | Makepad 2 has | Where |
|---|---|---|
| 3D scene, camera, depth | 3D scene scope, offscreen pass with colour + depth textures, orbit camera | `draw/src/scene_3d.rs`; camera/compositing via the `xr` crate in `examples/box3d`, `examples/cad` (makepad-full) |
| Lit materials | PBR shader: base colour, metal/roughness, normal, occlusion, emissive, environment maps | `draw/src/shader/draw_pbr.rs` |
| Models | glTF loader incl. animation | `libs/gltf` |
| Code-built props (agent-friendly) | CSG solids, geometry generators, cube | `libs/csg`, `draw/src/geometry`, `draw/src/shader/draw_cube.rs` |
| Text in 3D, any script | 3D text with the same shaping/bidi the apps use for Gujarati/Arabic tickets | `draw/src/shader/draw_text_3d.rs`, `libs/rustybuzz` |
| Labels pinned to 3D objects | world position → screen anchor per draw call | `SceneDrawCallAnchor` in `scene_3d.rs` |
| Motion | animator, frame tweens | `widgets/src/animator.rs`, `libs/frametween` |
| Physics (parcels dropping, piles) | box3d, rapier | `libs/box3d`, `libs/rapier` |
| Photoreal capture of a real counter | Gaussian splats | `libs/splat` |
| Presenter mode | `Slide`, `SlideChapter`, slides view | `widgets/src/slides_view.rs`, `examples/slides` |
| Web target | WebGL2 runtime; the canvas sizes to **its own element** unless marked `fullpage`, so it can sit inside a page section | `platform/src/os/web/web.js` (`update_window_info`) |
| Page ↔ wasm messages | JS message bridge | `web.js` `create_js_message_bridge`, `libs/wasm_bridge` |
| Download size | `cargo makepad wasm --brotli --split --strip --small-fonts --no-threads` | `tools/cargo_makepad/src/wasm/` |

Gaps found while checking:

- `vendor/makepad` (the tree the apps build against) has the 3D draw
  code but **not the `xr` crate** the 3D examples use for the orbit
  camera and for compositing the 3D pass into the UI. The spike
  (§11 Phase 0) either vendors `xr` at a matching revision or writes
  the stage's own small camera on the offscreen pass.
- The local `vendor/makepad-full` clone (915ce7c) does **not compile
  `makepad-widgets` for wasm32** (E0616, private
  `ScriptHeap.objects`), so it cannot be the stage's base as-is.
- `cargo makepad wasm` needs the nightly `rust-src` component
  (`rustup component add rust-src --toolchain nightly`) for its
  size-optimised std build — a one-time owner/dev-machine step.

**Measured baseline (2026-11-05, this machine):** a minimal Makepad 2
app (one `Window` + `Label`) built against `vendor/makepad` with
stable Rust, `wasm32-unknown-unknown`, `opt-level = "z"`, LTO, one
codegen unit, `panic = "abort"`, stripped: **6.3 MB wasm → 1.32 MB
brotli (q11)**. That is before cargo-makepad's own `--strip` /
`--split` passes and before any 3D scene code; Phase 0 measures the
real stage. **Fonts are separate and dominate if left alone:**
`vendor/makepad/widgets/resources` is **49 MB** (it carries large CJK
fonts). A stage loads only the fonts it draws — a Latin face plus
subset Devanagari/Gujarati for 3D text — via `--small-fonts` and its
own font list; never the default theme folder.

### 3.2 Threads and headers

cargo-makepad builds wasm **with threads by default**, which needs
`SharedArrayBuffer` and therefore cross-origin isolation (COOP
`same-origin` + COEP `require-corp`) on every page that runs a
stage. That breaks ordinary embeds (YouTube, maps) and is easy to
lose in a hosting change. **Build stages `--no-threads`.** Revisit
only if a scene truly needs worker threads (heavy physics), and then
set the headers on that path only.

### 3.3 When the stage runs live

`stage.js` picks **live** only when all hold, else **clip**, else
**still**:

- WebGL2 is available;
- `navigator.deviceMemory ≥ 4` and `hardwareConcurrency ≥ 4` (unknown
  on desktop Safari/Firefox counts as yes);
- no `Save-Data`, connection not `2g`/`slow-2g`;
- no `prefers-reduced-motion` (reduced motion gets **stills**, not clips).

It loads after `load` + idle **and** when the stage nears the
viewport — never before the largest text/image has painted. While
live it pauses when off-screen or the tab is hidden, caps the device
pixel ratio (2 on desktop, 1.5 on phones), and steps quality down
(shadows → MSAA → splat count) if frames exceed 20 ms for a second.
Clips are muted, `playsinline`, with a poster; on mobile data they
wait for a tap.

### 3.4 Scrolling and layout

- **Native scrolling always** — no scroll-jacking, no hijacked wheel
  or touch. The page measures each chapter's progress
  (IntersectionObserver + scroll) and posts it; the stage eases
  toward it.
- **Phones (< 720):** stage is a sticky band at the top (about 45 % of
  the viewport height); chapter text scrolls beneath it.
- **Tablet/desktop (≥ 720):** stage is a sticky column beside the
  chapter text, or full-bleed behind it with a scrim for contrast.
- Breakpoints follow the apps: **375** floor, **720** (the apps'
  rail/bottom-nav switch), **1024**, **1440**; type and spacing fluid
  with `clamp()`.
- **Toys** (Moklanjo's volumetric box, Khaynjo's table you can seat,
  kumkaj.com's "cut the internet" switch) always have an HTML form or
  button equivalent that works with no WebGL.
- Product → product navigation uses cross-document **View
  Transitions** within a site. Across domains (kumkaj.com → a product
  site) transitions cannot carry over, so the kumkaj.com camera dives
  into the storefront and the product site's hero opens on the same
  camera pose — continuity by design, not by API.

------------------------------------------------------------------------

## 4. Stack and repo layout

- **Static site generator: Zola** — one Rust binary, built-in
  multilingual sites, Sass, link checker, fast builds; no Node
  toolchain in the workspace. Plain CSS (custom properties,
  container queries, scroll-driven animations where supported) and a
  few KB of vanilla JS. No CSS framework, no tag managers.
- **2D motion:** CSS transitions, `animation-timeline: view()`, View
  Transitions, inline SVG. All honour `prefers-reduced-motion`.
- **3D:** one Makepad 2 stage crate per site.

Placement follows docs/README.md — brand at root, product with its app:

```text
web/kumkaj/                 Zola site → kumkaj.com (replaces web/index.html)
web/kumkaj/stage/           Makepad stage crate (kumkaj-web-stage)
apps/moklanjo/web/          Zola site → moklanjo.com (+ play-store-listing.txt)
apps/moklanjo/web/stage/    Makepad stage crate (moklanjo-web-stage)
apps/khaynjo/web/           Zola site → khaynjo.com
apps/khaynjo/web/stage/     Makepad stage crate (khaynjo-web-stage)
scripts/build-site.sh       names no site: KUMKAJ_SITE_DIR=<dir> → zola build,
                            stage wasm, offscreen renders → dist/web/<site>/
```

- A stage crate depends on `vendor/makepad` **only** — never on an app
  crate (apps pull SQLite, sockets and printers; a stage is a small
  pure scene) and never on another site's stage. It may *copy* an
  app's colours or shapes.
- Stage crates join the Cargo workspace; add `default-members` (the
  apps and tools) so a plain `cargo build` does not compile stages.
- `apps/moklanjo/scripts/package-web.sh` is replaced by
  `scripts/build-site.sh` with `KUMKAJ_SITE_DIR=apps/moklanjo/web`; the
  `DEV.md` platform table and publish checklist are updated in the
  same change.

------------------------------------------------------------------------

## 5. Look per site

| Site | Palette source | Feel | Type (self-hosted, subset WOFF2; Latin + Devanagari + Gujarati) |
|---|---|---|---|
| kumkaj.com | Today's page: navy `#041534`, terracotta `#A23E18`, surface `#F9F9FC` — until a Kumkaj mark exists (owner step) | Calm, trustworthy, "the house the products live in"; night scene in the hero | Fraunces (display) + Source Sans 3, Noto Sans Devanagari / Gujarati for Indic pages |
| moklanjo.com | Moklanjo tokens (`DESIGN.md` § Color tokens): navy `#072058` / `#0A3A8C`, orange `#DF5E16`, yellow `#F7B043`, surface `#D6E0EC` | Speed and precision — the icon's motion streaks become camera trails | Anek Latin / Anek Devanagari / Anek Gujarati (one variable family, width axis for "fast" headlines) |
| khaynjo.com | Khaynjo tokens (`DESIGN.md` § Color tokens — Khaynjo): deep red `#9A0E10`, maroon `#69090C`, red-orange `#D64D21`, gold `#F69D2C`, cream `#F9F4F0` | Warm, appetising, the energy of a lunch rush | Baloo 2 / Baloo Bhai 2 (Gujarati) — rounded like the logo type — with a plain sans for body |

Fonts are self-hosted (no Google Fonts call at runtime: privacy, and
one less origin for Indian mobile networks). Contrast rules from
`DESIGN.md` apply on the web too: dark ink on light wells, never
grey-on-grey, white text only on deep chrome.

------------------------------------------------------------------------

## 6. How AI agents build the animated presentation

The pipeline below is what lets Claude Code (or any coding agent in
this workspace) build, check and keep improving the 3D story without
a 3D artist on every change.

1. **Story file first.** Each site keeps `content/story.toml`: one entry
   per chapter with id, headline, body, claim ids, camera keyframes
   (position / target / fov at progress 0..1), objects on stage, beats
   (timed events), poster frame, alt text and speaker notes. Zola
   renders the HTML chapters from it; the stage embeds the same file
   for its keyframes. One file: text and motion cannot drift.
2. **Props in code first, models second.** Agents build the low-poly
   props from code (CSG + geometry generators): parcels, scale, label
   printer, carrier bag, van body, tables, chairs, plates, kitchen
   printer, tablets, paper tickets. Stylised and flat-to-soft-lit to
   match both app icons; tiny to download and easy to change in a
   diff. glTF only where code modelling is poor (people, vehicles):
   CC0 sources (Kenney, Quaternius, Poly Haven) or commissioned, with
   every licence listed in the site's `assets/LICENSES.md`. **Any
   device screen in a scene shows a real capture of the app** (from the
   app itself, e.g. `local/screencap`), mapped as a texture — never
   mock UI. The room is illustration; the screens are evidence.
3. **See-what-you-built loop.** Agent edits the scene → runs the stage
   **natively** (same code, fast compile) → renders frames offscreen at
   each chapter's key progress values to PNG → looks at them against
   the storyboard → iterates. Then builds wasm and screenshots the real
   page in headless Chromium at 375 / 720 / 1024 / 1440 and once under a
   throttled mid-range-Android profile.
4. **Budgets are tests** (§7). CI fails on: first-view weight, stage
   wasm size, Lighthouse LCP / CLS / INP, missing `<h1>` or alt text,
   broken links, a missing translation in a shipped language, or a
   claim without a source.
5. **Claims check.** `content/claims.toml` maps each claim id →
   sentence → source (doc path + row, e.g. `apps/moklanjo/docs/MOKLANJO.md`
   "Carrier AWB / barcode number pools — Ship"). On any content
   change the agent re-reads each source; a claim whose row is not
   **Ship / Have / Done** cannot publish. This is vision §21's
   "never advertised as implemented until evidence exists", enforced.
   A claim may also carry a `gate` (e.g. `lan-pilot`): it stays off the
   live site until the owner marks that gate passed — vision §31's
   "no hardware claims before they are tested".
6. **One scene, every render.** From the same scene: AVIF/WebP stills
   per chapter, AV1 + H.264 clips per chapter, a 1200×630 OG image per
   page (< 300 KB so WhatsApp shows it), the Play feature graphic
   (1024×500) and 1080×1920 clips for WhatsApp status / sales.
7. **Words and languages.** Agents draft English and translate into
   Hindi and Gujarati; nothing in another language publishes until a
   native speaker approves it. Agents never invent testimonials,
   numbers, client names or logos; a quote needs written consent.
8. **Presenter mode.** `/present` on each product site: the same stage
   full-screen, one chapter per arrow key or clicker press, speaker
   notes from `story.toml`. A service worker caches the stage and its
   assets, so an account manager can present inside a courier office
   or restaurant **with no internet** — the site proves the product's
   promise. Because a stage is an ordinary Makepad app, the same crate
   also builds as a native Windows / Linux / Android presenter for
   sales tablets.

------------------------------------------------------------------------

## 7. Budgets (performance and accessibility)

| Measure | Budget | Measured on |
|---|---|---|
| LCP | ≤ 2.0 s | Lighthouse mobile (mid-range Android, 4G) |
| CLS | ≤ 0.05 | same |
| INP | ≤ 200 ms | same |
| First view (HTML + CSS + fonts + hero still) | ≤ 200 KB transferred | same |
| JS before the stage | ≤ 10 KB | — |
| Stage wasm + assets per site | set after the Phase 0 spike; working target ≤ 3 MB brotli in total, fetched only for live stages | — |
| Clip per chapter | ≤ 600 KB | — |
| Live frame rate | 60 fps desktop, ≥ 30 fps phone — else the device gets clips | real devices |

Accessibility: **WCAG 2.2 AA**. The canvas is decorative
(`aria-hidden`); every chapter says in text what the scene shows. All
toys have HTML equivalents and keyboard control. Visible focus,
reduced motion honoured, `lang` set per page, `dir="rtl"` ready for
Urdu/Arabic pages later.

------------------------------------------------------------------------

## 8. Languages

- **Wave 1:** English, Hindi, Gujarati (first clients work in Gujarati
  and Hindi; the brand names are Memoni).
- **Wave 2 by demand:** Marathi, Tamil, Telugu, Bengali, Urdu (RTL), …
- The **apps** ship 29 languages (vision §23) — the sites may say so
  (it ships); the sites' own copy grows wave by wave.
- URLs `/hi/…`, `/gu/…` with `hreflang`; the language switch keeps
  the visitor on the same page.

------------------------------------------------------------------------

## 9. Search and sharing

- Each product site targets its trade's searches; **tool pages**
  (Moklanjo's volumetric-weight calculator, Khaynjo's tax-inside /
  tax-added bill calculator) are HTML forms first, so they rank and
  work everywhere.
- Structured data: `SoftwareApplication` (operating systems Android /
  Windows / Linux, offer price 0) on product sites, `Organization`
  (Gotigin) on kumkaj.com, `BreadcrumbList`, `FAQPage` only for real
  FAQs.
- Per-page OG images rendered by the stage (§6 step 6).
- `sitemap.xml`, `robots.txt`, canonical URLs across languages.
- Name no competitor in copy or visuals (§12 item 6); describe our
  own mechanics.

------------------------------------------------------------------------

## 10. Hosting, downloads, analytics, legal

- **Hosting:** Cloudflare Pages, one project per domain — free tier,
  Indian edge locations, brotli, HTTP/3. `_headers`: hashed assets
  immutable for a year, HTML no-cache, `application/wasm`, HSTS, a
  strict CSP. (The build output is plain files; Netlify or GitHub
  Pages work the same.)
- **Downloads:** Play Store when listed; direct APK, Windows MSI and
  Linux tarball from GitHub Releases or Cloudflare R2, each with its
  SHA-256 and release notes. `/download` reads a small `releases.toml`
  written by each app's packaging scripts. Unsigned MSIs trigger
  SmartScreen warnings that cost downloads — a code-signing
  certificate is an owner step.
- **Analytics:** cookieless (Cloudflare Web Analytics). No third-party
  trackers, so no cookie banner — and it matches "your data stays
  yours".
- **Contact:** static hosting cannot receive forms. Use WhatsApp
  click-to-chat with a prefilled message (product, trade, city) and an
  email address shown as text. A lead form waits for a backend
  (Kumkaj Cloud, later).
- **Legal, per site:** privacy policy (the apps keep data on the
  device; what the website itself collects), terms, licences. India's
  DPDP Act 2023 needs a grievance contact — owner supplies Gotigin's
  details.

------------------------------------------------------------------------

## 11. Roadmap

Each phase ends with something live; each site can ship on its own.

| Phase | Work | Exit check |
|---|---|---|
| **0 — Stage spike** (~1 week) | In a Zola page: a non-fullpage Makepad canvas; scroll progress posted from JS moves the camera; a PBR prop + one glTF + 3D Gujarati text; `--no-threads --brotli --split` build; offscreen frames → PNG → AV1/H.264 clip; decide `xr` crate vs own camera | Sizes and fps on a mid-range Android phone (Chrome) and an old Intel-graphics Windows laptop written into §3.1 / §7. If phones cannot hold 30 fps, phones get clips (still Makepad-rendered) and desktops stay live |
| **1 — Foundations + legal + kumkaj.com v1** (~1–2 weeks) | Three Zola skeletons, tokens, fonts, `story.toml` / `claims.toml`, CI budgets, hosting + DNS + headers; `/privacy` + `/terms` on all three domains (unblocks Play); kumkaj.com v1 with stills; §2 fixes; `DEV.md` + packaging updated | Lighthouse budgets green; Play listing can point to moklanjo.com/privacy |
| **2 — moklanjo.com v1** (~2 weeks) | All pages HTML-first, volumetric calculator, downloads, guide; then the live parcel-journey stage + clips | Claims check green; stage budget met |
| **3 — khaynjo.com v1** (~2 weeks) | All pages HTML-first, bill calculator, downloads, guide; then the live service stage + clips | same |
| **4 — kumkaj.com stage + presenter + Indic** | Lights-out hero, storefront turntable, 29-language ring; `/present` on both product sites (offline); Hindi + Gujarati with native review | Presenter works in airplane mode after one visit |
| **5 — Growth** | Glossary and how-to pages, case studies with consent, wave-2 languages, Kumkaj Cloud pages once Cloud exists | — |

**Launch gates (from `PRODUCTION_STATE_FOR_OFFLINE_USAGE.md` §3, both
apps, as of 2026-10-31):** neither app has been installed from its
built package on a clean machine, nor run on real shop hardware.
So a product site may go public with its story, features and tools,
but **`/download` links go live only after the clean-machine install
passes**, and claims gated `lan-pilot` (printing in the kitchen,
tablets and extra counters on the shop Wi-Fi) appear only after the
real-hardware pilot. Building the sites does not wait for either.

Moklanjo goes before Khaynjo because it is the current ship focus
(vision §33) and its Play listing already names moklanjo.com. Swap 2
and 3 if a Khaynjo rollout needs its site first — the sites are
independent.

------------------------------------------------------------------------

## 12. Owner decisions and inputs (not agent work)

1. **Kumkaj mark** — only the product icons exist.
2. **Gotigin details** — registered name and address, contact email,
   grievance officer (privacy policy, footer, `Organization` data).
3. **DNS** for the three domains and a hosting account.
4. **WhatsApp Business number(s)** for click-to-chat, per product.
5. **Windows code-signing certificate**; Google Play developer account.
6. **Competitor naming** — recommended: never name competitors or
   carriers; describe our mechanics only.
7. **Open-source stance** — the app crates declare
   `MIT OR Apache-2.0`. If the source is public, kumkaj.com gets an
   open-source page; if not, correct the licence field.
8. **Native-speaker reviewers** for Hindi and Gujarati.
9. **Real photos / video and testimonials** from client shops (with
   written consent) — or none.
10. **Dev machine:** `rustup component add rust-src --toolchain nightly`
    for `cargo makepad wasm`.
11. **Approve the budgets** after the Phase 0 spike.
12. **Launch gates** (§11): clean-machine install of each package, and
    the real-hardware LAN pilot — both already on the owner's list in
    `PRODUCTION_STATE_FOR_OFFLINE_USAGE.md` §3.
13. **Khaynjo dish-photo shelf** (`apps/khaynjo/assets/items/`) —
    confirm the licence allows web/marketing use, or keep it off the
    sites.
14. **Menu set-up offer** — whether Gotigin prepares a restaurant's
    menu template for it (khaynjo.com `/switch` says how a menu comes
    over; it must not promise a service nobody has agreed to run).

------------------------------------------------------------------------

## 13. kumkaj.com

### 13.1 Pages

| Path | Content |
|---|---|
| `/` | The story below |
| `/principles` | Software adapts to users (vision §2); local-first (§6); your data, your device; free forever locally (§21); no price per seat, item, table or local device; cloud never holds data hostage (§22) |
| `/languages` | The 29 app languages, RTL for Arabic/Urdu, tickets printed in the shop's own script |
| `/cloud` | Kumkaj Cloud — **planned**: what it will meter (storage, transfer, requests), what is never charged; no sign-up until it exists |
| `/security` | Data stays on the shop's devices; extra tills join over the shop's own Wi-Fi with a pairing code (no TLS on the LAN today — said plainly); backups and restore |
| `/about` | Gotigin; why concrete products, not one generic app; the names — in Memoni, *Moklanjo* is "to send" and *Khaynjo* is "food" |
| `/press` | Marks, app screenshots, stage renders, fact sheet |
| `/privacy`, `/terms`, `/contact` | Legal and contact (§10) |

### 13.2 Home story — "The internet went down. The counter didn't."

Stage: a small street at dusk, two storefronts side by side — a courier
counter and a restaurant — each with its Kumkaj device glowing.

| # | Chapter | On stage | Says |
|---|---|---|---|
| 1 | **Hero** | A shimmering signal cloud hangs over the street. Scroll: the cloud breaks up and drifts off; the street's lights flicker — but in both storefronts a label still prints and a kitchen ticket still prints. Toy: **Cut the internet** switch (HTML fallback: a two-state image) | "The internet went down. The counter didn't." |
| 2 | **Built for your trade** | Camera rises; the storefronts become dioramas on plinths: the courier desk (scale, parcels, label printer) and the restaurant (tables, kitchen pass). Tap one → the camera dives in → navigation to that product site, whose hero opens on the same camera pose | "A courier desk for couriers. A restaurant system for restaurants. Not one app that makes you translate." |
| 3 | **Your books live in your shop** | A glass box around the shop's devices; sales, parties and the day book stack up inside it. Local Wi-Fi lines join a second till and a waiter's tablet. A dotted line to a cloud appears only if the visitor switches it on — labelled *Kumkaj Cloud, planned, pay only for what you use* | "Your data stays on your devices. Extra tills join over your own Wi-Fi." |
| 4 | **Free forever on your devices** | A price tag turns to ₹0; things we never charge for fall into place around it — staff, menu items, tables, devices on your Wi-Fi, local reports | "Run it free, forever. If you choose our cloud later, pay only for what you use — same rates for everyone." |
| 5 | **Speaks your language** | A slowly turning ring of a welcome word in 29 languages, drawn as 3D text by the same shaping engine the apps use for receipts; Arabic and Urdu read right-to-left | "29 languages, on every screen and ticket." |
| 6 | **Made by Gotigin** | The two storefronts again, lights on | Choose your trade → Moklanjo / Khaynjo |
