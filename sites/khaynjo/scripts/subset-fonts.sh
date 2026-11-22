#!/usr/bin/env bash
# Shrink the fonts in a staged khaynjo.com tree to the characters the site draws.
#
# Every font a Makepad site declares is fetched before its first frame, and
# the files are whole typefaces: 22 files, 3.8 MB. This rewrites the
# staged copies — never the ones in the repo — so each keeps only
#   - every character that appears in sites/khaynjo/src/*.rs (all the site's copy,
#     tables and symbols live there), and
#   - plain Latin for anything typed or computed: Basic Latin everywhere, and
#     in the Latin faces also Latin-1, Latin Extended-A, general punctuation,
#     currency signs and the four arrows.
# Shaping tables are kept for the glyphs that stay, so conjuncts and joined
# forms still work. Names are replaced with placeholders (some of these
# faces reserve their name for the unmodified font).
#
# Variable display faces are pinned to the weight `ui.rs` draws (Baloo 2
# wght=720) before subsetting, so the staged file drops unused masters.
#
#   ./sites/khaynjo/scripts/subset-fonts.sh <staged public dir>
#
# Needs pyftsubset (fonttools). Without it the fonts stay whole: the site
# works the same and a first visit is heavier. Text a visitor types in a
# script the site's own copy does not use shows empty boxes — add a sample
# of it to the sources if that ever matters.
# Used by preview-web.sh and deploy-firebase.sh.
set -euo pipefail

OUT="${1:?usage: subset-fonts.sh <staged public dir>}"
SITE="$(cd "$(dirname "$0")/.." && pwd)"
FONT_DIR="$OUT/khaynjo_site/resources/fonts"
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
# Rust `\u{f04c}` escapes are four source characters, not the glyph. Pull
# those codepoints so FA solid (and any other PUA icons) survive subsetting
# — otherwise shelf play/pause chips paint □ tofu on the live site.
FA_RANGES=""
while read -r hex; do
  [[ -n "$hex" ]] || continue
  if [[ -n "$FA_RANGES" ]]; then
    FA_RANGES+=","
  fi
  FA_RANGES+="U+${hex}"
done < <(grep -ohE '\\u\{[0-9a-fA-F]+\}' "$SITE"/src/*.rs | sed -E 's/\\u\{([0-9a-fA-F]+)\}/\1/' | tr 'a-f' 'A-F' | sort -u)
if [[ -z "$FA_RANGES" ]]; then
  # Shelf chips in ui.rs / main.rs — keep even if grep misshapes.
  FA_RANGES='U+F04B,U+F04C,U+F053,U+F054,U+F067,U+F068'
fi

# Pin Baloo 2 to the instance ui.rs asks for (FontMember weight: 720.0).
if [[ -f "$FONT_DIR/Baloo2-Variable.ttf" ]] && command -v fonttools >/dev/null 2>&1; then
  before_pin="$(stat -c %s "$FONT_DIR/Baloo2-Variable.ttf")"
  if fonttools varLib.instancer "$FONT_DIR/Baloo2-Variable.ttf" \
      wght=720 \
      -o "$WORK/Baloo2-Variable.ttf" > "$WORK/pin.txt" 2>&1; then
    cp "$WORK/Baloo2-Variable.ttf" "$FONT_DIR/Baloo2-Variable.ttf"
    after_pin="$(stat -c %s "$FONT_DIR/Baloo2-Variable.ttf")"
    echo "Pinned Baloo2-Variable.ttf wght=720: $(( before_pin / 1024 )) KB → $(( after_pin / 1024 )) KB"
  else
    echo "note: Baloo2 pin skipped — $(tr '\n' ' ' < "$WORK/pin.txt")" >&2
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
    fa-solid*) ranges="$ranges,$FA_RANGES" ;;  # PUA icons from `\u{…}` in sources
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
