#!/usr/bin/env bash
# Ensure Microsoft makemsix (Linux) is available for Store-valid BlockMaps.
# Builds vendor/msix-packaging once; installs binary + lib under tools/makemsix/.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DEST="$ROOT/tools/makemsix"
BIN="$DEST/makemsix"
LIB="$DEST/libmsix.so"

if [[ -x "$BIN" && -f "$LIB" ]]; then
  echo "$BIN"
  exit 0
fi

SDK="$ROOT/vendor/msix-packaging"
if [[ ! -d "$SDK/.git" ]]; then
  echo "==> Cloning microsoft/msix-packaging (pack support)"
  mkdir -p "$ROOT/vendor"
  git clone --depth 1 https://github.com/microsoft/msix-packaging.git "$SDK"
fi

# Arch/modern ICU needs C++17; upstream still defaults to 14.
sed -i 's/set(CMAKE_CXX_STANDARD 14)/set(CMAKE_CXX_STANDARD 17)/' \
  "$SDK/CMakeLists.txt" "$SDK/lib/xerces/CMakeLists.txt" 2>/dev/null || true

echo "==> Building makemsix (Release, pack on)"
rm -rf "$SDK/.vs"
mkdir -p "$SDK/.vs"
(
  cd "$SDK/.vs"
  cmake -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_C_COMPILER="${CC:-gcc}" \
    -DCMAKE_CXX_COMPILER="${CXX:-g++}" \
    -DCMAKE_CXX_STANDARD=17 \
    -DCMAKE_CXX_STANDARD_REQUIRED=ON \
    -DSKIP_BUNDLES=on \
    -DUSE_VALIDATION_PARSER=on \
    -DCMAKE_TOOLCHAIN_FILE=../cmake/linux.cmake \
    -DMSIX_PACK=on \
    -DMSIX_SAMPLES=off \
    -DMSIX_TESTS=off \
    -DLINUX=on ..
  cmake --build . -j"$(nproc)" --target makemsix
)

BUILT_BIN="$SDK/.vs/bin/makemsix"
BUILT_LIB="$SDK/.vs/lib/libmsix.so"
if [[ ! -x "$BUILT_BIN" || ! -f "$BUILT_LIB" ]]; then
  echo "makemsix build did not produce bin/lib under $SDK/.vs" >&2
  exit 1
fi

mkdir -p "$DEST"
cp -f "$BUILT_BIN" "$BIN"
cp -f "$BUILT_LIB" "$LIB"
chmod +x "$BIN"
echo "Installed $BIN"
echo "$BIN"
