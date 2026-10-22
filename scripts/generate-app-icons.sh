#!/usr/bin/env bash
# Regenerate an app's icons / logo from its product master
# (apps/<app>/assets/). App: KUMKAJ_APP_DIR (default apps/courier).
# - PNG + .ico for OS launchers / window chrome (Android + desktop)
# - WebP for in-app embedding (Makepad Image / crate_resource)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_DIR="${KUMKAJ_APP_DIR:-$ROOT/apps/courier}"
SRC="${KUMKAJ_ICON_SRC:-$APP_DIR/assets/courier-app-icon.png}"
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

resize_png 32 "$RES/icon_32.png"
resize_png 64 "$RES/icon_64.png"
resize_png 128 "$RES/icon_128.png"
resize_png 256 "$RES/icon_256.png"
resize_png 512 "$RES/icon_512.png"
resize_png 1024 "$RES/icon_1024.png"

# Brand mark PNGs (legacy names still referenced by docs / older paths).
cp -f "$RES/icon_512.png" "$RES/kumkaj-mark.png"
cp -f "$RES/icon_128.png" "$RES/kumkaj-mark-128.png"

# In-app embedding: lossless WebP + alpha. Lossy VP8 drops alpha and can
# composite onto black; Makepad then shows a solid black square.
webp_from() {
  local size="$1" dest="$2"
  magick "$SRC" -resize "${size}x${size}" -background none -gravity center \
    -extent "${size}x${size}" -define webp:lossless=true "$dest"
  echo "wrote $dest (${size}x${size} webp lossless)"
}

webp_from 128 "$RES/kumkaj-mark-128.webp"
webp_from 256 "$RES/kumkaj-mark.webp"
webp_from 512 "$RES/kumkaj-mark-512.webp"

# Optional AVIF companions when ImageMagick builds with AVIF.
if magick -list format 2>/dev/null | grep -qi '^ *AVIF'; then
  magick "$SRC" -resize 128x128 -background none -gravity center -extent 128x128 \
    -quality 55 "$RES/kumkaj-mark-128.avif"
  magick "$SRC" -resize 256x256 -background none -gravity center -extent 256x256 \
    -quality 55 "$RES/kumkaj-mark.avif"
  magick "$SRC" -resize 512x512 -background none -gravity center -extent 512x512 \
    -quality 55 "$RES/kumkaj-mark-512.avif"
  echo "wrote kumkaj-mark*.avif"
fi

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
echo "Reinstall Android (cargo courier-android-dev) so the launcher picks up icons."
