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
#   ./sites/moklanjo/scripts/deploy-firebase.sh --i-approve-firebase-deploy  # owner-approved upload
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
    --i-approve-firebase-deploy) ;;  # handled by require_firebase_deploy_approval
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


# Firebase Hosting uploads burn quota — require owner approval (docs/WEBSITES.md).
# shellcheck source=../../../scripts/require-firebase-deploy-approval.sh
source "$ROOT/scripts/require-firebase-deploy-approval.sh"
require_firebase_deploy_approval "$DRY" "$@"


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
# Language packs (site i18n): the wasm and site-i18n.js fetch /i18n/<code>.tsv.
mkdir -p "$OUT/i18n"
cp "$SITE"/translations/*.tsv "$OUT/i18n/"
"$SITE/scripts/patch-web-shell.sh" "$OUT/index.html"
# Fonts: only the characters this site draws (the repo keeps them whole).
"$SITE/scripts/subset-fonts.sh" "$OUT"

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
PREVIEW="$SITE/static/media/link-preview.jpg"
if [[ ! -f "$PREVIEW" ]]; then
  echo "error: missing sites/moklanjo/static/media/link-preview.jpg — run sites/moklanjo/scripts/make-link-preview.sh" >&2
  exit 1
fi
PREVIEW_V="$(sha256sum "$PREVIEW" | cut -c1-10)"
PREVIEW_URL="https://moklanjo.com/media/link-preview.jpg?v=$PREVIEW_V"
mkdir -p "$OUT/privacy"
pandoc "$PRIVACY_MD" \
  -f markdown \
  -t html5 \
  --template="$PRIVACY_TMPL" \
  --metadata title="Privacy — Moklanjo" \
  -V lang=en \
  -V preview_v="$PREVIEW_V" \
  -o "$OUT/privacy/index.html"
cp "$PRIVACY_MD" "$OUT/privacy.md"
echo "Presented /privacy from $PRIVACY_MD"

# Same CANCELLATION.md pattern as privacy — formatted presentation.
CANCELLATION_MD="$ROOT/apps/moklanjo/docs/CANCELLATION.md"
CANCELLATION_TMPL="$SITE/cancellation-template.html"
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
  --metadata title="Cancellation — Moklanjo" \
  -V lang=en \
  -V preview_v="$PREVIEW_V" \
  -o "$OUT/cancellation/index.html"
cp "$CANCELLATION_MD" "$OUT/cancellation.md"
echo "Presented /cancellation from apps/moklanjo/docs/CANCELLATION.md"


# Cache-bust og/twitter image on secondary shells (same sha10 as home).
for secondary in privacy terms cancellation account; do
  f="$OUT/$secondary/index.html"
  [[ -f "$f" ]] || continue
  sed -i -E "s#(https://moklanjo\.com/media/link-preview\.jpg)(\?v=[0-9a-f]+)?\"#\1?v=$PREVIEW_V\"#g" "$f"
done


if [[ ! -f "$OUT/privacy/index.html" ]]; then
  echo "error: privacy page missing after pandoc" >&2
  exit 1
fi
if [[ ! -f "$OUT/cancellation/index.html" ]]; then
  echo "error: cancellation page missing after pandoc" >&2
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
# PREVIEW_URL already set above (sha10 of link-preview.jpg).
HEAD_INJECT="$(sed -e "s|__BUILD_ID__|$BUILD_ID|g" -e "s|__PREVIEW_URL__|$PREVIEW_URL|g" "$SITE/shell/head.html")"
BODY_INJECT='        <link rel="stylesheet" href="./css/auth-nav.css?v='"$BUILD_ID"'">
        <script src="./js/lang-nav.js?v='"$BUILD_ID"'" defer></script>
        <noscript><p>Moklanjo — courier operating system. Cloud plans (ex-tax/yr): Free forever ₹0 · Steady ₹6,000 · Swift ₹14,000 · Priority ₹28,000 · Custom quote — <a href="/shop">/shop</a>. <a href="/privacy">Privacy</a> · <a href="/terms/">Terms</a> · <a href="/cancellation">Cancellation</a></p></noscript>
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
