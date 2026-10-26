#!/usr/bin/env bash
# Stage the app's install/uninstall helpers + manifest into a desktop
# package folder. The helpers are each app's own (apps/<app>/scripts/:
# install-linux.sh, uninstall-linux.sh, install-windows.ps1,
# uninstall-windows.ps1, Setup.bat, Uninstall.bat, desktop-ids.sh) — they
# name that app's exe, folders and labels.
# Usage (after binary + resources are in $OUT):
#   source "$KUMKAJ_APP_DIR/scripts/desktop-ids.sh"
#   # KUMKAJ_* already exported (KUMKAJ_APP_DIR, KUMKAJ_BIN_NAME, env ids)
#   stage_desktop_package "$OUT" linux|windows
stage_desktop_package() {
  local out="${1:?}"
  local os="${2:?}" # linux | windows
  local scripts="${KUMKAJ_APP_DIR:?stage_desktop_package: set KUMKAJ_APP_DIR}/scripts"
  local bin="${KUMKAJ_BIN_NAME:?stage_desktop_package: set KUMKAJ_BIN_NAME}"

  mkdir -p "$out"
  cp -f "$scripts/desktop-ids.sh" "$out/desktop-ids.sh"
  # The quick-start card (other devices, pairing, what happens when…),
  # readable in any text viewer; the MSI adds a Start Menu entry for it.
  local root
  root="$(cd "$scripts/../../.." && pwd)"
  cp -f "$root/docs/LAN_QUICK_START.md" "$out/Quick start.txt"

  # Manifest baked into the package (install copies it into the install dir).
  {
    echo "# Packaged with ${KUMKAJ_APP_LABEL:?} — identifies prod vs debug install."
    echo "KUMKAJ_ENV=${KUMKAJ_ENV:?}"
    echo "KUMKAJ_APP_ID=${KUMKAJ_APP_ID:?}"
    echo "KUMKAJ_APP_LABEL=${KUMKAJ_APP_LABEL:?}"
    echo "INSTALL_SLUG=$(desktop_install_slug)"
    echo "BIN_NAME=$(desktop_bin_name)"
    echo "ENTRY_ID=$(desktop_entry_id)"
  } >"$out/install-manifest.env"

  if [[ "$os" == "linux" ]]; then
    cp -f "$scripts/install-linux.sh" "$out/install.sh"
    cp -f "$scripts/uninstall-linux.sh" "$out/uninstall.sh"
    chmod +x "$out/install.sh" "$out/uninstall.sh" "$out/desktop-ids.sh"
    # Portable launcher without full install (same layout as after install).
    cat >"$out/run.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
ROOT="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export MAKEPAD_PACKAGE_DIR=.
cd "\$ROOT"
exec "\$ROOT/bin/$bin" "\$@"
EOF
    chmod +x "$out/run.sh"
  else
    cp -f "$scripts/install-windows.ps1" "$out/install.ps1"
    cp -f "$scripts/uninstall-windows.ps1" "$out/uninstall.ps1"
    # Double-click entry points (opening .ps1 in Explorer often launches Notepad).
    cp -f "$scripts/Setup.bat" "$out/Setup.bat"
    cp -f "$scripts/Uninstall.bat" "$out/Uninstall.bat"
  fi
}
