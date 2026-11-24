#!/usr/bin/env bash
# Shrink the fonts in a staged moklanjo.com tree to the characters the site draws,
# and pin the variable display face to the weights actually drawn (src/ui.rs:
# AnekLatin weight 640 + 760; width 100). Character subset alone left AnekLatin
# ~250 KB; pin + subset lands near ~90 KB without changing those drawn weights.
#
# Every font a Makepad site declares is fetched before its first frame, and
# the files are whole typefaces: 5 files, 1.1 MB. This rewrites the
# staged copies — never the ones in the repo — so each keeps only
#   - every character that appears in sites/moklanjo/src/*.rs (all the site's copy,
#     tables and symbols live there), and
#   - plain Latin for anything typed or computed: Basic Latin everywhere, and
#     in the Latin faces also Latin-1, Latin Extended-A, general punctuation,
#     currency signs and the four arrows.
# Shaping tables are kept for the glyphs that stay, so conjuncts and joined
# forms still work. Names are replaced with placeholders (some of these
# faces reserve their name for the unmodified font).
#
#   ./sites/moklanjo/scripts/subset-fonts.sh <staged public dir>
#
# Needs pyftsubset (fonttools). Without it the fonts stay whole: the site
# works the same and a first visit is heavier. Text a visitor types in a
# script the site's own copy does not use shows empty boxes — add a sample
# of it to the sources if that ever matters.
# Used by preview-web.sh and deploy-firebase.sh.
set -euo pipefail

OUT="${1:?usage: subset-fonts.sh <staged public dir>}"
SITE="$(cd "$(dirname "$0")/.." && pwd)"
FONT_DIR="$OUT/moklanjo_site/resources/fonts"
LATIN_MORE='U+00A0-00FF,U+0100-017F,U+2010-2027,U+2030-203A,U+20A0-20BF,U+2190-2193'
# Glyphs the shaper splits a character into before it looks the glyphs up:
# Thai / Lao SARA AM (ำ → ํ + า, ຳ → ໍ + າ), the two-part vowels of Bengali,
# Oriya, Tamil, Telugu, Kannada, Malayalam, Sinhala, Khmer and Tibetan, and
# the nuktas of decomposed letters. The packs rarely use these on their own,
# so a subset cut to the packs' characters drops them and draws □ in their
# place (seen 2026-11-24: Thai ทำ, คำ). Kept in every face; a face without
# the script ignores them.
SHAPER_PARTS='U+0E32,U+0E4D,U+0EB2,U+0ECD,U+093C,U+09BC,U+09BE,U+09C7,U+09D7,U+0A3C,U+0B3C,U+0B3E,U+0B47,U+0B56,U+0B57,U+0BBE,U+0BC6,U+0BC7,U+0BD7,U+0C46,U+0C56,U+0CBF,U+0CC2,U+0CC6,U+0CCA,U+0CD5,U+0CD6,U+0D3E,U+0D46,U+0D47,U+0D57,U+0DCA,U+0DCF,U+0DD9,U+0DDC,U+0DDF,U+0F71,U+0F72,U+0F74,U+0F80,U+1025,U+102E,U+1031,U+17C1'
# Weights drawn in sites/moklanjo/src/ui.rs (Display / DisplaySoft).
ANEK_WGHT_MIN=640
ANEK_WGHT_MAX=760

if [[ ! -d "$FONT_DIR" ]]; then
  echo "error: no fonts at $FONT_DIR" >&2
  exit 1
fi
if ! command -v pyftsubset >/dev/null 2>&1; then
  echo "note: pyftsubset (fonttools) is not installed — fonts stay whole"
  exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
# One line holding every distinct character of the sources.
# Site copy (src/*.rs) and every language pack (translations/*.tsv): the script
# faces keep exactly the characters the translated page can draw.
cat "$SITE"/src/*.rs "$SITE"/translations/*.tsv | LC_ALL=C.UTF-8 grep -o . | LC_ALL=C.UTF-8 sort -u | tr -d '\n' > "$WORK/chars.txt"
if [[ "$(LC_ALL=C.UTF-8 wc -m < "$WORK/chars.txt")" -lt 60 ]]; then
  echo "error: could not read the site's characters from $SITE/src" >&2
  exit 1
fi

# Pin AnekLatin's variable axes to the drawn weight range and default width
# before the glyph subset (fontTools.varLib.instancer).
pin_anek_latin() {
  local src="$1" dest="$2"
  python3 - "$src" "$dest" "$ANEK_WGHT_MIN" "$ANEK_WGHT_MAX" <<'PY'
import sys
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

src, dest, wmin, wmax = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4])
font = TTFont(src)
if "fvar" not in font:
    font.save(dest)
    raise SystemExit(0)
# Return value is the pinned face (do not assume in-place mutation).
pinned = instantiateVariableFont(
    font, {"wght": (wmin, wmax), "wdth": 100}, overlap=True
)
pinned.save(dest)
axes = [(a.axisTag, a.minValue, a.maxValue) for a in pinned["fvar"].axes] if "fvar" in pinned else []
print(f"pinned AnekLatin axes={axes}", file=sys.stderr)
PY
}

before=0
after=0
count=0
for font in "$FONT_DIR"/*.ttf "$FONT_DIR"/*.otf "$OUT"/makepad_widgets/resources/LiberationMono-Regular.ttf "$OUT"/makepad_widgets/resources/IBMPlexSans-Text.ttf "$OUT"/makepad_widgets/resources/IBMPlexSans-SemiBold.ttf; do
  [[ -f "$font" ]] || continue
  name="$(basename "$font")"
  ranges="U+0020-007E,$SHAPER_PARTS"
  case "$name" in
    Noto*) ;;                       # a script face: its Latin is never drawn
    *) ranges="$ranges,$LATIN_MORE" ;;
  esac
  src_for_subset="$font"
  if [[ "$name" == AnekLatin-Variable.ttf ]]; then
    if ! pin_anek_latin "$font" "$WORK/anek-pinned.ttf" > "$WORK/pin-log.txt" 2>&1; then
      echo "error: AnekLatin axis pin failed" >&2
      cat "$WORK/pin-log.txt" >&2
      exit 1
    fi
    src_for_subset="$WORK/anek-pinned.ttf"
  fi
  if ! pyftsubset "$src_for_subset" --output-file="$WORK/$name" \
      --text-file="$WORK/chars.txt" --unicodes="$ranges" \
      --layout-features='*' --notdef-outline --no-hinting --obfuscate-names \
      > "$WORK/log.txt" 2>&1; then
    echo "error: pyftsubset failed on $name" >&2
    cat "$WORK/log.txt" >&2
    exit 1
  fi
  was="$(stat -c %s "$font")"
  now="$(stat -c %s "$WORK/$name")"
  # A face that did not get smaller (already a subset) is left as it is.
  if (( now < was )); then
    cp "$WORK/$name" "$font"
  else
    now="$was"
  fi
  before=$(( before + was ))
  after=$(( after + now ))
  count=$(( count + 1 ))
done
if (( count == 0 )); then
  echo "error: no font matched in $FONT_DIR" >&2
  exit 1
fi
anek_kb=0
if [[ -f "$FONT_DIR/AnekLatin-Variable.ttf" ]]; then
  anek_kb=$(( $(stat -c %s "$FONT_DIR/AnekLatin-Variable.ttf") / 1024 ))
fi
echo "Subset $count fonts: $(( before / 1024 )) KB → $(( after / 1024 )) KB ($(LC_ALL=C.UTF-8 wc -m < "$WORK/chars.txt") characters in the sources); AnekLatin ${anek_kb} KB"

# Drop Makepad widget faces this site never declares (staging copies only).
WIDGETS="$OUT/makepad_widgets/resources"
if [[ -d "$WIDGETS" ]]; then
  dropped=0
  for junk in \
      NewCMMath-Regular.otf \
      NotoColorEmoji.ttf \
      NotoSans-Regular.ttf \
      jetbrains_mono_variable.ttf \
      IBMPlexSans-Italic.ttf \
      IBMPlexSans-BoldItalic.ttf \
      LXGWWenKaiBold.ttf \
      LXGWWenKaiRegular.ttf \
      fa-solid-900.ttf; do
    if [[ -f "$WIDGETS/$junk" ]]; then
      rm -f "$WIDGETS/$junk"
      dropped=$(( dropped + 1 ))
    fi
  done
  if [[ -d "$WIDGETS/icons" ]]; then
    rm -rf "$WIDGETS/icons"
    dropped=$(( dropped + 1 ))
  fi
  if (( dropped > 0 )); then
    echo "Dropped $dropped unused makepad_widgets resource(s) from staging"
  fi
fi
