#!/usr/bin/env bash
# Desktop live-dev for THIS app (Moklanjo): loads
# apps/moklanjo/env/development.env then runs with --hot --remote.
# Usage: ./apps/moklanjo/scripts/dev.sh   or: cargo moklanjo-dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
export KUMKAJ_ROOT="$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/moklanjo"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
load_kumkaj_env development
exec cargo run -p kumkaj-moklanjo -- "$@" --hot --remote
