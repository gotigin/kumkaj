#!/usr/bin/env bash
# Build release Linux binary for Kumkaj.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production
cargo build -p kumkaj --release
# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
BIN="$TARGET_DIR/release/kumkaj"
if [[ ! -f "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi
OUT="$ROOT/dist/linux"
mkdir -p "$OUT"
cp -f "$BIN" "$OUT/kumkaj"
rm -rf "$OUT/makepad_widgets"
cat > "$OUT/README.txt" <<'EOF'
Kumkaj Linux build
==================
Run from the repo machine (or keep source checkout for fonts): ./kumkaj
Live-dev (from repo): cargo kumkaj-dev

Data directory: ~/.local/share/com.kumkaj.app/
(Dev builds use com.kumkaj.app.debug — see env/development.env)
Default Owner PIN: 0000 — change under More → Team after first unlock.
EOF
echo "Linux package: $OUT/kumkaj"
