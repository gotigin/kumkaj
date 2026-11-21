#!/usr/bin/env bash
# Put moklanjo.com's own start-up screen into a staged Makepad wasm index.html.
#
# cargo-makepad writes a grey bar that says "Loading.." and nothing else. This
# replaces it with shell/boot.{css,html,js}: the logo in a ring that fills as
# the wasm arrives, the name, a line that changes, and an honest message when
# the page is slow or cannot start. Everything is inlined, so the screen is
# there with the first response and costs no request.
#
#   ./sites/moklanjo/scripts/patch-boot-screen.sh <path/to/index.html>
#
# The wasm must sit beside index.html (its size drives the progress ring).
# Safe to run twice. Called by patch-boot-screen.sh (preview-web.sh and deploy-firebase.sh).
set -euo pipefail

INDEX="${1:?usage: patch-boot-screen.sh <path/to/index.html>}"
SITE="$(cd "$(dirname "$0")/.." && pwd)"
SHELL_SRC="$SITE/shell"
WASM_NAME="moklanjo-site.wasm"

if [[ ! -f "$INDEX" ]]; then
  echo "error: missing $INDEX" >&2
  exit 1
fi
PKG="$(cd "$(dirname "$INDEX")" && pwd)"
for f in boot.css boot.html boot.js logo.webp; do
  if [[ ! -f "$SHELL_SRC/$f" ]]; then
    echo "error: missing $SHELL_SRC/$f" >&2
    exit 1
  fi
done
if [[ ! -f "$PKG/$WASM_NAME" ]]; then
  echo "error: missing $PKG/$WASM_NAME (needed for the progress ring)" >&2
  exit 1
fi
if grep -q 'id="boot"' "$INDEX"; then
  echo "Already patched: $INDEX"
  exit 0
fi
if ! grep -q "class='canvas_loader'" "$INDEX"; then
  echo "error: $INDEX has no canvas_loader block — cargo-makepad changed its shell" >&2
  exit 1
fi

WASM_BYTES="$(stat -c %s "$PKG/$WASM_NAME")"
BODY_TMP="$(mktemp)"
trap 'rm -f "$BODY_TMP" "$INDEX.tmp"' EXIT
# The logo travels inside the page (about 3 KB) so it shows at once.
{
  printf 's|__WASM_BYTES__|%s|\n' "$WASM_BYTES"
  printf 's|__LOGO_DATA_URI__|data:image/webp;base64,%s|\n' "$(base64 -w0 "$SHELL_SRC/logo.webp")"
} > "$BODY_TMP.sed"
sed -f "$BODY_TMP.sed" "$SHELL_SRC/boot.html" > "$BODY_TMP"
rm -f "$BODY_TMP.sed"

awk -v css="$SHELL_SRC/boot.css" -v body="$BODY_TMP" -v js="$SHELL_SRC/boot.js" '
  function cat(file,   line) {
    while ((getline line < file) > 0) print line
    close(file)
  }
  # cargo-makepad: <div class="canvas_loader"> <div> Loading.. </div> </div>
  skip > 0 {
    if ($0 ~ /<\/div>/) skip--
    next
  }
  /<\/head>/ && !styled {
    print "        <style>"
    cat(css)
    print "        </style>"
    styled = 1
  }
  /class=.canvas_loader./ && !booted {
    cat(body)
    print "        <script>"
    cat(js)
    print "        </script>"
    booted = 1
    skip = 2
    next
  }
  { print }
  END { if (!styled || !booted) exit 3 }
' "$INDEX" > "$INDEX.tmp"
mv "$INDEX.tmp" "$INDEX"

if ! grep -q 'id="boot"' "$INDEX" || grep -q 'Loading\.\.$' "$INDEX"; then
  echo "error: start-up screen was not placed in $INDEX" >&2
  exit 1
fi
echo "Patched $INDEX (start-up screen, wasm $WASM_BYTES bytes)"
