#!/usr/bin/env bash
# Shared MSIX packager for Kumkaj Windows apps (Store / sideload).
# Called by apps/<app>/scripts/msix.sh after it exports:
#   KUMKAJ_APP_DIR, KUMKAJ_BIN_NAME, KUMKAJ_MSIX_NAME (display label),
#   KUMKAJ_MSIX_IDENTITY (Identity Name — never change once published),
#   KUMKAJ_MSI_PKG_PROD / KUMKAJ_MSI_PKG_DEV (reuse Windows package dirs),
#   KUMKAJ_MSI_PACKAGE_PROD (script that builds the Windows folder package).
#
#   ./apps/khaynjo/scripts/msix.sh        # prod → dist/Khaynjo-<ver>.msix
#   ./apps/khaynjo/scripts/msix.sh dev    # → dist/KhaynjoDev-<ver>.msix
#
# Publisher CN must match the signing cert / Partner Center publisher.
# Override: KUMKAJ_MSIX_PUBLISHER='CN=…' KUMKAJ_MSIX_PUBLISHER_DISPLAY='…'
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:-prod}"

: "${KUMKAJ_APP_DIR:?set by apps/<app>/scripts/msix.sh}"
: "${KUMKAJ_BIN_NAME:?}"
: "${KUMKAJ_MSIX_NAME:?}"
: "${KUMKAJ_MSIX_IDENTITY:?}"
: "${KUMKAJ_MSI_PKG_PROD:?}"
: "${KUMKAJ_MSI_PKG_DEV:?}"
: "${KUMKAJ_MSI_PACKAGE_PROD:?}"

PUBLISHER="${KUMKAJ_MSIX_PUBLISHER:-CN=Kumkaj}"
PUBLISHER_DISPLAY="${KUMKAJ_MSIX_PUBLISHER_DISPLAY:-Kumkaj}"

case "$MODE" in
  prod|production)
    PKG="$KUMKAJ_MSI_PKG_PROD"
    PACKAGE_SCRIPT="$KUMKAJ_MSI_PACKAGE_PROD"
    IDENTITY="$KUMKAJ_MSIX_IDENTITY"
    LABEL="$KUMKAJ_MSIX_NAME"
    ;;
  dev|development)
    PKG="$KUMKAJ_MSI_PKG_DEV"
    if [[ -n "${KUMKAJ_MSI_PACKAGE_DEV:-}" ]]; then
      PACKAGE_SCRIPT="$KUMKAJ_MSI_PACKAGE_DEV"
    else
      # Same as MSI: most apps use windows-dev.sh next to windows-prod.sh
      PACKAGE_SCRIPT="$(dirname "$KUMKAJ_MSI_PACKAGE_PROD")/windows-dev.sh"
      if [[ ! -x "$PACKAGE_SCRIPT" && ! -f "$PACKAGE_SCRIPT" ]]; then
        PACKAGE_SCRIPT="$(dirname "$KUMKAJ_MSI_PACKAGE_PROD")/package-windows-dev.sh"
      fi
    fi
    IDENTITY="${KUMKAJ_MSIX_IDENTITY_DEV:-${KUMKAJ_MSIX_IDENTITY}.Dev}"
    LABEL="${KUMKAJ_MSIX_NAME}Dev"
    ;;
  *)
    echo "usage: msix.sh [prod|dev]" >&2
    exit 2
    ;;
esac

echo "==> Building Windows package for MSIX ($MODE)"
if [[ "$PACKAGE_SCRIPT" == *.sh ]]; then
  bash "$PACKAGE_SCRIPT"
else
  "$PACKAGE_SCRIPT"
fi

EXE="${KUMKAJ_BIN_NAME}.exe"
if [[ ! -f "$PKG/$EXE" ]]; then
  echo "missing $PKG/$EXE — package step failed?" >&2
  exit 1
fi

VER="$(python3 - <<PY
import re, pathlib
toml = pathlib.Path(r"$KUMKAJ_APP_DIR") / "Cargo.toml"
text = toml.read_text(encoding="utf-8")
m = re.search(r'^version\s*=\s*"([^"]+)"', text, re.M)
print(m.group(1) if m else "0.1.0")
PY
)"

ICON=""
for cand in \
  "$KUMKAJ_APP_DIR/resources/icon_512.png" \
  "$KUMKAJ_APP_DIR/resources/icon_1024.png" \
  "$KUMKAJ_APP_DIR/resources/icon_256.png"
do
  if [[ -f "$cand" ]]; then ICON="$cand"; break; fi
done
if [[ -z "$ICON" ]]; then
  echo "no icon_*.png under $KUMKAJ_APP_DIR/resources" >&2
  exit 1
fi

OUT="$ROOT/dist/${LABEL}-${VER}.msix"
LAYOUT="$ROOT/dist/${LABEL}-${VER}-msix-layout"

echo "==> Packing MSIX"
python3 "$ROOT/scripts/msix/pack_msix.py" \
  --payload "$PKG" \
  --out "$OUT" \
  --layout "$LAYOUT" \
  --name "$IDENTITY" \
  --publisher "$PUBLISHER" \
  --publisher-display "$PUBLISHER_DISPLAY" \
  --display "$LABEL" \
  --version "$VER" \
  --exe "$EXE" \
  --description "$LABEL business books" \
  --icon-png "$ICON"

# Optional: sign when a PFX is provided (typically on Windows with signtool in PATH).
if [[ -n "${KUMKAJ_MSIX_PFX:-}" ]]; then
  if command -v signtool >/dev/null 2>&1; then
    echo "==> Signing with signtool"
    SIGN_ARGS=(sign /fd SHA256 /a /f "$KUMKAJ_MSIX_PFX")
    if [[ -n "${KUMKAJ_MSIX_PFX_PASSWORD:-}" ]]; then
      SIGN_ARGS+=(/p "$KUMKAJ_MSIX_PFX_PASSWORD")
    fi
    if [[ -n "${KUMKAJ_MSIX_TIMESTAMP_URL:-}" ]]; then
      SIGN_ARGS+=(/tr "$KUMKAJ_MSIX_TIMESTAMP_URL" /td SHA256)
    fi
    signtool "${SIGN_ARGS[@]}" "$OUT"
  else
    echo "KUMKAJ_MSIX_PFX set but signtool not in PATH — left unsigned: $OUT" >&2
  fi
fi

echo
echo "Done: $OUT"
echo "Layout (MakeAppx fallback): $LAYOUT"
echo "Publisher: $PUBLISHER  Identity: $IDENTITY"
echo "Store: set KUMKAJ_MSIX_PUBLISHER to your Partner Center Publisher CN, sign with the Store .pfx, upload the .msix."
