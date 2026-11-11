#!/usr/bin/env bash
# Inject site-shell.css (375px min-width + horizontal scroll) into a staged
# Makepad wasm index.html. Used by deploy-firebase.sh and preview-web.sh.
set -euo pipefail
INDEX="${1:?usage: patch-web-shell.sh <path/to/index.html>}"
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CSS_REL="./css/site-shell.css"
CSS_SRC="$ROOT/sites/khaynjo/static/css/site-shell.css"
OUT_DIR="$(cd "$(dirname "$INDEX")" && pwd)"

if [[ ! -f "$INDEX" ]]; then
  echo "error: missing $INDEX" >&2
  exit 1
fi
if [[ ! -f "$CSS_SRC" ]]; then
  echo "error: missing $CSS_SRC" >&2
  exit 1
fi

mkdir -p "$OUT_DIR/css"
cp -f "$CSS_SRC" "$OUT_DIR/css/site-shell.css"

# Viewport: keep device-width; shell CSS owns min-width + overflow-x.
# Drop auto_reload.js — it polls /$watch, which only cargo-makepad's
# hot-reload server implements (python http.server / Firebase 404 forever).
TMP="$INDEX.tmp"
awk -v css="$CSS_REL" '
  /auto_reload\.js/ { next }
  /full_canvas\.css/ {
    print
    if (!css_done) {
      print "        <link rel='\''stylesheet'\'' type='\''text/css'\'' href='\''" css "'\''>"
      css_done=1
    }
    next
  }
  { print }
' "$INDEX" > "$TMP"
mv "$TMP" "$INDEX"
echo "Patched $INDEX (+ site-shell.css, - auto_reload.js)"
