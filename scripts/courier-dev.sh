#!/usr/bin/env bash
# Desktop live-dev: loads apps/courier/env/development.env then runs with --hot --remote.
# Usage: ./scripts/courier-dev.sh   or: cargo courier-dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env development
exec cargo run -p kumkaj-courier -- "$@" --hot --remote
