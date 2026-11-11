#!/usr/bin/env bash
# Build the Makepad khaynjo-site (wasm) + static shell (catalogue, privacy,
# terms, Analytics) and deploy ONE tree to Firebase Hosting → khaynjo.com.
#
# One-time: npm i -g firebase-tools && firebase login
# Domain: Firebase Console → Hosting → add khaynjo.com (project khaynjo-2b2a2)
#
#   ./sites/khaynjo/scripts/deploy-firebase.sh
#   ./sites/khaynjo/scripts/deploy-firebase.sh --dry-run
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/khaynjo"
OUT="$SITE/public"
WASM_PKG="$ROOT/target/makepad-wasm-app/release/khaynjo-site"
PROJECT="${FIREBASE_PROJECT:-}"
DRY=0

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY=1 ;;
    -h|--help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown arg: $arg" >&2
      exit 1
      ;;
  esac
done

export PATH="${HOME}/.local/bin:${PATH}"

if [[ ! -d "$ROOT/vendor/makepad/widgets" ]]; then
  echo "error: vendor/makepad missing — run ./scripts/setup-makepad.sh" >&2
  exit 1
fi
if ! command -v firebase >/dev/null 2>&1; then
  echo "error: firebase CLI not found (npm i -g firebase-tools)" >&2
  exit 1
fi

echo "1/4 Bundle Firebase Analytics → static/js/firebase-init.js"
(
  cd "$SITE"
  if [[ ! -d node_modules/firebase ]]; then
    npm install
  fi
  npm run build:firebase
)

echo "2/4 Build Makepad wasm (khaynjo-site, release, --no-threads)"
(
  cd "$ROOT"
  cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
    wasm --no-threads --strip --brotli build -p khaynjo-site --release
)

if [[ ! -f "$WASM_PKG/index.html" || ! -f "$WASM_PKG/khaynjo-site.wasm" ]]; then
  echo "error: wasm package incomplete at $WASM_PKG" >&2
  exit 1
fi

echo "3/4 Stage Firebase public/ = wasm package + static shell"
# Refresh public catalogue (items shelf + starters) into static/ before overlay.
"$ROOT/apps/khaynjo/scripts/publish-catalogue.sh"
rm -rf "$OUT"
mkdir -p "$OUT"
# Makepad package (index.html, wasm, makepad_platform/, resources/, …)
rsync -a --delete "$WASM_PKG/" "$OUT/"
# Site static shell overlays (catalogue, terms, js, media, css)
rsync -a "$SITE/static/" "$OUT/"
"$SITE/scripts/patch-web-shell.sh" "$OUT/index.html"

# Same PRIVACY.md the app embeds — formatted presentation (not source view).
PRIVACY_MD="$ROOT/apps/khaynjo/docs/PRIVACY.md"
PRIVACY_TMPL="$SITE/privacy-template.html"
if [[ ! -f "$PRIVACY_MD" ]]; then
  echo "error: missing $PRIVACY_MD" >&2
  exit 1
fi
if ! command -v pandoc >/dev/null 2>&1; then
  echo "error: pandoc required to present PRIVACY.md at /privacy" >&2
  exit 1
fi
mkdir -p "$OUT/privacy"
pandoc "$PRIVACY_MD" \
  -f markdown \
  -t html5 \
  --template="$PRIVACY_TMPL" \
  --metadata title="Privacy — Khaynjo" \
  -V lang=en \
  -o "$OUT/privacy/index.html"
# Keep the source file available, but /privacy is the formatted page.
cp "$PRIVACY_MD" "$OUT/privacy.md"
echo "Presented /privacy from apps/khaynjo/docs/PRIVACY.md (formatted HTML + privacy.md source)"

if [[ ! -f "$OUT/v1/catalogue/public/index.json" ]]; then
  echo "error: catalogue missing in $OUT/v1/catalogue/" >&2
  exit 1
fi
if [[ ! -f "$OUT/js/firebase-init.js" ]]; then
  echo "error: missing $OUT/js/firebase-init.js" >&2
  exit 1
fi
if [[ ! -f "$OUT/privacy/index.html" ]]; then
  echo "error: privacy page missing after pandoc" >&2
  exit 1
fi

# Crawl/Analytics shell on the Makepad index (same document as the canvas).
# Cache-bust query so browsers fetch new wasm/JS after each deploy (filenames
# stay stable; HTML is already max-age=0).
BUILD_ID="$(date -u +%Y%m%d%H%M%S)-$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo local)"
INDEX="$OUT/index.html"
TMP="$OUT/index.html.tmp"
HEAD_INJECT='        <title>Khaynjo</title>
        <meta name="description" content="Lunch rush. No internet. No problem. — restaurant OS on your own devices.">
        <meta name="khaynjo-build" content="'"$BUILD_ID"'">
        <link rel="canonical" href="https://khaynjo.com/">
        <meta property="og:title" content="Khaynjo">
        <meta property="og:description" content="From table to kitchen to bill — on your own devices.">
        <meta property="og:url" content="https://khaynjo.com/">
        <meta property="og:type" content="website">'
BODY_INJECT='        <script src="./js/firebase-init.js?v='"$BUILD_ID"'" defer></script>
        <noscript><p>Khaynjo — restaurant operating system. <a href="/privacy">Privacy</a> · <a href="/terms/">Terms</a> · <a href="/download">Download</a></p></noscript>
    </body>'
awk -v head="$HEAD_INJECT" -v body="$BODY_INJECT" -v bid="$BUILD_ID" '
  BEGIN { done_title=0; done_body=0 }
  /<title>khaynjo-site<\/title>/ && !done_title {
    print head
    done_title=1
    next
  }
  {
    gsub(/\.\/makepad_platform\/web_gl\.js/, "./makepad_platform/web_gl.js?v=" bid)
    gsub(/\.\/khaynjo-site\.wasm/, "./khaynjo-site.wasm?v=" bid)
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
echo "Patched $INDEX (title/OG + Analytics + noscript + cache-bust v=$BUILD_ID)"

echo "4/4 firebase deploy --only hosting"
FB_ARGS=(deploy --only hosting)
if [[ -n "$PROJECT" ]]; then
  FB_ARGS+=(--project "$PROJECT")
fi
(
  cd "$SITE"
  if [[ "$DRY" -eq 1 ]]; then
    firebase "${FB_ARGS[@]}" --dry-run
  else
    firebase "${FB_ARGS[@]}"
  fi
)

echo "Done → https://khaynjo.com/ (Makepad wasm + /v1/catalogue/ + /privacy /terms)"
echo "App catalogue test: KHAYNJO_CATALOGUE_DIR=$SITE/static/v1/catalogue"
