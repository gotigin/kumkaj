#!/usr/bin/env bash
# Production Windows x86_64 package (Windows 10 + Windows 11).
# Makepad layout: kumkaj-courier.exe + kumkaj_courier/resources/ (underscores).
# Includes Setup.bat / install.ps1 / uninstall (Add/Remove Programs + keep/remove data).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/courier"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
export KUMKAJ_BIN_NAME=kumkaj-courier
export KUMKAJ_INSTALL_SLUG=kumkaj-courier
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=../../../scripts/windows-env.sh
source "$ROOT/scripts/windows-env.sh"
# shellcheck source=desktop-ids.sh
source "$KUMKAJ_APP_DIR/scripts/desktop-ids.sh"
# shellcheck source=../../../scripts/stage-desktop-package.sh
source "$ROOT/scripts/stage-desktop-package.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production
makepad_enable_package_dir

OUT="$ROOT/dist/windows"
rm -rf "$OUT"
mkdir -p "$OUT"
TARGET="$(windows_default_target)"

if [[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* || "$(uname -s)" == Windows_NT ]]; then
  if rustup target list --installed 2>/dev/null | grep -qx 'x86_64-pc-windows-msvc'; then
    TARGET=x86_64-pc-windows-msvc
  fi
  echo "Native Windows build ($TARGET)"
  cargo build -p kumkaj-courier --release --target "$TARGET"
else
  windows_ensure_toolchain
  cargo build -p kumkaj-courier --release --target "$TARGET"
fi

# shellcheck source=../../../scripts/cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
EXE=""
for cand in \
  "$TARGET_DIR/$TARGET/release/kumkaj-courier.exe" \
  "$TARGET_DIR/$TARGET/release/kumkaj-courier" \
  "$TARGET_DIR/release/kumkaj-courier.exe"
do
  if [[ -f "$cand" ]]; then
    EXE="$cand"
    break
  fi
done
if [[ -z "$EXE" ]]; then
  echo "Built binary not found under $TARGET_DIR/$TARGET/release/" >&2
  exit 1
fi

cp -f "$EXE" "$OUT/kumkaj-courier.exe"
makepad_copy_desktop_resources "$OUT"
stage_desktop_package "$OUT" windows

cat >"$OUT/README.txt" <<EOF
Kumkaj Courier Windows (production)
===================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Target: $TARGET (x86_64 — Windows 10 and Windows 11)

Install (recommended):
  Double-click Setup.bat
  (Do not open install.ps1 in Notepad — use Setup.bat.)

  Installs to: %LOCALAPPDATA%\\Programs\\Kumkaj Courier\\
  Start Menu + Settings → Apps uninstall entry

This folder is the installer package (exe + fonts + Setup.bat), not a lone
Setup.exe. After Setup, use the Start Menu shortcut — that is the installed app.

Portable run (no install):
  .\\kumkaj-courier.exe
  (keep this whole folder — exe + kumkaj_courier\\resources\\)

Uninstall:
  Start Menu → Uninstall Kumkaj Courier
  or Settings → Apps → Kumkaj Courier
  or double-click Uninstall.bat

Uninstall asks whether to keep or remove books data:
  %APPDATA%\\${KUMKAJ_APP_ID}\\
Re-run uninstall later to delete leftover data if you chose Keep.

First start asks for the Owner PIN (Server Mode) — there is no default PIN to change.
EOF

echo "Windows package: $OUT/"
