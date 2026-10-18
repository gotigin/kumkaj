#!/usr/bin/env bash
# Desktop live-dev: loads env/development.env then runs with --hot --remote.
# Usage: ./scripts/kumkaj-dev.sh   or: cargo kumkaj-dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env development
exec cargo run -p kumkaj -- "$@" --hot --remote
