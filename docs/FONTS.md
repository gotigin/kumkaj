# Fonts — where each one comes from

**Rule:** a font enters this repo from its official upstream release — never as a copy from a
developer machine's font folders (`/usr/share/fonts`, a distro package). Record the source here
when you add or replace one. Scripts that need a font source download a pinned release and check
its SHA-256 (`scripts/site-cjk-font.sh`).

Use **static** instances for UI text. A variable font draws its *default* instance, which can be
Thin (NotoSansHebrew's is), and its glyphs keep overlapping contours, which Makepad's small-text
atlas (MSDF) can fill as boxes. `fc-query --format='%{variable}\n' f.ttf` or fontTools (`'fvar' in
TTFont(f)`) tells them apart.

## Sources (checked 2026-11-24)

| Files | Upstream | Notes |
|---|---|---|
| `NotoSans-Regular`, `NotoSans{Arabic,Armenian,Bengali,Ethiopic,Georgian,Gujarati,Gurmukhi,Hebrew,Kannada,Khmer,Lao,Malayalam,MeeteiMayek,Myanmar,OlChiki,Oriya,Sinhala,Tamil,Telugu,Thaana,Thai}-Regular.ttf` (apps + sites); khaynjo site also `NotoSerifTibetan-Regular.ttf`; apps + kumkaj site `NotoSansTibetan-Regular.ttf` (on-disk name — file is Noto **Serif** Tibetan static) | notofonts, static hinted TTF: `https://raw.githubusercontent.com/notofonts/notofonts.github.io/d14cc173a70dc57962ed2d4ad278fc9df83bf9f3/fonts/<Family>/hinted/ttf/<Family>-Regular.ttf` (`NotoSansTibetan-Regular.ttf` ← `NotoSerifTibetan`; khaynjo site uses `NotoSerifTibetan-Regular.ttf` for the same bytes) | Byte-identical to that release (SHA-256 compared). |
| `SiteCJK-{SC,TC,JP,KR}.otf` (sites), `CatalogueCJK.ttf` / `CatalogueHangul.ttf` (kumkaj), `NotoSansCJK-WorldShelf.otf` (khaynjo) | notofonts/noto-cjk release **Sans2.004**, `Sans/OTC/NotoSansCJK-Regular.ttc` (SHA-256 `b76b0433…690a`), cut with pyftsubset | `scripts/site-cjk-font.sh` downloads and checks it. The Catalogue faces were variable *Thin* subsets until 2026-11-24; now cut from this Regular. |
| `IBMPlexSans-Text.ttf`, `IBMPlexSans-SemiBold.ttf`, `fa-solid-900.ttf`, `LiberationMono-Regular.ttf`, `LXGWWenKai{Regular,Bold}.ttf`, `NotoColorEmoji.ttf` | Makepad's bundled resources (`vendor/makepad/widgets/resources`, upstream Makepad) | Byte-identical to Makepad's copies. |
| `Baloo2-Variable.ttf`, `AnekLatin-Variable.ttf`, `Fraunces-Variable.ttf` (site display faces), `NotoSans-Bold.ttf`, `NotoSansGujarati-Bold.ttf` (apps), `NotoSansDevanagari-Regular.ttf` (static, v2.006) | Google Fonts / notofonts, source not recorded | Display faces draw headlines (large sizes). Record the release when one is next touched. |
