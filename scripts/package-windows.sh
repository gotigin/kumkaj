#!/usr/bin/env bash
# Cross/native Windows packaging notes + build when target is available.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production
OUT="$ROOT/dist/windows"
mkdir -p "$OUT"

if rustup target list --installed | grep -q 'x86_64-pc-windows-gnu\|x86_64-pc-windows-msvc'; then
  TARGET=$(rustup target list --installed | grep 'x86_64-pc-windows' | head -1)
  echo "Building for $TARGET"
  cargo build -p kumkaj --release --target "$TARGET"
  TARGET_DIR="$(cargo metadata --format-version=1 --no-deps | python3 -c 'import sys,json; print(json.load(sys.stdin)["target_directory"])')"
  if [[ -f "$TARGET_DIR/$TARGET/release/kumkaj.exe" ]]; then
    cp -f "$TARGET_DIR/$TARGET/release/kumkaj.exe" "$OUT/kumkaj.exe"
  elif [[ -f "$TARGET_DIR/$TARGET/release/kumkaj" ]]; then
    cp -f "$TARGET_DIR/$TARGET/release/kumkaj" "$OUT/kumkaj.exe"
  else
    echo "Built binary not found under $TARGET_DIR/$TARGET/release/" >&2
    exit 1
  fi
else
  echo "Windows target not installed on this machine."
  echo "Install (example): rustup target add x86_64-pc-windows-gnu"
  echo "Then re-run this script on a Windows host or with a working cross toolchain."
fi

cat > "$OUT/README.txt" <<'EOF'
Kumkaj Windows packaging
========================
Preferred: build on a Windows machine with Rust + MSVC:
  cargo build -p kumkaj --release
  copy target\release\kumkaj.exe

Cross-compile from Linux when a Windows target + linker is configured.
Data directory on Windows: %APPDATA%\com.kumkaj.app\
EOF
echo "Windows package folder: $OUT"
