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
TARGET_DIR="$(cargo metadata --format-version=1 --no-deps | python3 -c 'import sys,json; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/kumkaj"
if [[ ! -f "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi
OUT="$ROOT/dist/linux"
mkdir -p "$OUT"
cp -f "$BIN" "$OUT/kumkaj"
mkdir -p "$OUT/menu"
cp -f "$ROOT/apps/kumkaj/resources/menu/"*.webp "$OUT/menu/"
cat > "$OUT/README.txt" <<'EOF'
Kumkaj Linux build
==================
Run: ./kumkaj
Live-dev (from repo): cargo kumkaj-dev

Data directory: ~/.local/share/com.kumkaj.app/
(Dev builds use com.kumkaj.app.debug — see env/development.env)
Default Owner PIN: 0000 — change under More → Team after first unlock.
EOF
echo "Linux package: $OUT/kumkaj"

