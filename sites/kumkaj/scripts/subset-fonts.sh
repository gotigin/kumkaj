#!/usr/bin/env bash
# Shrink the fonts in a staged kumkaj.com tree to the characters the site draws.
#
# Every font a Makepad site declares is fetched before its first frame, and
# the files are whole typefaces: 8 files, 1.5 MB. This rewrites the
# staged copies — never the ones in the repo — so each keeps only
#   - every character that appears in sites/kumkaj/src/*.rs (all the site's copy,
#     tables and symbols live there), and
#   - plain Latin for anything typed or computed: Basic Latin everywhere, and
#     in the Latin faces also Latin-1, Latin Extended-A, general punctuation,
#     currency signs and the four arrows.
# Shaping tables are kept for the glyphs that stay, so conjuncts and joined
# forms still work. Names are replaced with placeholders (some of these
# faces reserve their name for the unmodified font).
#
#   ./sites/kumkaj/scripts/subset-fonts.sh <staged public dir>
#
# Needs pyftsubset (fonttools). Without it the fonts stay whole: the site
# works the same and a first visit is heavier. Text a visitor types in a
# script the site's own copy does not use shows empty boxes — add a sample
# of it to the sources if that ever matters.
# Used by preview-web.sh and deploy-firebase.sh.
#
# Variable display faces are pinned to the weight `ui.rs` draws (Fraunces
# wght=620) before subsetting, so the staged file drops unused masters.
set -euo pipefail

OUT="${1:?usage: subset-fonts.sh <staged public dir>}"
SITE="$(cd "$(dirname "$0")/.." && pwd)"
FONT_DIR="$OUT/kumkaj_site/resources/fonts"
LATIN_MORE='U+00A0-00FF,U+0100-017F,U+2010-2027,U+2030-203A,U+20A0-20BF,U+2190-2193'

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
cat "$SITE"/src/*.rs | LC_ALL=C.UTF-8 grep -o . | LC_ALL=C.UTF-8 sort -u | tr -d '\n' > "$WORK/chars.txt"
if [[ "$(LC_ALL=C.UTF-8 wc -m < "$WORK/chars.txt")" -lt 60 ]]; then
  echo "error: could not read the site's characters from $SITE/src" >&2
  exit 1
fi

# Pin variable display to the instance ui.rs asks for (weight: 620.0).
if [[ -f "$FONT_DIR/Fraunces-Variable.ttf" ]] && command -v fonttools >/dev/null 2>&1; then
  before_pin="$(stat -c %s "$FONT_DIR/Fraunces-Variable.ttf")"
  if fonttools varLib.instancer "$FONT_DIR/Fraunces-Variable.ttf" \
      wght=620 opsz=9 SOFT=0 WONK=1 \
      -o "$WORK/Fraunces-Variable.ttf" > "$WORK/pin.txt" 2>&1; then
    cp "$WORK/Fraunces-Variable.ttf" "$FONT_DIR/Fraunces-Variable.ttf"
    after_pin="$(stat -c %s "$FONT_DIR/Fraunces-Variable.ttf")"
    echo "Pinned Fraunces-Variable.ttf wght=620: $(( before_pin / 1024 )) KB → $(( after_pin / 1024 )) KB"
  else
    echo "note: Fraunces pin skipped — $(tr '\n' ' ' < "$WORK/pin.txt")" >&2
  fi
fi

# Pin variable display faces to the weight ui.rs asks for (Fraunces
# FontMember weight: 620) before glyph subsetting — variation tables
# otherwise stay huge on the wire (docs/WEBSITES.md § fonts).
pin_variable_display() {
  local font="$1" pinned="$2"
  case "$(basename "$font")" in
    Fraunces-Variable.ttf)
      python3 - "$font" "$pinned" <<'PY'
import sys
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer
src, dst = sys.argv[1], sys.argv[2]
font = TTFont(src)
if "fvar" not in font:
    font.save(dst)
else:
    instancer.instantiateVariableFont(
        font, {"wght": 620, "opsz": 9, "SOFT": 0, "WONK": 0}
    ).save(dst)
PY
      ;;
    *)
      cp "$font" "$pinned"
      ;;
  esac
}

before=0
after=0
count=0
for font in "$FONT_DIR"/*.ttf "$FONT_DIR"/*.otf "$OUT"/makepad_widgets/resources/LiberationMono-Regular.ttf "$OUT"/makepad_widgets/resources/IBMPlexSans-Text.ttf "$OUT"/makepad_widgets/resources/IBMPlexSans-SemiBold.ttf; do
  [[ -f "$font" ]] || continue
  name="$(basename "$font")"
  ranges='U+0020-007E'
  case "$name" in
    Noto*) ;;                       # a script face: its Latin is never drawn
    *) ranges="$ranges,$LATIN_MORE" ;;
  esac
  pin_variable_display "$font" "$WORK/pinned-$name"
  if ! pyftsubset "$WORK/pinned-$name" --output-file="$WORK/$name" \
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
echo "Subset $count fonts: $(( before / 1024 )) KB → $(( after / 1024 )) KB ($(LC_ALL=C.UTF-8 wc -m < "$WORK/chars.txt") characters in the sources)"
