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
# Variable display faces are pinned to the weight `ui.rs` draws (Fraunces
# wght=620) before subsetting, so the staged file drops unused masters.
#
#   ./sites/kumkaj/scripts/subset-fonts.sh <staged public dir>
#
# Needs pyftsubset (fonttools). Without it the fonts stay whole: the site
# works the same and a first visit is heavier. Text a visitor types in a
# script the site's own copy does not use shows empty boxes — add a sample
# of it to the sources if that ever matters.
# Used by preview-web.sh and deploy-firebase.sh.
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
# Site copy (src/*.rs) and every language pack (translations/*.tsv): the script
# faces keep exactly the characters the translated page can draw.
cat "$SITE"/src/*.rs "$SITE"/translations/*.tsv | LC_ALL=C.UTF-8 grep -o . | LC_ALL=C.UTF-8 sort -u | tr -d '\n' > "$WORK/chars.txt"
if [[ "$(LC_ALL=C.UTF-8 wc -m < "$WORK/chars.txt")" -lt 60 ]]; then
  echo "error: could not read the site's characters from $SITE/src" >&2
  exit 1
fi

# Pin Fraunces to the instance ui.rs asks for (FontMember weight: 620.0).
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
  if ! pyftsubset "$font" --output-file="$WORK/$name" \
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
