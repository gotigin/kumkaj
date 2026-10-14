#!/usr/bin/env bash
# Production Windows x86_64 package (Windows 10 + Windows 11).
# Makepad layout: kumkaj.exe + <crate>/resources/ beside it (MAKEPAD_PACKAGE_DIR=.).
# Usage: ./scripts/package-windows.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=windows-env.sh
source "$ROOT/scripts/windows-env.sh"

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
  cargo build -p kumkaj --release --target "$TARGET"
else
  windows_ensure_toolchain
  cargo build -p kumkaj --release --target "$TARGET"
fi

# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
EXE=""
for cand in \
  "$TARGET_DIR/$TARGET/release/kumkaj.exe" \
  "$TARGET_DIR/$TARGET/release/kumkaj" \
  "$TARGET_DIR/release/kumkaj.exe"
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

cp -f "$EXE" "$OUT/kumkaj.exe"
makepad_copy_desktop_resources "$OUT"

cat > "$OUT/README.txt" <<EOF
Kumkaj Windows (production)
===========================
Makepad desktop package — copy this WHOLE folder to the PC (keep structure):

  kumkaj.exe
  kumkaj/resources/            (fonts + icons + menu)

Target: $TARGET (x86_64 — Windows 10 and Windows 11)
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Env: env/production.env

Run from inside the folder:
  .\\kumkaj.exe

No console window (GUI subsystem). Data: %APPDATA%\\com.kumkaj.app\\

Default Owner PIN: 0000 — change under More → Team after first unlock.
EOF

echo "Windows package: $OUT/"
