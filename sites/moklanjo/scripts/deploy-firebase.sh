#!/usr/bin/env bash
# Build the Makepad moklanjo-site (wasm) + static shell (privacy, terms, OG)
# and deploy ONE tree to Firebase Hosting → moklanjo.com.
#
# One-time: npm i -g firebase-tools && firebase login
# The Firebase project is the default in sites/moklanjo/.firebaserc, and only
# that: the script reads it and always passes it to firebase.
# Domain: Firebase Console → Hosting → add moklanjo.com
#
#   ./sites/moklanjo/scripts/deploy-firebase.sh
#   ./sites/moklanjo/scripts/deploy-firebase.sh --dry-run
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SITE="$ROOT/sites/moklanjo"
OUT="$SITE/public"
WASM_PKG="$ROOT/target/makepad-wasm-app/release/moklanjo-site"
DRY=0

# This site's own Firebase project: the default in .firebaserc, read here and
# always passed to firebase. Without --project the CLI uses whatever
# `firebase use` last set for this folder — on 2026-11-21 that was a sibling
# site's project on four of the five Kumkaj sites.
PROJECT="$(sed -n 's/.*"default"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SITE/.firebaserc" 2>/dev/null | head -1)"
if [[ -z "$PROJECT" ]]; then
  echo "error: no default project in ${SITE#"$ROOT"/}/.firebaserc" >&2
  exit 1
fi
if [[ -n "${FIREBASE_PROJECT:-}" && "$FIREBASE_PROJECT" != "$PROJECT" ]]; then
  echo "error: FIREBASE_PROJECT=$FIREBASE_PROJECT is not this site's project ($PROJECT)" >&2
  exit 1
fi

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

echo "1/4 Bundle Firebase Auth → static/js/firebase-init.js + account.js"
(
  cd "$SITE"
  if [[ ! -d node_modules/firebase ]]; then
    npm install
  fi
  npm run build
)

echo "2/4 Build Makepad wasm (moklanjo-site, release, --no-threads)"
(
  cd "$ROOT"
  cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
    wasm --no-threads --strip --brotli build -p moklanjo-site --release
)

if [[ ! -f "$WASM_PKG/index.html" || ! -f "$WASM_PKG/moklanjo-site.wasm" ]]; then
  echo "error: wasm package incomplete at $WASM_PKG" >&2
  exit 1
fi

echo "3/4 Stage Firebase public/ = wasm package + static shell"
rm -rf "$OUT"
mkdir -p "$OUT"
# cargo-makepad writes a .br twin beside every file; Firebase Hosting
# compresses on its own and never serves those.
rsync -a --delete --exclude='*.br' "$WASM_PKG/" "$OUT/"
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
# Title, description and link-preview tags live in shell/head.html (one copy
# for deploy and preview). Crawlers and chat apps read this HTML only — they
# never run the wasm.
PREVIEW="$SITE/static/media/link-preview.jpg"
if [[ ! -f "$PREVIEW" ]]; then
  echo "error: missing sites/moklanjo/static/media/link-preview.jpg — run sites/moklanjo/scripts/make-link-preview.sh" >&2
  exit 1
fi
# The picture's address changes when the picture does: WhatsApp, Telegram and
# Facebook keep a preview for days and only fetch again for a new address.
PREVIEW_URL="https://moklanjo.com/media/link-preview.jpg?v=$(sha256sum "$PREVIEW" | cut -c1-10)"
HEAD_INJECT="$(sed -e "s|__BUILD_ID__|$BUILD_ID|g" -e "s|__PREVIEW_URL__|$PREVIEW_URL|g" "$SITE/shell/head.html")"
BODY_INJECT='        <noscript><p>Moklanjo — courier operating system. <a href="/privacy">Privacy</a> · <a href="/terms/">Terms</a> · <a href="/download">Download</a></p></noscript>
    </body>'
BODY_INJECT='        <script src="./js/firebase-init.js?v='"$BUILD_ID"'" defer></script>
'"$BODY_INJECT"
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

echo "4/4 firebase deploy --only hosting → $PROJECT"
FB_ARGS=(deploy --only hosting --project "$PROJECT")
(
  cd "$SITE"
  if [[ "$DRY" -eq 1 ]]; then
    firebase "${FB_ARGS[@]}" --dry-run
  else
    firebase "${FB_ARGS[@]}"
  fi
)

echo "Done → https://moklanjo.com/ (Makepad wasm + /privacy /terms)"
