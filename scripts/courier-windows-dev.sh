#!/usr/bin/env bash
# Development Windows x86_64 package (Windows 10 + Windows 11).
# Makepad layout: kumkaj.exe + <crate>/resources/ beside it (MAKEPAD_PACKAGE_DIR=.).
# Usage: ./scripts/kumkaj-windows-dev.sh   or: cargo kumkaj-windows-dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=windows-env.sh
source "$ROOT/scripts/windows-env.sh"

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

# Dev profile (unoptimized + debuginfo) — matches Linux `cargo kumkaj-dev`.
# Production shipping stays `./scripts/package-windows.sh` (--release).
cargo build -p kumkaj --target "$TARGET"

# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
EXE=""
for cand in \
  "$TARGET_DIR/$TARGET/debug/kumkaj.exe" \
  "$TARGET_DIR/$TARGET/debug/kumkaj"
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

cp -f "$EXE" "$OUT/kumkaj.exe"
makepad_copy_desktop_resources "$OUT"

cat > "$OUT/README.txt" <<EOF
Kumkaj Windows (development)
============================
Makepad desktop package — copy this WHOLE folder to the PC (keep structure):

  kumkaj.exe
  kumkaj/resources/            (fonts + icons + menu)

Profile: debug (dev) — not --release
Target: $TARGET (x86_64 — Windows 10 and Windows 11)
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Env: env/development.env

Run from inside the folder:
  .\\kumkaj.exe

Data: %APPDATA%\\com.kumkaj.app.debug\\

Production (release): ./scripts/package-windows.sh → dist/windows/
EOF

echo "Windows Dev package: $OUT/"
echo "Copy the whole folder to Windows (exe + resources)."
