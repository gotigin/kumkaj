#!/usr/bin/env bash
# Development Windows x86_64 package (debug app id).
# Makepad layout: kumkaj-courier.exe + kumkaj-courier/resources/
# Includes install.ps1 / uninstall.ps1 for Dev (com.kumkaj.courier.debug).
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

OUT="$ROOT/dist/windows-dev"
rm -rf "$OUT"
mkdir -p "$OUT"
TARGET="$(windows_default_target)"

if [[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* ]]; then
  if rustup target list --installed 2>/dev/null | grep -qx 'x86_64-pc-windows-msvc'; then
    TARGET=x86_64-pc-windows-msvc
  fi
else
  windows_ensure_toolchain
fi

cargo build -p kumkaj-courier --target "$TARGET"

# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
EXE=""
for cand in \
  "$TARGET_DIR/$TARGET/debug/kumkaj-courier.exe" \
  "$TARGET_DIR/$TARGET/debug/kumkaj-courier"
do
  if [[ -f "$cand" ]]; then
    EXE="$cand"
    break
  fi
done
if [[ -z "$EXE" ]]; then
  echo "Built binary not found under $TARGET_DIR/$TARGET/debug/" >&2
  exit 1
fi

cp -f "$EXE" "$OUT/kumkaj-courier.exe"
makepad_copy_desktop_resources "$OUT"
stage_desktop_package "$OUT" windows

cat >"$OUT/README.txt" <<EOF
Kumkaj Courier Windows (development / debug)
============================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Profile: debug (dev) — not --release
Target: $TARGET

Install:
  powershell -ExecutionPolicy Bypass -File .\\install.ps1
→ %LOCALAPPDATA%\\Programs\\Kumkaj Courier Dev\\
→ data: %APPDATA%\\${KUMKAJ_APP_ID}\\

Uninstall:
  Start Menu → Uninstall Kumkaj Courier Dev
  or: powershell -ExecutionPolicy Bypass -File .\\uninstall.ps1 -Dev

Prod: ./scripts/package-windows.sh → dist/windows/
EOF

echo "Windows Dev package: $OUT/"
echo "Copy the whole folder to Windows (exe + resources + install.ps1)."
