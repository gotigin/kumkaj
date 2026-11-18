#!/usr/bin/env bash
# Build the Makepad moklanjo-site (wasm) + static shell (privacy, terms, OG)
# and deploy ONE tree to Firebase Hosting → moklanjo.com.
#
# One-time: npm i -g firebase-tools && firebase login
# Create a Firebase project, then either:
#   echo '{ "projects": { "default": "YOUR_PROJECT_ID" } }' > sites/moklanjo/.firebaserc
#   or: FIREBASE_PROJECT=YOUR_PROJECT_ID ./sites/moklanjo/scripts/deploy-firebase.sh
# Domain: Firebase Console → Hosting → add moklanjo.com
#
#   ./sites/moklanjo/scripts/deploy-firebase.sh
#   ./sites/moklanjo/scripts/deploy-firebase.sh --dry-run
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/moklanjo"
OUT="$SITE/public"
WASM_PKG="$ROOT/target/makepad-wasm-app/release/moklanjo-site"
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
    echo "error: no Firebase project configured for moklanjo.com." >&2
    echo "  Create a project, then set sites/moklanjo/.firebaserc default" >&2
    echo "  or pass FIREBASE_PROJECT=… (owner step — domains are registered)." >&2
    exit 1
  fi
fi

echo "1/3 Build Makepad wasm (moklanjo-site, release, --no-threads)"
(
  cd "$ROOT"
  cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
    wasm --no-threads --strip --brotli build -p moklanjo-site --release
)

if [[ ! -f "$WASM_PKG/index.html" || ! -f "$WASM_PKG/moklanjo-site.wasm" ]]; then
  echo "error: wasm package incomplete at $WASM_PKG" >&2
  exit 1
fi

echo "2/3 Stage Firebase public/ = wasm package + static shell"
rm -rf "$OUT"
mkdir -p "$OUT"
rsync -a --delete "$WASM_PKG/" "$OUT/"
rsync -a "$SITE/static/" "$OUT/"
"$SITE/scripts/patch-web-shell.sh" "$OUT/index.html"

PRIVACY_MD="$ROOT/apps/moklanjo/docs/PRIVACY.md"
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
  --metadata title="Privacy — Moklanjo" \
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

BUILD_ID="$(date -u +%Y%m%d%H%M%S)-$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo local)"
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
# Optional Analytics bundle if the owner added client/firebase-init.js later.
ANALYTICS_LINE=""
if [[ -f "$OUT/js/firebase-init.js" ]]; then
  ANALYTICS_LINE='        <script src="./js/firebase-init.js?v='"$BUILD_ID"'" defer></script>'
  BODY_INJECT="$ANALYTICS_LINE
$BODY_INJECT"
fi
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
echo "Patched $INDEX (title/OG + noscript + cache-bust v=$BUILD_ID)"

echo "3/3 firebase deploy --only hosting"
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

echo "Done → https://moklanjo.com/ (Makepad wasm + /privacy /terms)"
