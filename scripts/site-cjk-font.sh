#!/usr/bin/env bash
# Cut the CJK faces a Kumkaj site needs for its own language packs.
#
#   ./scripts/site-cjk-font.sh <site>      # khaynjo | moklanjo | kirano | tabiyat | kumkaj
#
# Writes sites/<site>/resources/fonts/SiteCJK-{SC,TC,JP,KR}.otf: Noto Sans CJK
# Regular (OFL) cut to the characters of translations/zh.tsv, zh-Hant.tsv,
# ja.tsv and ko.tsv — one face per language so Han characters take their own
# regional shapes. Each is a few hundred KB and loads only when a visitor picks
# that language (font_cjk_* in ui.rs). `every_translation_draws_with_the_site_fonts`
# fails when a pack gains a character its face lacks — rerun this script then.
# Needs pyftsubset and curl.
#
# Source: the official notofonts/noto-cjk release Sans2.004 (pinned URL, checked
# against its SHA-256), downloaded once into ~/.cache/kumkaj-fonts. Never a copy
# from the machine's own font folders (docs/FONTS.md). NOTO_CJK_TTC=<file> uses a
# local copy of that same release instead.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITE="${1:?usage: site-cjk-font.sh <site>}"
DIR="$ROOT/sites/$SITE"
CJK_URL="https://raw.githubusercontent.com/notofonts/noto-cjk/Sans2.004/Sans/OTC/NotoSansCJK-Regular.ttc"
CJK_SHA256="b76b0433203017ca80401b2ee0dd69350349871c4b19d504c34dbdd80541690a"
[[ -d "$DIR/translations" ]] || { echo "error: no $DIR/translations" >&2; exit 1; }
command -v pyftsubset >/dev/null || { echo "error: pyftsubset (fonttools) not installed" >&2; exit 1; }
if [[ -n "${NOTO_CJK_TTC:-}" ]]; then
  TTC="$NOTO_CJK_TTC"
else
  CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/kumkaj-fonts"
  TTC="$CACHE/NotoSansCJK-Regular-Sans2.004.ttc"
  if [[ ! -f "$TTC" ]]; then
    mkdir -p "$CACHE"
    echo "Downloading Noto Sans CJK (Sans2.004, ~19 MB) from notofonts/noto-cjk…"
    curl -fL --retry 2 -o "$TTC.part" "$CJK_URL"
    mv "$TTC.part" "$TTC"
  fi
fi
[[ -f "$TTC" ]] || { echo "error: missing $TTC" >&2; exit 1; }
echo "$CJK_SHA256  $TTC" | sha256sum -c --quiet - \
  || { echo "error: $TTC is not the pinned Noto Sans CJK Sans2.004 release" >&2; exit 1; }
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
# face index in the collection: 0 JP, 1 KR, 2 SC, 3 TC
for spec in "zh:SC:2" "zh-Hant:TC:3" "ja:JP:0" "ko:KR:1"; do
  IFS=: read -r code tag index <<<"$spec"
  pack="$DIR/translations/$code.tsv"
  {
    [[ -f "$pack" ]] && cut -f2- "$pack"
    # Punctuation and digits the copy may use in full width.
    printf '，。、；：？！「」『』（）《》〈〉【】・…—～％０１２３４５６７８９'
  } | LC_ALL=C.UTF-8 grep -o . | LC_ALL=C.UTF-8 sort -u | tr -d '\n' > "$WORK/$tag.txt"
  out="$DIR/resources/fonts/SiteCJK-$tag.otf"
  pyftsubset "$TTC" --font-number="$index" --text-file="$WORK/$tag.txt" \
    --layout-features='*' --notdef-outline --output-file="$out"
  echo "SiteCJK-$tag.otf ($code): $(LC_ALL=C.UTF-8 wc -m < "$WORK/$tag.txt") characters, $(( $(stat -c %s "$out") / 1024 )) KB"
done
