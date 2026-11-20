#!/usr/bin/env bash
# Build the Makepad kirano-site (wasm) + static shell (privacy, terms, OG)
# and deploy ONE tree to Firebase Hosting → kirano.shop.
#
# One-time: npm i -g firebase-tools && firebase login
# Create a Firebase project, then either:
#   echo '{ "projects": { "default": "YOUR_PROJECT_ID" } }' > sites/kirano/.firebaserc
#   or: FIREBASE_PROJECT=YOUR_PROJECT_ID ./sites/kirano/scripts/deploy-firebase.sh
# Domain: Firebase Console → Hosting → add kirano.shop
#
#   ./sites/kirano/scripts/deploy-firebase.sh
#   ./sites/kirano/scripts/deploy-firebase.sh --dry-run
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/kirano"
OUT="$SITE/public"
WASM_PKG="$ROOT/target/makepad-wasm-app/release/kirano-site"
PROJECT="${FIREBASE_PROJECT:-}"
DRY=0

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY=1 ;;
    -h|--help)
      sed -n '2,16p' "$0"
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
if [[ -z "$PROJECT" ]]; then
  if [[ -f "$SITE/.firebaserc" ]] && grep -q '"default"' "$SITE/.firebaserc" 2>/dev/null; then
    :
  else
    echo "error: no Firebase project configured for kirano.shop." >&2
    echo "  Create a project, then set sites/kirano/.firebaserc default" >&2
    echo "  or pass FIREBASE_PROJECT=… (owner step — domains are registered)." >&2
    exit 1
  fi
fi

echo "1/4 Bundle Firebase Auth → static/js/firebase-init.js + account.js"
(
  cd "$SITE"
  if [[ ! -d node_modules/firebase ]]; then
    npm install
  fi
  npm run build
)

echo "2/4 Build Makepad wasm (kirano-site, release, --no-threads)"
(
  cd "$ROOT"
  cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
    wasm --no-threads --strip --brotli build -p kirano-site --release
)

if [[ ! -f "$WASM_PKG/index.html" || ! -f "$WASM_PKG/kirano-site.wasm" ]]; then
  echo "error: wasm package incomplete at $WASM_PKG" >&2
  exit 1
fi

echo "3/4 Stage Firebase public/ = wasm package + static shell"
rm -rf "$OUT"
mkdir -p "$OUT"
rsync -a --delete "$WASM_PKG/" "$OUT/"
rsync -a "$SITE/static/" "$OUT/"
"$SITE/scripts/patch-web-shell.sh" "$OUT/index.html"

PRIVACY_MD="$ROOT/apps/kirano/docs/PRIVACY.md"
PRIVACY_TMPL="$SITE/privacy-template.html"
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
  --metadata title="Privacy — Kirano" \
  -V lang=en \
  -o "$OUT/privacy/index.html"
cp "$PRIVACY_MD" "$OUT/privacy.md"
echo "Presented /privacy from $PRIVACY_MD"

if [[ ! -f "$OUT/privacy/index.html" ]]; then
  echo "error: privacy page missing after pandoc" >&2
  exit 1
fi
if [[ ! -f "$OUT/terms/index.html" ]]; then
  echo "error: missing $OUT/terms/index.html" >&2
  exit 1
fi
if [[ ! -f "$OUT/js/firebase-init.js" ]]; then
  echo "error: missing $OUT/js/firebase-init.js" >&2
  exit 1
fi
if [[ ! -f "$OUT/js/account.js" ]]; then
  echo "error: missing $OUT/js/account.js" >&2
  exit 1
fi
if [[ ! -f "$OUT/account/index.html" ]]; then
  echo "error: missing $OUT/account/index.html" >&2
  exit 1
fi

BUILD_ID="$(date -u +%Y%m%d%H%M%S)-$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo local)"
INDEX="$OUT/index.html"
TMP="$OUT/index.html.tmp"
HEAD_INJECT='        <title>Kirano</title>
        <meta name="description" content="Busy counter. No internet. Still billing. — grocery OS on your own devices.">
        <meta name="kirano-build" content="'"$BUILD_ID"'">
        <link rel="canonical" href="https://kirano.shop/">
        <meta property="og:title" content="Kirano">
        <meta property="og:description" content="Busy counter. No internet. Still billing. — grocery OS on your own devices.">
        <meta property="og:url" content="https://kirano.shop/">
        <meta property="og:type" content="website">'
BODY_INJECT='        <noscript><p>Kirano — grocery counter OS. <a href="/privacy">Privacy</a> · <a href="/terms/">Terms</a> · <a href="/download">Download</a></p></noscript>
    </body>'
BODY_INJECT='        <script src="./js/firebase-init.js?v='"$BUILD_ID"'" defer></script>
'"$BODY_INJECT"
awk -v head="$HEAD_INJECT" -v body="$BODY_INJECT" -v bid="$BUILD_ID" '
  BEGIN { done_title=0; done_body=0 }
  /<title>kirano-site<\/title>/ && !done_title {
    print head
    done_title=1
    next
  }
  {
    gsub(/\.\/makepad_platform\/web_gl\.js/, "./makepad_platform/web_gl.js?v=" bid)
    gsub(/\.\/kirano-site\.wasm/, "./kirano-site.wasm?v=" bid)
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
echo "Patched $INDEX (title/OG + noscript + cache-bust v=$BUILD_ID)"

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

echo "Done → https://kirano.shop/ (Makepad wasm + /privacy /terms)"
