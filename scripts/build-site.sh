#!/usr/bin/env bash
# Build one Kumkaj static site with Zola.
# Names no product: pass the site directory.
#
#   KUMKAJ_SITE_DIR=web/kumkaj ./scripts/build-site.sh
#   KUMKAJ_SITE_DIR=apps/moklanjo/web ./scripts/build-site.sh
#   KUMKAJ_SITE_DIR=apps/khaynjo/web ./scripts/build-site.sh
#
# Output: dist/web/<site-name>/  (site-name = basename of KUMKAJ_SITE_DIR,
# except apps/*/web → moklanjo | khaynjo)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SITE_DIR="${KUMKAJ_SITE_DIR:-}"

if [[ -z "$SITE_DIR" ]]; then
  echo "usage: KUMKAJ_SITE_DIR=<path> $0" >&2
  echo "  e.g. KUMKAJ_SITE_DIR=web/kumkaj $0" >&2
  exit 1
fi

# Resolve relative to workspace root.
case "$SITE_DIR" in
  /*) ABS_SITE="$SITE_DIR" ;;
  *)  ABS_SITE="$ROOT/$SITE_DIR" ;;
esac

if [[ ! -f "$ABS_SITE/config.toml" ]]; then
  echo "error: no config.toml in $ABS_SITE" >&2
  exit 1
fi

BASE="$(basename "$ABS_SITE")"
PARENT="$(basename "$(dirname "$ABS_SITE")")"
if [[ "$BASE" == "web" && ( "$PARENT" == "moklanjo" || "$PARENT" == "khaynjo" ) ]]; then
  SITE_NAME="$PARENT"
elif [[ "$BASE" == "kumkaj" ]]; then
  SITE_NAME="kumkaj"
else
  SITE_NAME="$BASE"
fi

OUT="$ROOT/dist/web/$SITE_NAME"
export PATH="${HOME}/.local/bin:${PATH}"

if ! command -v zola >/dev/null 2>&1; then
  echo "error: zola not found (install to ~/.local/bin or PATH)" >&2
  exit 1
fi

mkdir -p "$OUT"
(
  cd "$ABS_SITE"
  zola build --output-dir "$OUT" --force
)

echo "Site package: $OUT"
