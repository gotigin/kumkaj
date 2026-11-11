#!/usr/bin/env bash
# Build the same release wasm tree deploy uses, stage it under public/, and
# serve on http://127.0.0.1:8012 — so you can catch layout/cache/content bugs
# before Firebase (no hosting cost).
#
#   ./sites/khaynjo/scripts/preview-web.sh
#   ./sites/khaynjo/scripts/preview-web.sh --no-build   # re-serve last public/
#
# Faster iteration (debug wasm, hot rebuild): ./sites/khaynjo/scripts/run-web.sh
# Native window (fastest): ./sites/khaynjo/scripts/run-desktop.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/khaynjo"
OUT="$SITE/public"
WASM_PKG="$ROOT/target/makepad-wasm-app/release/khaynjo-site"
PORT="${PORT:-8012}"
DO_BUILD=1

for arg in "$@"; do
  case "$arg" in
    --no-build) DO_BUILD=0 ;;
    -h|--help)
      sed -n '2,14p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown arg: $arg" >&2
      exit 1
      ;;
  esac
done

if [[ "$DO_BUILD" -eq 1 ]]; then
  echo "1/3 Build Makepad wasm (khaynjo-site, release, --no-threads)"
  (
    cd "$ROOT"
    cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
      wasm --no-threads --strip build -p khaynjo-site --release
  )
  if [[ ! -f "$WASM_PKG/index.html" || ! -f "$WASM_PKG/khaynjo-site.wasm" ]]; then
    echo "error: wasm package incomplete at $WASM_PKG" >&2
    exit 1
  fi

  echo "2/3 Stage $OUT (= wasm + static shell, same as deploy minus catalogue publish)"
  rm -rf "$OUT"
  mkdir -p "$OUT"
  rsync -a --delete "$WASM_PKG/" "$OUT/"
  rsync -a "$SITE/static/" "$OUT/"

  BUILD_ID="preview-$(date -u +%Y%m%d%H%M%S)"
  INDEX="$OUT/index.html"
  TMP="$OUT/index.html.tmp"
  awk -v bid="$BUILD_ID" '
    {
      gsub(/\.\/makepad_platform\/web_gl\.js/, "./makepad_platform/web_gl.js?v=" bid)
      gsub(/\.\/khaynjo-site\.wasm/, "./khaynjo-site.wasm?v=" bid)
      print
    }
  ' "$INDEX" > "$TMP"
  mv "$TMP" "$INDEX"
  printf '%s\n' "$BUILD_ID" > "$OUT/build-id.txt"
  echo "Staged build-id=$BUILD_ID"
else
  if [[ ! -f "$OUT/index.html" ]]; then
    echo "error: $OUT missing — run without --no-build first" >&2
    exit 1
  fi
fi

echo "3/3 Serve $OUT at http://127.0.0.1:${PORT}/ (Ctrl+C to stop)"
echo "    Open that URL in a private window if an old tab still looks cached."
cd "$OUT"
exec python3 -m http.server "$PORT" --bind 127.0.0.1
