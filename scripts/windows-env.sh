# Source from package / windows-dev scripts.
# Prefers x86_64-pc-windows-gnu (cross-compile from Linux with mingw-w64).
# One PE binary targets Windows 10 and Windows 11 (x86_64).
#
# Makepad desktop packaging (same model as Makepad Builder / packaging guide):
#   MAKEPAD_PACKAGE_DIR=.
#   resources live beside the .exe as <crate>/resources/...

windows_default_target() {
  echo "x86_64-pc-windows-gnu"
}

windows_ensure_toolchain() {
  local target
  target="$(windows_default_target)"
  if ! rustup target list --installed 2>/dev/null | grep -qx "$target"; then
    echo "Installing Rust target $target …"
    rustup target add "$target"
  fi
  if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "Missing x86_64-w64-mingw32-gcc (Arch: pacman -S mingw-w64-gcc)" >&2
    return 1
  fi
  export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="${CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER:-x86_64-w64-mingw32-gcc}"
  export CC_x86_64_pc_windows_gnu="${CC_x86_64_pc_windows_gnu:-x86_64-w64-mingw32-gcc}"
  export CXX_x86_64_pc_windows_gnu="${CXX_x86_64_pc_windows_gnu:-x86_64-w64-mingw32-g++}"
  echo "Windows cross: target=$target linker=$CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER"
}

# Compile-time package root: load resources relative to the .exe directory.
makepad_enable_package_dir() {
  export MAKEPAD_PACKAGE_DIR=.
}

# Stage desktop resources next to kumkaj-courier.exe.
# One tree only: kumkaj-courier/resources/ (fonts + icons + menu).
# Do NOT copy vendor/makepad/widgets/resources — that duplicated Noto/FA and
# shipped unused IBM Plex / studio icons. WorldFonts uses self:resources/fonts/.
makepad_copy_desktop_resources() {
  local out="$1"
  local root="${KUMKAJ_ROOT:-}"
  if [[ -z "$root" ]]; then
    echo "makepad_copy_desktop_resources: KUMKAJ_ROOT not set" >&2
    return 1
  fi

  rm -rf "$out/kumkaj-courier" "$out/kumkaj" "$out/makepad_widgets"
  mkdir -p "$out/kumkaj-courier/resources/fonts"

  rsync -a --exclude 'android/' \
    "$root/apps/courier/resources/" "$out/kumkaj-courier/resources/"

  # CJK + colour emoji: live in vendor Makepad; WorldFonts expects them under
  # kumkaj-courier/resources/fonts (same folder as the other script fonts).
  local widgets_res="$root/vendor/makepad/widgets/resources"
  local fonts_out="$out/kumkaj-courier/resources/fonts"
  local f
  for f in LXGWWenKaiRegular.ttf LXGWWenKaiBold.ttf NotoColorEmoji.ttf; do
    if [[ -f "$fonts_out/$f" ]]; then
      continue
    fi
    if [[ -f "$widgets_res/$f" ]]; then
      cp -f "$widgets_res/$f" "$fonts_out/$f"
    else
      echo "warn: missing $f (run ./scripts/setup-makepad.sh + ./scripts/fetch-ui-fonts.sh)" >&2
    fi
  done
}
