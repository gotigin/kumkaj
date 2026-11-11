#!/usr/bin/env bash
# Run khaynjo.com (Makepad fullpage) in the browser.
# Serves at http://127.0.0.1:8012 — open that URL after the build finishes.
# (kumkaj.com runs on :8010; served locally, each site links to the others'
# local ports instead of the public domains.)
#
# Builds debug wasm, injects the 375px min-width shell CSS, then serves.
# For a production-shaped release tree use ./preview-web.sh instead.
#
# Uses cargo-makepad from vendor/makepad-full so packaging matches the
# vendored Makepad the apps build against.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/khaynjo"
PKG="$ROOT/target/makepad-wasm-app/debug/khaynjo-site"
PORT="${PORT:-8012}"

cd "$ROOT"
cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
  wasm --no-threads build -p khaynjo-site "$@"

if [[ ! -f "$PKG/index.html" ]]; then
  echo "error: missing $PKG/index.html" >&2
  exit 1
fi

# Overlay static CSS + link it (min-width 375, overflow-x when narrower).
mkdir -p "$PKG/css"
rsync -a "$SITE/static/css/" "$PKG/css/" 2>/dev/null || true
"$SITE/scripts/patch-web-shell.sh" "$PKG/index.html"

echo "Serving $PKG at http://127.0.0.1:${PORT}/ (Ctrl+C to stop)"
echo "    Hard-refresh if an old tab still looks cached."
echo "    Edit → re-run this script (no live wasm hot-reload on this path)."
cd "$PKG"
exec python3 -m http.server "$PORT" --bind 127.0.0.1
