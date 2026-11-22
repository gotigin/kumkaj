#!/usr/bin/env bash
# Build the same release wasm tree deploy uses, stage it under public/, and
# serve on http://127.0.0.1:8010 — catch layout/cache/content bugs
# before Firebase (no hosting cost).
#
#   ./sites/kumkaj/scripts/preview-web.sh
#   ./sites/kumkaj/scripts/preview-web.sh --no-build
#
# Faster iteration (debug wasm): ./sites/kumkaj/scripts/run-web.sh
# Native window: ./sites/kumkaj/scripts/run-desktop.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/kumkaj"
OUT="$SITE/public"
WASM_PKG="$ROOT/target/makepad-wasm-app/release/kumkaj-site"
PORT="${PORT:-8010}"
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
  local PRIVACY_MD="$ROOT/sites/kumkaj/PRIVACY.md"
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
    --metadata title="Privacy — Kumkaj" \
    -V lang=en \
    -V preview_v="$PREVIEW_V" \
    -o "$OUT/privacy/index.html"
  cp "$PRIVACY_MD" "$OUT/privacy.md"
  echo "Presented /privacy from $PRIVACY_MD"
}
present_cancellation() {
  local CANCELLATION_MD="$ROOT/sites/kumkaj/CANCELLATION.md"
  local CANCELLATION_TMPL="$SITE/cancellation-template.html"
  if [[ ! -f "$CANCELLATION_MD" ]]; then
    echo "error: missing $CANCELLATION_MD" >&2
    exit 1
  fi
  if [[ ! -f "$CANCELLATION_TMPL" ]]; then
    echo "error: missing $CANCELLATION_TMPL" >&2
    exit 1
  fi
  mkdir -p "$OUT/cancellation"
  pandoc "$CANCELLATION_MD" \
    -f markdown \
    -t html5 \
    --template="$CANCELLATION_TMPL" \
    --metadata title="Cancellation — Kumkaj" \
    -V lang=en \
    -V preview_v="$PREVIEW_V" \
    -o "$OUT/cancellation/index.html"
  cp "$CANCELLATION_MD" "$OUT/cancellation.md"
  echo "Presented /cancellation from $CANCELLATION_MD"
}



if [[ "$DO_BUILD" -eq 1 ]]; then
  echo "1/3 Build Makepad wasm (kumkaj-site, release, --no-threads)"
  (
    cd "$ROOT"
    cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
      wasm --no-threads --strip build -p kumkaj-site --release
  )
  if [[ ! -f "$WASM_PKG/index.html" || ! -f "$WASM_PKG/kumkaj-site.wasm" ]]; then
    echo "error: wasm package incomplete at $WASM_PKG" >&2
    exit 1
  fi

  echo "2/3 Stage $OUT (= wasm + static shell)"
  rm -rf "$OUT"
  mkdir -p "$OUT"
  # cargo-makepad writes a .br twin beside every file; Firebase Hosting
  # compresses on its own and never serves those.
  rsync -a --delete --exclude='*.br' "$WASM_PKG/" "$OUT/"
  rsync -a "$SITE/static/" "$OUT/"
  "$SITE/scripts/patch-web-shell.sh" "$OUT/index.html"
  # Fonts: only the characters this site draws (the repo keeps them whole).
  "$SITE/scripts/subset-fonts.sh" "$OUT"
  PREVIEW="$SITE/static/media/link-preview.jpg"
  if [[ ! -f "$PREVIEW" ]]; then
    echo "error: missing sites/kumkaj/static/media/link-preview.jpg — run sites/kumkaj/scripts/make-link-preview.sh" >&2
    exit 1
  fi
  PREVIEW_V="$(sha256sum "$PREVIEW" | cut -c1-10)"
  PREVIEW_URL="https://kumkaj.com/media/link-preview.jpg?v=$PREVIEW_V"
  present_privacy
  present_cancellation

  # Cache-bust og/twitter image on secondary shells (same sha10 as home).
  for secondary in privacy terms cancellation account shop; do
    f="$OUT/$secondary/index.html"
    [[ -f "$f" ]] || continue
    sed -i -E "s#(https://kumkaj\.com/media/link-preview\.jpg)(\?v=[0-9a-f]+)?\"#\1?v=${PREVIEW_V}\"#g" "$f"
  done


  BUILD_ID="preview-$(date -u +%Y%m%d%H%M%S)"
  INDEX="$OUT/index.html"
  TMP="$OUT/index.html.tmp"
  # Title, description and link-preview tags live in shell/head.html (one copy
  # for deploy and preview). Crawlers and chat apps read this HTML only — they
  # never run the wasm.
  # PREVIEW_URL already set above (sha10 of link-preview.jpg).
  HEAD_INJECT="$(sed -e "s|__BUILD_ID__|$BUILD_ID|g" -e "s|__PREVIEW_URL__|$PREVIEW_URL|g" "$SITE/shell/head.html")"
  BODY_INJECT='        <noscript><p>Kumkaj — business operating systems by Gotigin. <a href="/privacy">Privacy</a> · <a href="/terms/">Terms</a> · <a href="/cancellation">Cancellation</a></p></noscript>
    </body>'
  awk -v head="$HEAD_INJECT" -v body="$BODY_INJECT" -v bid="$BUILD_ID" '
    BEGIN { done_title=0; done_body=0 }
    /<title>kumkaj-site<\/title>/ && !done_title {
      print head
      done_title=1
      next
    }
    {
      gsub(/\.\/makepad_platform\/web_gl\.js/, "./makepad_platform/web_gl.js?v=" bid)
      gsub(/\.\/kumkaj-site\.wasm/, "./kumkaj-site.wasm?v=" bid)
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
