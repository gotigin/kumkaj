#!/usr/bin/env bash
# Regenerate Khaynjo's launcher icons, window icons and in-app mark from its
# master art (apps/khaynjo/assets/khaynjo-app-icon.png).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/khaynjo"
export KUMKAJ_ICON_SRC="$KUMKAJ_APP_DIR/assets/khaynjo-app-icon.png"
exec "$ROOT/scripts/generate-app-icons.sh"
