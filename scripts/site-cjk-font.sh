#!/usr/bin/env bash
# Cut the CJK faces a Kumkaj site needs for its own language packs.
#
#   ./scripts/site-cjk-font.sh <site>      # khaynjo | moklanjo | kirano | tabiyat | kumkaj
#
# Writes sites/<site>/resources/fonts/SiteCJK-{SC,TC,JP,KR}.otf: Noto Sans CJK
# (system /usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc, OFL) cut to the
# characters of translations/zh.tsv, zh-Hant.tsv, ja.tsv and ko.tsv — one face
# per language so Han characters take their own regional shapes. Each is a few
# hundred KB and loads only when a visitor picks that language (font_cjk_* in
# ui.rs). `every_translation_draws_with_the_site_fonts` fails when a pack gains
# a character its face lacks — rerun this script then. Needs pyftsubset.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITE="${1:?usage: site-cjk-font.sh <site>}"
DIR="$ROOT/sites/$SITE"
TTC="${NOTO_CJK_TTC:-/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc}"
[[ -d "$DIR/translations" ]] || { echo "error: no $DIR/translations" >&2; exit 1; }
[[ -f "$TTC" ]] || { echo "error: missing $TTC (install noto-fonts-cjk or set NOTO_CJK_TTC)" >&2; exit 1; }
command -v pyftsubset >/dev/null || { echo "error: pyftsubset (fonttools) not installed" >&2; exit 1; }
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
