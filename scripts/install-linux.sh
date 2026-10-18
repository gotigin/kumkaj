#!/usr/bin/env bash
# Install Kumkaj Courier for the current user (Linux).
# Run from a package folder (dist/linux or dist/linux-dev) that contains:
#   kumkaj-courier                 (binary)
#   kumkaj-courier/resources/      (fonts + icons)
#   install-manifest.env
#   uninstall.sh
#   desktop-ids.sh
#
# Usage:
#   ./install.sh
#   ./scripts/install-linux.sh /path/to/package
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKG="${1:-}"
if [[ -z "$PKG" ]]; then
  if [[ -f "$SCRIPT_DIR/install-manifest.env" ]]; then
    PKG="$SCRIPT_DIR"
  elif [[ -f "$SCRIPT_DIR/../dist/linux/install-manifest.env" ]]; then
    PKG="$(cd "$SCRIPT_DIR/../dist/linux" && pwd)"
  else
    PKG="$SCRIPT_DIR"
  fi
fi
PKG="$(cd "$PKG" && pwd)"

IDS="$PKG/desktop-ids.sh"
[[ -f "$IDS" ]] || IDS="$SCRIPT_DIR/desktop-ids.sh"
# shellcheck source=desktop-ids.sh
source "$IDS"

MANIFEST="$PKG/install-manifest.env"
if [[ ! -f "$MANIFEST" ]]; then
  echo "Missing install-manifest.env in $PKG" >&2
  echo "Build with ./scripts/package-linux.sh (or package-linux-dev.sh)." >&2
  exit 1
fi
desktop_load_manifest "$MANIFEST"

BIN_SRC=""
for cand in "$PKG/bin/kumkaj-courier" "$PKG/kumkaj-courier"; do
  if [[ -f "$cand" ]]; then
    BIN_SRC="$cand"
    break
  fi
done
if [[ -z "$BIN_SRC" ]]; then
  echo "Missing binary: $PKG/bin/kumkaj-courier" >&2
  exit 1
fi
if [[ ! -d "$PKG/kumkaj-courier/resources" ]]; then
  echo "Missing resources: $PKG/kumkaj-courier/resources" >&2
  echo "Re-run the package script so fonts/icons ship beside the binary." >&2
  exit 1
fi

INSTALL_DIR="$(desktop_install_dir_linux)"
BIN_DIR="$(desktop_bin_dir_linux)"
APP_DIR="$(desktop_applications_dir_linux)"
BIN_NAME="$(desktop_bin_name)"
ENTRY_ID="$(desktop_entry_id)"
DATA_DIR="$(desktop_data_dir_linux)"
LABEL="${KUMKAJ_APP_LABEL}"
SLUG="$(desktop_install_slug)"

echo "Installing $LABEL"
echo "  App files:    $INSTALL_DIR"
echo "  Command:      $BIN_DIR/$BIN_NAME"
echo "  Data (later): $DATA_DIR"
echo "  App id:       $KUMKAJ_APP_ID"

rm -rf "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/bin" "$BIN_DIR" "$APP_DIR"

cp -f "$BIN_SRC" "$INSTALL_DIR/bin/kumkaj-courier"
chmod +x "$INSTALL_DIR/bin/kumkaj-courier"
# Makepad layout: resources at <install>/kumkaj-courier/resources next to run cwd.
mkdir -p "$INSTALL_DIR/kumkaj-courier"
rsync -a --delete "$PKG/kumkaj-courier/resources/" "$INSTALL_DIR/kumkaj-courier/resources/"

cat >"$INSTALL_DIR/run.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
ROOT="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export MAKEPAD_PACKAGE_DIR=.
cd "\$ROOT"
exec "\$ROOT/bin/kumkaj-courier" "\$@"
EOF
chmod +x "$INSTALL_DIR/run.sh"

cat >"$BIN_DIR/$BIN_NAME" <<EOF
#!/usr/bin/env bash
exec "$INSTALL_DIR/run.sh" "\$@"
EOF
chmod +x "$BIN_DIR/$BIN_NAME"

UNINSTALL_SRC="$PKG/uninstall.sh"
[[ -f "$UNINSTALL_SRC" ]] || UNINSTALL_SRC="$SCRIPT_DIR/uninstall-linux.sh"
cp -f "$UNINSTALL_SRC" "$INSTALL_DIR/uninstall.sh"
chmod +x "$INSTALL_DIR/uninstall.sh"
cp -f "$IDS" "$INSTALL_DIR/desktop-ids.sh"

desktop_write_manifest "$INSTALL_DIR/install-manifest.env"
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/kumkaj"
desktop_write_manifest "${XDG_CONFIG_HOME:-$HOME/.config}/kumkaj/${SLUG}.env"

ICON="$INSTALL_DIR/kumkaj-courier/resources/icon_256.png"
[[ -f "$ICON" ]] || ICON="$INSTALL_DIR/kumkaj-courier/resources/icon_128.png"

cat >"$APP_DIR/${ENTRY_ID}.desktop" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=$LABEL
Comment=Courier books + booking desk
Exec=$INSTALL_DIR/run.sh
Icon=$ICON
Terminal=false
Categories=Office;Finance;
StartupWMClass=kumkaj-courier
EOF

cat >"$APP_DIR/${ENTRY_ID}-uninstall.desktop" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=Uninstall $LABEL
Comment=Remove $LABEL (asks whether to keep books data)
Exec=$INSTALL_DIR/uninstall.sh
Terminal=true
Categories=Utility;
EOF

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

echo
echo "Installed $LABEL."
echo "  Start:  $BIN_NAME   or app menu \"$LABEL\""
echo "  Remove: app menu \"Uninstall $LABEL\"  or  $INSTALL_DIR/uninstall.sh"
echo "  Data:   $DATA_DIR  (created on first run; uninstall asks keep vs delete)"
