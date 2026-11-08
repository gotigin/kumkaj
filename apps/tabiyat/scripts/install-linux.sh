#!/usr/bin/env bash
# Install Tabiyat for the current user (Linux).
# Run from a package folder (dist/linux or dist/linux-dev) that contains:
#   kumkaj-tabiyat                 (binary under bin/)
#   kumkaj_tabiyat/resources/      (fonts + icons; underscores = Makepad crate name)
#   install-manifest.env
#   uninstall.sh
#   desktop-ids.sh
#
# Usage:
#   ./install.sh
#   ./apps/tabiyat/scripts/install-linux.sh /path/to/package
#   ./apps/tabiyat/scripts/install-linux.sh --dev
#     (from the repo: rebuilds dist/tabiyat-linux-dev/, then installs it)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODE=""
PKG=""
for arg in "$@"; do
  case "$arg" in
    --dev|--debug|--development) MODE=development ;;
    --prod|--production) MODE=production ;;
    *)
      if [[ -n "$PKG" ]]; then
        echo "Unexpected argument: $arg" >&2
        exit 1
      fi
      PKG="$arg"
      ;;
  esac
done
if [[ -z "$PKG" && "$MODE" == "development" && -f "$SCRIPT_DIR/package-linux-dev.sh" ]]; then
  # Always rebuild. A package left in dist/ would install yesterday's binary.
  echo "Building the development package, then installing it."
  "$SCRIPT_DIR/package-linux-dev.sh"
  DEV_PKG="$SCRIPT_DIR/../../../dist/tabiyat-linux-dev"
  if [[ ! -f "$DEV_PKG/install-manifest.env" ]]; then
    echo "No development package at dist/tabiyat-linux-dev/" >&2
    exit 1
  fi
  PKG="$(cd "$DEV_PKG" && pwd)"
fi
if [[ -z "$PKG" ]]; then
  if [[ -f "$SCRIPT_DIR/install-manifest.env" ]]; then
    # Staged copy inside a package folder. That folder is the package.
    PKG="$SCRIPT_DIR"
  elif [[ -f "$SCRIPT_DIR/../../../dist/tabiyat-linux/install-manifest.env" ]]; then
    PKG="$(cd "$SCRIPT_DIR/../../../dist/tabiyat-linux" && pwd)"
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
  echo "Build with ./apps/tabiyat/scripts/package-linux.sh (or package-linux-dev.sh)." >&2
  exit 1
fi
desktop_load_manifest "$MANIFEST"

BIN_SRC=""
for cand in "$PKG/bin/kumkaj-tabiyat" "$PKG/kumkaj-tabiyat"; do
  if [[ -f "$cand" ]]; then
    BIN_SRC="$cand"
    break
  fi
done
if [[ -z "$BIN_SRC" ]]; then
  echo "Missing binary: $PKG/bin/kumkaj-tabiyat" >&2
  exit 1
fi
RES_DIR="kumkaj_tabiyat"
if [[ ! -d "$PKG/$RES_DIR/resources" ]]; then
  if [[ -d "$PKG/kumkaj-tabiyat/resources" ]]; then
    echo "Found legacy hyphenated resources at $PKG/kumkaj-tabiyat/resources" >&2
    echo "Makepad needs kumkaj_tabiyat/resources (underscores). Re-run the package script." >&2
    exit 1
  fi
  echo "Missing resources: $PKG/$RES_DIR/resources" >&2
  echo "Re-run the package script so fonts/icons ship beside the binary." >&2
  exit 1
fi
if [[ ! -f "$PKG/$RES_DIR/resources/fonts/NotoSans-Regular.ttf" ]]; then
  echo "Missing UI font: $PKG/$RES_DIR/resources/fonts/NotoSans-Regular.ttf" >&2
  echo "Run ./scripts/fetch-ui-fonts.sh apps/tabiyat then re-package." >&2
  exit 1
fi
if [[ ! -f "$PKG/$RES_DIR/resources/fonts/fa-solid-900.ttf" ]]; then
  echo "Missing icon font: $PKG/$RES_DIR/resources/fonts/fa-solid-900.ttf" >&2
  echo "Floor/header icon buttons will be empty boxes without it." >&2
  echo "Run ./scripts/fetch-ui-fonts.sh apps/tabiyat then re-package." >&2
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

cp -f "$BIN_SRC" "$INSTALL_DIR/bin/kumkaj-tabiyat"
chmod +x "$INSTALL_DIR/bin/kumkaj-tabiyat"
# Makepad layout: resources at <install>/kumkaj_tabiyat/resources next to run cwd.
mkdir -p "$INSTALL_DIR/$RES_DIR"
rsync -a --delete "$PKG/$RES_DIR/resources/" "$INSTALL_DIR/$RES_DIR/resources/"
# Menu shelf + client templates (package root; exe under bin/ → lookup goes up).
if [[ -d "$PKG/assets" ]]; then
  rsync -a --delete "$PKG/assets/" "$INSTALL_DIR/assets/"
fi
# Drop any leftover hyphenated tree from older installs.
rm -rf "$INSTALL_DIR/kumkaj-tabiyat"

cat >"$INSTALL_DIR/run.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
ROOT="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export MAKEPAD_PACKAGE_DIR=.
cd "\$ROOT"
exec "\$ROOT/bin/kumkaj-tabiyat" "\$@"
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

ICON="$INSTALL_DIR/$RES_DIR/resources/icon_256.png"
[[ -f "$ICON" ]] || ICON="$INSTALL_DIR/$RES_DIR/resources/icon_128.png"

cat >"$APP_DIR/${ENTRY_ID}.desktop" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=$LABEL
Comment=Restaurant orders, kitchen and books
Exec=$INSTALL_DIR/run.sh
Icon=$ICON
Terminal=false
Categories=Office;Finance;
StartupWMClass=kumkaj-tabiyat
EOF

cat >"$APP_DIR/${ENTRY_ID}-uninstall.desktop" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=Uninstall $LABEL
Comment=Remove $LABEL (asks which local data to keep)
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
