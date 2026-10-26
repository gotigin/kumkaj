# Source from an app's package / windows-dev scripts (KUMKAJ_APP_DIR set).
# Prefers x86_64-pc-windows-gnu (cross-compile from Linux with mingw-w64).
# One PE binary targets Windows 10 and Windows 11 (x86_64).
#
# Makepad desktop packaging (same model as Makepad Builder / packaging guide):
#   MAKEPAD_PACKAGE_DIR=.
#   resources live beside the .exe as <rust_crate_name>/resources/...
#   Cargo package `kumkaj-khaynjo` → rustc module `kumkaj_khaynjo` (underscores).
#   Packaged dep paths are therefore `kumkaj_khaynjo/resources/...` — NOT hyphens.

# Folder name Makepad opens at runtime for `self:resources/...`.
# Derived from the app being packaged (KUMKAJ_CRATE_DIR override, else the
# package name in $KUMKAJ_APP_DIR/Cargo.toml, hyphens → underscores).
makepad_crate_resources_dirname() {
  if [[ -n "${KUMKAJ_CRATE_DIR:-}" ]]; then
    echo "$KUMKAJ_CRATE_DIR"
    return
  fi
  local app_dir="${KUMKAJ_APP_DIR:-}"
  if [[ -n "$app_dir" && -f "$app_dir/Cargo.toml" ]]; then
    local pkg
    pkg="$(grep -m1 '^name = ' "$app_dir/Cargo.toml" | cut -d'"' -f2 | tr '-' '_')"
    if [[ -n "$pkg" ]]; then
      echo "$pkg"
      return
    fi
  fi
  echo "makepad_crate_resources_dirname: set KUMKAJ_APP_DIR (apps/<app>)" >&2
  return 1
}

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

# Compile-time package root: load resources relative to the .exe / cwd.
makepad_enable_package_dir() {
  export MAKEPAD_PACKAGE_DIR=.
}

# Stage desktop resources next to the app's .exe / linux package root.
# One tree only: <crate>/resources/ (fonts + icons). Underscores required.
# Do NOT copy vendor/makepad/widgets/resources as a second tree.
makepad_copy_desktop_resources() {
  local out="$1"
  local root="${KUMKAJ_ROOT:-}"
  if [[ -z "$root" ]]; then
    echo "makepad_copy_desktop_resources: KUMKAJ_ROOT not set" >&2
    return 1
  fi

  local crate_dir
  crate_dir="$(makepad_crate_resources_dirname)" || return 1
  # Remove both the old hyphenated and the current underscore tree.
  rm -rf "$out/${crate_dir//_/-}" "$out/$crate_dir" "$out/kumkaj" "$out/makepad_widgets"
  mkdir -p "$out/$crate_dir/resources/fonts"

  rsync -a --exclude 'android/' \
    "${KUMKAJ_APP_DIR:?}/resources/" "$out/$crate_dir/resources/"

  # CJK + colour emoji: live in vendor Makepad; WorldFonts expects them under
  # <crate>/resources/fonts (same folder as the other script fonts).
  local widgets_res="$root/vendor/makepad/widgets/resources"
  local fonts_out="$out/$crate_dir/resources/fonts"
  local f
  for f in LXGWWenKaiRegular.ttf LXGWWenKaiBold.ttf NotoColorEmoji.ttf; do
    if [[ -f "$fonts_out/$f" ]]; then
      continue
    fi
    if [[ -f "$widgets_res/$f" ]]; then
      cp -f "$widgets_res/$f" "$fonts_out/$f"
    else
      echo "warn: missing $f (run ./scripts/setup-makepad.sh + ./scripts/fetch-ui-fonts.sh $KUMKAJ_APP_DIR)" >&2
    fi
  done

  if [[ ! -f "$fonts_out/NotoSans-Regular.ttf" ]]; then
    echo "error: missing NotoSans-Regular.ttf under $fonts_out" >&2
    echo "Run ./scripts/fetch-ui-fonts.sh $KUMKAJ_APP_DIR before packaging." >&2
    return 1
  fi
}
