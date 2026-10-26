#!/usr/bin/env bash
# Regenerate Moklanjo's launcher icons, window icons and in-app mark from its
# master art (apps/moklanjo/assets/moklanjo-app-icon.png).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/moklanjo"
export KUMKAJ_ICON_SRC="$KUMKAJ_APP_DIR/assets/moklanjo-app-icon.png"
exec "$ROOT/scripts/generate-app-icons.sh"
