#!/usr/bin/env bash
# Desktop live-dev for THIS app (Kumkaj Courier): loads
# apps/courier/env/development.env then runs with --hot --remote.
# Usage: ./apps/courier/scripts/dev.sh   or: cargo courier-dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
export KUMKAJ_ROOT="$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/courier"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
load_kumkaj_env development
exec cargo run -p kumkaj-courier -- "$@" --hot --remote
