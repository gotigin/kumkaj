#!/usr/bin/env bash
# Inject site-shell.css (320px min-width + horizontal scroll), the Android
# Chrome Desktop-site / zoom guard, and the start-up screen into a staged
# Makepad wasm index.html. Used by deploy-firebase.sh and preview-web.sh.
set -euo pipefail
INDEX="${1:?usage: patch-web-shell.sh <path/to/index.html>}"
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE_NAME="moklanjo"
CSS_REL="./css/site-shell.css"
CSS_SRC="$ROOT/sites/$SITE_NAME/static/css/site-shell.css"
GUARD_REL="./js/mobile-viewport-guard.js"
GUARD_SRC="$ROOT/sites/$SITE_NAME/static/js/mobile-viewport-guard.js"
OUT_DIR="$(cd "$(dirname "$INDEX")" && pwd)"

if [[ ! -f "$INDEX" ]]; then
  echo "error: missing $INDEX" >&2
  exit 1
fi
if [[ ! -f "$CSS_SRC" ]]; then
  echo "error: missing $CSS_SRC" >&2
  exit 1
fi
if [[ ! -f "$GUARD_SRC" ]]; then
  echo "error: missing $GUARD_SRC" >&2
  exit 1
fi

mkdir -p "$OUT_DIR/css" "$OUT_DIR/js"
cp -f "$CSS_SRC" "$OUT_DIR/css/site-shell.css"
cp -f "$GUARD_SRC" "$OUT_DIR/js/mobile-viewport-guard.js"

# Viewport: keep device-width; shell CSS owns min-width + overflow-x.
# Drop auto_reload.js — it polls /$watch, which only cargo-makepad's
# hot-reload server implements (python http.server / Firebase 404 forever).
TMP="$INDEX.tmp"
awk -v css="$CSS_REL" -v guard="$GUARD_REL" '
  /<html(>|[[:space:]])/ { print "    <html lang=\"en\">"; next }
  /auto_reload\.js/ { next }
  /name="viewport"/ {
    print "        <meta name='\''viewport'\'' content='\''width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover'\'' />"
    next
  }
  /full_canvas\.css/ {
    print
    if (!css_done) {
      print "        <link rel='\''stylesheet'\'' type='\''text/css'\'' href='\''" css "'\''>"
      css_done=1
    }
    next
  }
  /<\/head>/ {
    if (!guard_done) {
      print "        <script src='\''" guard "'\''></script>"
      guard_done=1
    }
  }
  { print }
' "$INDEX" > "$TMP"
mv "$TMP" "$INDEX"
echo "Patched $INDEX (+ site-shell.css, + mobile-viewport-guard.js, - auto_reload.js)"

# The start-up screen (logo, progress ring) replaces cargo-makepad's grey
# "Loading.." bar: shell/boot.{css,html,js}, inlined.
"$(dirname "$0")/patch-boot-screen.sh" "$INDEX"
