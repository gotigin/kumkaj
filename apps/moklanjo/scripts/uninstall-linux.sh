#!/usr/bin/env bash
# Uninstall Moklanjo (Linux). Re-runnable: if app files are gone but
# books data remains, still offers to delete the data directory.
#
# Usage:
#   ./uninstall.sh
#   ~/.local/lib/kumkaj-moklanjo/uninstall.sh
#   ./apps/moklanjo/scripts/uninstall-linux.sh [--prod|--dev]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

IDS="$SCRIPT_DIR/desktop-ids.sh"
# (In the repo, desktop-ids.sh sits beside this script too.)
# shellcheck source=desktop-ids.sh
source "$IDS"

MODE=""
for arg in "$@"; do
  case "$arg" in
    --prod|--production) MODE=production ;;
    --dev|--debug|--development) MODE=development ;;
  esac
done

# Resolve identity: local manifest → config bookmark → flag → prod default.
load_identity() {
  if [[ -f "$SCRIPT_DIR/install-manifest.env" ]]; then
    desktop_load_manifest "$SCRIPT_DIR/install-manifest.env"
    return 0
  fi
  local cfg="${XDG_CONFIG_HOME:-$HOME/.config}/kumkaj"
  if [[ "$MODE" == "development" && -f "$cfg/kumkaj-moklanjo-debug.env" ]]; then
    desktop_load_manifest "$cfg/kumkaj-moklanjo-debug.env"
    return 0
  fi
  if [[ "$MODE" == "production" && -f "$cfg/kumkaj-moklanjo.env" ]]; then
    desktop_load_manifest "$cfg/kumkaj-moklanjo.env"
    return 0
  fi
  # Prefer whichever bookmark exists when no flag.
  if [[ -f "$cfg/kumkaj-moklanjo.env" ]]; then
    desktop_load_manifest "$cfg/kumkaj-moklanjo.env"
    return 0
  fi
  if [[ -f "$cfg/kumkaj-moklanjo-debug.env" ]]; then
    desktop_load_manifest "$cfg/kumkaj-moklanjo-debug.env"
    return 0
  fi
  if [[ "$MODE" == "development" ]]; then
    export KUMKAJ_ENV=development
    export KUMKAJ_APP_ID=com.moklanjo.debug
    export KUMKAJ_APP_LABEL="Moklanjo Dev"
    return 0
  fi
  export KUMKAJ_ENV=production
  export KUMKAJ_APP_ID=com.moklanjo
  export KUMKAJ_APP_LABEL="Moklanjo"
}

load_identity

LABEL="${KUMKAJ_APP_LABEL}"
INSTALL_DIR="$(desktop_install_dir_linux)"
# If we were launched from an install tree, trust that path.
if [[ -f "$SCRIPT_DIR/bin/kumkaj-moklanjo" || -f "$SCRIPT_DIR/run.sh" ]]; then
  INSTALL_DIR="$SCRIPT_DIR"
fi
BIN_DIR="$(desktop_bin_dir_linux)"
APP_DIR="$(desktop_applications_dir_linux)"
BIN_NAME="$(desktop_bin_name)"
ENTRY_ID="$(desktop_entry_id)"
DATA_DIR="$(desktop_data_dir_linux)"
SLUG="$(desktop_install_slug)"
CFG_BOOKMARK="${XDG_CONFIG_HOME:-$HOME/.config}/kumkaj/${SLUG}.env"

echo "Uninstall $LABEL"
echo "  App id:    $KUMKAJ_APP_ID"
echo "  App files: $INSTALL_DIR"
echo "  Data:      $DATA_DIR"

APP_PRESENT=0
[[ -d "$INSTALL_DIR" && ( -f "$INSTALL_DIR/run.sh" || -f "$INSTALL_DIR/bin/kumkaj-moklanjo" ) ]] && APP_PRESENT=1

if [[ "$APP_PRESENT" -eq 1 ]]; then
  echo
  echo "Removing app files…"
  rm -f "$BIN_DIR/$BIN_NAME"
  rm -f "$APP_DIR/${ENTRY_ID}.desktop"
  rm -f "$APP_DIR/${ENTRY_ID}-uninstall.desktop"
  # Remove install tree except we are running from inside it — delete after exit.
  if [[ "$SCRIPT_DIR" == "$INSTALL_DIR" ]]; then
    # Schedule delete of install dir after this script exits.
    CLEANUP_DIR="$INSTALL_DIR"
  else
    rm -rf "$INSTALL_DIR"
    CLEANUP_DIR=""
  fi
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
  fi
  echo "  App files removed."
else
  echo
  echo "App files already gone (or never installed here)."
  CLEANUP_DIR=""
fi

REMOVE_DATA=0
if desktop_prompt_remove_data "$DATA_DIR"; then
  REMOVE_DATA=1
fi

if [[ "$REMOVE_DATA" -eq 1 ]]; then
  echo "Removing data…"
  rm -rf "$DATA_DIR"
  echo "  Data removed: $DATA_DIR"
else
  if [[ -e "$DATA_DIR" ]]; then
    echo "Keeping data at: $DATA_DIR"
    echo "Run this uninstaller again later if you want to delete it."
  fi
fi

rm -f "$CFG_BOOKMARK"

echo
echo "Done."
if [[ "$REMOVE_DATA" -eq 0 && -e "$DATA_DIR" ]]; then
  echo "Reminder: books data is still on disk. Re-run uninstall to remove it."
fi

if [[ -n "${CLEANUP_DIR:-}" ]]; then
  # Detach and remove the install directory we are running from.
  nohup bash -c "sleep 1; rm -rf \"$CLEANUP_DIR\"" >/dev/null 2>&1 &
fi
