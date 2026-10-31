#!/usr/bin/env bash
# Regenerate an app's icons / logo from its product master
# (apps/<app>/assets/). Run through the app's own scripts/generate-icons.sh,
# which sets KUMKAJ_APP_DIR and KUMKAJ_ICON_SRC (no default app).
# - PNG + .ico for OS launchers / window chrome (Android + desktop)
# - WebP for in-app embedding (Makepad Image / crate_resource)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_DIR="${KUMKAJ_APP_DIR:?run apps/<app>/scripts/generate-icons.sh}"
SRC="${KUMKAJ_ICON_SRC:?run apps/<app>/scripts/generate-icons.sh}"
RES="$APP_DIR/resources"
ANDROID_RES="$RES/android/res"

if [[ ! -f "$SRC" ]]; then
  echo "Missing source mark: $SRC" >&2
  exit 1
fi
if ! command -v magick >/dev/null 2>&1; then
  echo "ImageMagick (magick) is required" >&2
  exit 1
fi

mkdir -p "$RES"

# Square PNG sizes for OS / Makepad window icon embedding (PNG required).
resize_png() {
  local size="$1" dest="$2"
  magick "$SRC" -resize "${size}x${size}" -background none -gravity center \
    -extent "${size}x${size}" "$dest"
  echo "wrote $dest (${size}x${size} png)"
}

# PNG sizes for OS / Makepad window chrome (desktop). Android launcher uses
# mipmaps below — these PNGs are not packaged into the APK assets tree.
resize_png 32 "$RES/icon_32.png"
resize_png 64 "$RES/icon_64.png"
resize_png 128 "$RES/icon_128.png"
resize_png 256 "$RES/icon_256.png"
resize_png 512 "$RES/icon_512.png"
resize_png 1024 "$RES/icon_1024.png"

# In-app embedding: lossless WebP + alpha only. Lossy VP8 drops alpha and can
# composite onto black; Makepad then shows a solid black square. No PNG/AVIF
# brand marks — those used to ship into Android assets unused (~2MB+).
webp_from() {
  local size="$1" dest="$2"
  magick "$SRC" -resize "${size}x${size}" -background none -gravity center \
    -extent "${size}x${size}" -define webp:lossless=true "$dest"
  echo "wrote $dest (${size}x${size} webp lossless)"
}

webp_from 256 "$RES/kumkaj-mark.webp"

# Drop legacy companions so they cannot be re-bundled into APK assets.
rm -f \
  "$RES/kumkaj-mark.png" \
  "$RES/kumkaj-mark-128.png" \
  "$RES/kumkaj-mark-128.webp" \
  "$RES/kumkaj-mark-512.webp" \
  "$RES/kumkaj-mark.avif" \
  "$RES/kumkaj-mark-128.avif" \
  "$RES/kumkaj-mark-512.avif" \
  "$RES/gotigin-mark.webp"


declare -A MIPMAPS=(
  [mipmap-mdpi]=48
  [mipmap-hdpi]=72
  [mipmap-xhdpi]=96
  [mipmap-xxhdpi]=144
  [mipmap-xxxhdpi]=192
)
for dir in "${!MIPMAPS[@]}"; do
  mkdir -p "$ANDROID_RES/$dir"
  resize_png "${MIPMAPS[$dir]}" "$ANDROID_RES/$dir/ic_launcher.png"
done

# Windows .ico (multi-size)
magick "$SRC" -define icon:auto-resize=256,128,64,48,32,16 "$RES/icon.ico"
echo "wrote $RES/icon.ico"

# Makepad window icons: set via .cargo/config.toml [env] MAKEPAD_APP_ICON_*
# pointing at the app resources/icon_*.png (no second resources/ tree).

echo "Android launcher icons ready under $ANDROID_RES/mipmap-*/ic_launcher.png"
echo "In-app logo: $RES/kumkaj-mark.webp (use crate_resource / load_image_dep)"
echo "Reinstall on Android (the app's android-dev.sh) so the launcher picks up icons."
