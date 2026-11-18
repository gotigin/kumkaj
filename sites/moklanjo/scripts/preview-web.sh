#!/usr/bin/env bash
# Build the same release wasm tree deploy uses, stage it under public/, and
# serve on http://127.0.0.1:8011 — catch layout/cache/content bugs
# before Firebase (no hosting cost).
#
#   ./sites/moklanjo/scripts/preview-web.sh
#   ./sites/moklanjo/scripts/preview-web.sh --no-build
#
# Faster iteration (debug wasm): ./sites/moklanjo/scripts/run-web.sh
# Native window: ./sites/moklanjo/scripts/run-desktop.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/moklanjo"
OUT="$SITE/public"
WASM_PKG="$ROOT/target/makepad-wasm-app/release/moklanjo-site"
PORT="${PORT:-8011}"
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

present_privacy() {
  local PRIVACY_MD="$ROOT/apps/moklanjo/docs/PRIVACY.md"
  local PRIVACY_TMPL="$SITE/privacy-template.html"
  if [[ ! -f "$PRIVACY_MD" ]]; then
    echo "error: missing $PRIVACY_MD" >&2
    exit 1
  fi
  if ! command -v pandoc >/dev/null 2>&1; then
    echo "error: pandoc required to present privacy at /privacy" >&2
    exit 1
  fi
  mkdir -p "$OUT/privacy"
  pandoc "$PRIVACY_MD" \
    -f markdown \
    -t html5 \
    --template="$PRIVACY_TMPL" \
    --metadata title="Privacy — Moklanjo" \
    -V lang=en \
    -o "$OUT/privacy/index.html"
  cp "$PRIVACY_MD" "$OUT/privacy.md"
  echo "Presented /privacy from $PRIVACY_MD"
}

if [[ "$DO_BUILD" -eq 1 ]]; then
  echo "1/3 Build Makepad wasm (moklanjo-site, release, --no-threads)"
  (
    cd "$ROOT"
    cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
      wasm --no-threads --strip build -p moklanjo-site --release
  )
  if [[ ! -f "$WASM_PKG/index.html" || ! -f "$WASM_PKG/moklanjo-site.wasm" ]]; then
    echo "error: wasm package incomplete at $WASM_PKG" >&2
    exit 1
  fi

  echo "2/3 Stage $OUT (= wasm + static shell)"
  rm -rf "$OUT"
  mkdir -p "$OUT"
  rsync -a --delete "$WASM_PKG/" "$OUT/"
  rsync -a "$SITE/static/" "$OUT/"
  "$SITE/scripts/patch-web-shell.sh" "$OUT/index.html"
  present_privacy

  BUILD_ID="preview-$(date -u +%Y%m%d%H%M%S)"
  INDEX="$OUT/index.html"
  TMP="$OUT/index.html.tmp"
  HEAD_INJECT='        <title>Moklanjo</title>
        <meta name="description" content="Book. Label. Hand over. Track. Settle. One desk, internet or not.">
        <meta name="moklanjo-build" content="'"$BUILD_ID"'">
        <link rel="canonical" href="https://moklanjo.com/">
        <meta property="og:title" content="Moklanjo">
        <meta property="og:description" content="Book. Label. Hand over. Track. Settle. One desk, internet or not.">
        <meta property="og:url" content="https://moklanjo.com/">
        <meta property="og:type" content="website">'
  BODY_INJECT='        <noscript><p>Moklanjo — courier desk OS. <a href="/privacy">Privacy</a> · <a href="/terms/">Terms</a> · <a href="/download">Download</a></p></noscript>
    </body>'
  awk -v head="$HEAD_INJECT" -v body="$BODY_INJECT" -v bid="$BUILD_ID" '
    BEGIN { done_title=0; done_body=0 }
    /<title>moklanjo-site<\/title>/ && !done_title {
      print head
      done_title=1
      next
    }
    {
      gsub(/\.\/makepad_platform\/web_gl\.js/, "./makepad_platform/web_gl.js?v=" bid)
      gsub(/\.\/moklanjo-site\.wasm/, "./moklanjo-site.wasm?v=" bid)
    }
    /<\/body>/ && !done_body {
      print body
      done_body=1
      next
    }
    { print }
    END {
      if (!done_title) exit 2
      if (!done_body) exit 3
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
