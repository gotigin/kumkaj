#!/usr/bin/env bash
# Build development Linux package (debug app id) + install/uninstall helpers.
# Output: dist/linux-dev/
# Uses debug Cargo profile + apps/courier/env/development.env → com.kumkaj.courier.debug
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=windows-env.sh
source "$ROOT/scripts/windows-env.sh"
# shellcheck source=desktop-ids.sh
source "$ROOT/scripts/desktop-ids.sh"
# shellcheck source=stage-desktop-package.sh
source "$ROOT/scripts/stage-desktop-package.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env development
makepad_enable_package_dir

cargo build -p kumkaj-courier
# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
BIN="$TARGET_DIR/debug/kumkaj-courier"
if [[ ! -f "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi

OUT="$ROOT/dist/linux-dev"
rm -rf "$OUT"
mkdir -p "$OUT/bin"

cp -f "$BIN" "$OUT/bin/kumkaj-courier"
chmod +x "$OUT/bin/kumkaj-courier"
makepad_copy_desktop_resources "$OUT"

stage_desktop_package "$OUT" linux

cat >"$OUT/README.txt" <<EOF
Kumkaj Courier Linux (development / debug)
==========================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}

Install:
  ./install.sh
→ ~/.local/lib/kumkaj-courier-debug/
→ command: kumkaj-courier-dev
→ data: ~/.local/share/${KUMKAJ_APP_ID}/

Uninstall:
  Uninstall Kumkaj Courier Dev (app menu)
  or: ./uninstall.sh --dev

Prod package: ./scripts/package-linux.sh → dist/linux/
Live from repo: cargo courier-dev
EOF

echo "Linux Dev package: $OUT/"
