#!/usr/bin/env bash
# Regenerate Kumkaj app icons (desktop + Android launcher mipmaps) from the brand mark.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/apps/kumkaj/resources/kumkaj-mark.png"
RES="$ROOT/apps/kumkaj/resources"
ANDROID_RES="$RES/android/res"

if [[ ! -f "$SRC" ]]; then
  echo "Missing source mark: $SRC" >&2
  exit 1
fi
if ! command -v magick >/dev/null 2>&1; then
  echo "ImageMagick (magick) is required" >&2
  exit 1
fi

resize() {
  local size="$1" dest="$2"
  magick "$SRC" -resize "${size}x${size}" -background none -gravity center -extent "${size}x${size}" "$dest"
  echo "wrote $dest (${size}x${size})"
}

resize 32 "$RES/icon_32.png"
resize 64 "$RES/icon_64.png"
resize 128 "$RES/icon_128.png"
resize 256 "$RES/icon_256.png"
resize 512 "$RES/icon_512.png"
resize 1024 "$RES/icon_1024.png"

declare -A MIPMAPS=(
  [mipmap-mdpi]=48
  [mipmap-hdpi]=72
  [mipmap-xhdpi]=96
  [mipmap-xxhdpi]=144
  [mipmap-xxxhdpi]=192
)
for dir in "${!MIPMAPS[@]}"; do
  mkdir -p "$ANDROID_RES/$dir"
  resize "${MIPMAPS[$dir]}" "$ANDROID_RES/$dir/ic_launcher.png"
done

# Windows .ico (multi-size)
magick "$SRC" -define icon:auto-resize=256,128,64,48,32,16 "$RES/icon.ico"
echo "wrote $RES/icon.ico"

echo "Android launcher icons ready under $ANDROID_RES/mipmap-*/ic_launcher.png"
echo "Reinstall the app (cargo kumkaj-android-dev) so the launcher picks them up."
