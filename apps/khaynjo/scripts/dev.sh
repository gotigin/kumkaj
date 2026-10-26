#!/usr/bin/env bash
# Desktop live-dev for THIS app (Khaynjo): loads
# apps/khaynjo/env/development.env then runs with --hot --remote.
# Usage: ./apps/khaynjo/scripts/dev.sh   or: cargo khaynjo-dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
export KUMKAJ_ROOT="$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/khaynjo"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
load_kumkaj_env development
# Window/taskbar icons are this app's (workspace .cargo/config.toml defaults
# to Moklanjo's). Cargo [env] does not override exported vars.
export MAKEPAD_APP_ICON_32="$KUMKAJ_APP_DIR/resources/icon_32.png"
export MAKEPAD_APP_ICON_64="$KUMKAJ_APP_DIR/resources/icon_64.png"
export MAKEPAD_APP_ICON_128="$KUMKAJ_APP_DIR/resources/icon_128.png"
export MAKEPAD_APP_ICON_256="$KUMKAJ_APP_DIR/resources/icon_256.png"
export MAKEPAD_APP_ICON_512="$KUMKAJ_APP_DIR/resources/icon_512.png"
export MAKEPAD_APP_ICON_1024="$KUMKAJ_APP_DIR/resources/icon_1024.png"
export MAKEPAD_APP_ICON_ICO="$KUMKAJ_APP_DIR/resources/icon.ico"
exec cargo run -p kumkaj-khaynjo -- "$@" --hot --remote
