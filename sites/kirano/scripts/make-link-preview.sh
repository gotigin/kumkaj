#!/usr/bin/env bash
# Take the picture that chat apps and social sites show when someone pastes
# a kirano.shop link (WhatsApp, X, LinkedIn, Facebook, Telegram, Discord,
# Slack, iMessage … all read the same Open Graph tags).
#
#   ./sites/kirano/scripts/make-link-preview.sh [url]
#
# url defaults to the live site. After a change to the first screen, run
# ./sites/kirano/scripts/preview-web.sh in another terminal and pass
# http://127.0.0.1:8013/ so the picture shows the new build, then deploy.
#
# Writes static/media/link-preview.jpg: the real first screen at 1200x630
# (the size every one of those apps accepts for a large preview), as a JPEG
# under 300 KB (WhatsApp drops bigger pictures; some apps cannot read WebP).
# Needs chromium (or google-chrome) and ImageMagick.
set -euo pipefail

SITE="$(cd "$(dirname "$0")/.." && pwd)"
URL="${1:-https://kirano.shop/}"
OUT="$SITE/static/media/link-preview.jpg"
W=1200
H=630
MAX_BYTES=300000

BROWSER=""
for b in chromium chromium-browser google-chrome google-chrome-stable; do
  if command -v "$b" >/dev/null 2>&1; then BROWSER="$b"; break; fi
done
if [[ -z "$BROWSER" ]]; then
  echo "error: chromium or google-chrome required" >&2
  exit 1
fi
if ! command -v magick >/dev/null 2>&1; then
  echo "error: ImageMagick (magick) required" >&2
  exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# The page is a WebGL canvas: software GL so it also paints on a machine with
# no display. The time budget covers the wasm download and the first frames.
"$BROWSER" --headless=new --no-first-run --user-data-dir="$WORK/profile" \
  --hide-scrollbars --use-angle=swiftshader --enable-unsafe-swiftshader \
  --ignore-gpu-blocklist --force-device-scale-factor=1 \
  --window-size="${W},${H}" --virtual-time-budget=25000 \
  --screenshot="$WORK/shot.png" "$URL" >/dev/null 2>&1 || true
if [[ ! -s "$WORK/shot.png" ]]; then
  echo "error: no screenshot from $URL" >&2
  exit 1
fi
if [[ "$(magick identify -format '%wx%h' "$WORK/shot.png")" != "${W}x${H}" ]]; then
  echo "error: screenshot is not ${W}x${H}" >&2
  exit 1
fi
# A screen still showing the start-up ring, or a blank one, has very few colours.
COLOURS="$(magick "$WORK/shot.png" -format '%k' info:)"
if (( COLOURS < 400 )); then
  echo "error: $URL had not painted (only $COLOURS colours) — is the server up?" >&2
  exit 1
fi

# Whatever the fold cuts through fades into the page instead of ending in
# half a line: the last 72 px blend into the most common colour of the last rows.
# (awk reads the whole histogram: cutting a pipe short with head fails the
# script under pipefail when the last rows hold many colours.)
BOTTOM="$(magick "$WORK/shot.png" -crop "${W}x6+0+$((H - 6))" +repage -format '%c' histogram:info: \
  | awk '{ n = $1 + 0; if (n > max) { max = n; line = $0 } }
         END { if (match(line, /#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]/)) print substr(line, RSTART, RLENGTH) }')"
BOTTOM="${BOTTOM:-#000000}"
magick "$WORK/shot.png" \
  \( -size "${W}x72" "gradient:none-${BOTTOM}" \) -gravity south -composite \
  "$WORK/faded.png"

mkdir -p "$(dirname "$OUT")"
for q in 88 84 80 74 68 60; do
  magick "$WORK/faded.png" -strip -interlace Plane -sampling-factor 4:2:0 -quality "$q" "$WORK/out.jpg"
  if (( $(stat -c %s "$WORK/out.jpg") <= MAX_BYTES )); then break; fi
done
if (( $(stat -c %s "$WORK/out.jpg") > MAX_BYTES )); then
  echo "error: picture is still over $MAX_BYTES bytes at quality 60" >&2
  exit 1
fi
mv "$WORK/out.jpg" "$OUT"
echo "Wrote ${OUT#"$SITE"/} from $URL ($(stat -c %s "$OUT") bytes, ${W}x${H})"
