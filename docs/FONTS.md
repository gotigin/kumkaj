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
| `Baloo2-Variable.ttf` (khaynjo, kirano, tabiyat sites) | Google Fonts `ofl/baloo2/Baloo2[wght].ttf`, commit `8e0cd05881f3ae5e933ba4161103005a3e841ce0`: `https://raw.githubusercontent.com/google/fonts/8e0cd05881f3ae5e933ba4161103005a3e841ce0/ofl/baloo2/Baloo2%5Bwght%5D.ttf` | Byte-identical (SHA-256 `d47a6852548059b1db49a1319d06d499d546c3fa2237cf9eee9c43c8abb025c2`). Site display face. |
| `AnekLatin-Variable.ttf` (moklanjo site) | Google Fonts `ofl/aneklatin/AnekLatin[wdth,wght].ttf`, commit `ba4334505244fc033e8e59342b4682ebf91c6a21`: `https://raw.githubusercontent.com/google/fonts/ba4334505244fc033e8e59342b4682ebf91c6a21/ofl/aneklatin/AnekLatin%5Bwdth%2Cwght%5D.ttf` | Byte-identical (SHA-256 `ef7077abf2166add6ab6a64b4a4a4407859bf4f9e5b9058b51ac01ada136b295`). Site display face. |
| `Fraunces-Variable.ttf` (kumkaj site) | Google Fonts `ofl/fraunces/Fraunces[SOFT,WONK,opsz,wght].ttf`, commit `ac502d8eff76ef4d9477cdcc8ef7d0c84fde5372`: `https://raw.githubusercontent.com/google/fonts/ac502d8eff76ef4d9477cdcc8ef7d0c84fde5372/ofl/fraunces/Fraunces%5BSOFT%2CWONK%2Copsz%2Cwght%5D.ttf` | Byte-identical (SHA-256 `177ff6c0f14e5550a3c624247cd1189611d4eb65d000b14944c63d967958abbb`). Site display face. |
| `NotoSans-Bold.ttf` (khaynjo, kirano, tabiyat apps) | `googlefonts/noto-fonts` static hinted TTF v2.008, commit `c971829a87e7920f960e7277c3dafd9bedd3c601`: `https://raw.githubusercontent.com/googlefonts/noto-fonts/c971829a87e7920f960e7277c3dafd9bedd3c601/hinted/ttf/NotoSans/NotoSans-Bold.ttf` | Byte-identical (SHA-256 `c976e4b1b99edc88775377fcc21692ca4bfa46b6d6ca6522bfda505b28ff9d6a`). Not on current `google/fonts` `main` (variable only); legacy notofonts monorepo. |
| `NotoSansGujarati-Bold.ttf` (khaynjo, kirano, tabiyat apps) | `googlefonts/noto-fonts` static hinted TTF v2.101, commit `8b09983625b2b5960fb1ce19e9b3cac293d782fe`: `https://raw.githubusercontent.com/googlefonts/noto-fonts/8b09983625b2b5960fb1ce19e9b3cac293d782fe/hinted/ttf/NotoSansGujarati/NotoSansGujarati-Bold.ttf` | Byte-identical (SHA-256 `dfa02697acd99735fab0082a0387ff7c2c9288227e80c0d277479cf29dd4cb4e`). Same legacy monorepo path as Latin Bold. |
| `NotoSansDevanagari-Regular.ttf` (apps + sites, static v2.006) | notofonts.github.io commit `4107892fb07b01fbccc96e43638028f6bf127a4a`: `https://raw.githubusercontent.com/notofonts/notofonts.github.io/4107892fb07b01fbccc96e43638028f6bf127a4a/fonts/NotoSansDevanagari/hinted/ttf/NotoSansDevanagari-Regular.ttf` (same bytes in release **NotoSansDevanagari-v2.006**, `notofonts/devanagari`, zip `NotoSansDevanagari/hinted/ttf/…`) | Byte-identical (SHA-256 `306b53ecfb182a504dd8a7446093c316387d2fd8dc350d0792ed1753fe0996cd`). |
