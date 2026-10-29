#!/usr/bin/env bash
# Build a per-user MSI from a staged Windows package folder. Shared by the
# apps; each app's own msi.sh sets who it is and calls this:
#   apps/moklanjo/scripts/msi.sh [prod|dev]   -> dist/Moklanjo[Dev]-<ver>.msi
#   apps/khaynjo/scripts/msi.sh [prod|dev]   -> dist/Khaynjo[Dev]-<ver>.msi
# Required from the caller (no default app — a missing one must fail, not
# build another product's installer):
#   KUMKAJ_APP_DIR, KUMKAJ_MSI_NAME, KUMKAJ_BIN_NAME,
#   KUMKAJ_MSI_UPGRADE_CODE, KUMKAJ_MSI_DEV_UPGRADE_CODE (never change them:
#   upgrades find the installed app by these),
#   KUMKAJ_MSI_PKG_PROD, KUMKAJ_MSI_PKG_DEV, KUMKAJ_MSI_PACKAGE_PROD.
# The licence page is the app's own scripts/License.rtf.
# The Windows package is always rebuilt first, so the MSI is never stale.
#
# The MSI installs to %LOCALAPPDATA%\Programs\<Label>\ (no admin), adds a
# Start Menu shortcut, and appears in Settings -> Apps for uninstall.
# A full uninstall asks which local data to delete (business records, shop
# setup / schema, screen preferences under %APPDATA%\<app id>\); upgrades
# never ask and always keep it. A Start Menu "Delete books data" shortcut
# offers the same categories on demand. Uses wixl (msitools). If missing, a
# pinned copy is downloaded to the user cache (Arch package mirrors).
# Override with WIXL=/path/to/wixl.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

need() { [[ -n "${!1:-}" ]] || { echo "build-msi.sh: $1 not set — run the app's scripts/msi.sh" >&2; exit 1; }; }
for v in KUMKAJ_APP_DIR KUMKAJ_MSI_NAME KUMKAJ_BIN_NAME KUMKAJ_MSI_UPGRADE_CODE \
  KUMKAJ_MSI_DEV_UPGRADE_CODE KUMKAJ_MSI_PKG_PROD KUMKAJ_MSI_PKG_DEV KUMKAJ_MSI_PACKAGE_PROD; do
  need "$v"
done
APP_DIR="$KUMKAJ_APP_DIR"
MSI_NAME="$KUMKAJ_MSI_NAME"
# The app's exe (Makepad keeps its resources in the same name with _).
BIN="$KUMKAJ_BIN_NAME"
RES_DIR="${BIN//-/_}"

FLAVOR="${1:-prod}"
case "$FLAVOR" in
  prod)
    PKG="$KUMKAJ_MSI_PKG_PROD"
    UPGRADE_CODE="$KUMKAJ_MSI_UPGRADE_CODE"
    OUT_NAME="$MSI_NAME"
    ;;
  dev)
    PKG="$KUMKAJ_MSI_PKG_DEV"
    UPGRADE_CODE="$KUMKAJ_MSI_DEV_UPGRADE_CODE"
    OUT_NAME="${MSI_NAME}Dev"
    ;;
  *) echo "usage: $0 [prod|dev]" >&2; exit 1;;
esac

# Always rebuild the Windows package first (cargo is incremental - cheap
# when nothing changed) so the MSI never ships a stale exe or resources.
case "$FLAVOR" in
  prod) "$KUMKAJ_MSI_PACKAGE_PROD" ;;
  dev)  "$APP_DIR/scripts/windows-dev.sh" ;;
esac
if [[ ! -f "$PKG/$BIN.exe" || ! -d "$PKG/$RES_DIR/resources/fonts" ]]; then
  echo "Windows package missing after build: $PKG" >&2
  exit 1
fi

# --- inputs from the package manifest ---------------------------------------
manifest_get() {
  sed -n "s/^$1=//p" "$PKG/install-manifest.env" | head -1 | tr -d '"'
}
LABEL="$(manifest_get KUMKAJ_APP_LABEL)"
[[ -n "$LABEL" ]] || { echo "install-manifest.env missing KUMKAJ_APP_LABEL" >&2; exit 1; }
APP_ID="$(manifest_get KUMKAJ_APP_ID)"
[[ -n "$APP_ID" ]] || { echo "install-manifest.env missing KUMKAJ_APP_ID" >&2; exit 1; }
# Per-flavor delete-data helper (app id / label / exe name substituted in).
sed -e "s/__APP_ID__/$APP_ID/g" -e "s/__APP_LABEL__/$LABEL/g" -e "s/__BIN_NAME__/$BIN/g" \
  "$ROOT/scripts/delete-books-data.ps1" >"$PKG/delete-books-data.ps1"
# Uninstall prompt script (same substitution; JScript is all wixl supports).
sed -e "s/__APP_ID__/$APP_ID/g" -e "s/__APP_LABEL__/$LABEL/g" -e "s/__BIN_NAME__/$BIN/g" \
  "$ROOT/scripts/msi-uninstall-data.js" >"$PKG/msi-uninstall-data.js"
VER="$(sed -n '/^\[package\]/,/^\[/p' "$APP_DIR/Cargo.toml" | sed -n 's/^version = "\(.*\)"/\1/p' | head -1)"
[[ -n "$VER" ]] || VER="0.1.0"

xml_escape() {
  sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g'
}

# --- generate the WiX source --------------------------------------------------
WXS="$PKG/package.wxs"
COMP_REFS_FILE="$(mktemp)"
trap 'rm -f "$COMP_REFS_FILE"' EXIT
COUNTER=0
DIR_COUNTER=0

# emit_file <wxs-file-descriptor> <indent> <fs-path> [shortcut]
emit_file() {
  local indent="$2" fspath="$3" shortcut="${4:-}"
  COUNTER=$((COUNTER + 1))
  local cid="C$COUNTER" fid="F$COUNTER" name
  name="$(basename "$fspath")"
  {
    printf '%s<Component Id="%s" Guid="*">\n' "$indent" "$cid"
    printf '%s  <File Id="%s" Source="%s" Name="%s" KeyPath="yes"/>\n' \
      "$indent" "$fid" "$(printf '%s' "$fspath" | xml_escape)" "$(printf '%s' "$name" | xml_escape)"
    if [[ "$shortcut" == "3" ]]; then
      printf '%s  <Shortcut Id="QuickStartShortcut" Directory="AppMenuFolder" Name="Quick start" Target="[INSTALLFOLDER]Quick start.txt" WorkingDirectory="INSTALLFOLDER" Description="Set up other devices with %s"/>\n' \
        "$indent" "$(printf '%s' "$LABEL" | xml_escape)"
    fi
    if [[ "$shortcut" == "2" ]]; then
      printf '%s  <Shortcut Id="DeleteDataShortcut" Directory="AppMenuFolder" Name="Delete books data" Target="[SystemFolder]WindowsPowerShell\\v1.0\\powershell.exe" Arguments="-NoProfile -ExecutionPolicy Bypass -File &quot;[INSTALLFOLDER]delete-books-data.ps1&quot;" WorkingDirectory="INSTALLFOLDER" Description="Delete %s books data"/>\n' \
        "$indent" "$(printf '%s' "$LABEL" | xml_escape)"
    fi
    if [[ "$shortcut" == "1" ]]; then
      printf '%s  <Shortcut Id="AppShortcut" Directory="AppMenuFolder" Name="%s" Target="[INSTALLFOLDER]%s" WorkingDirectory="INSTALLFOLDER" Icon="AppIcon.ico" Description="%s"/>\n' \
        "$indent" "$(printf '%s' "$LABEL" | xml_escape)" "$name" "$(printf '%s' "$LABEL" | xml_escape)"
      printf '%s  <Shortcut Id="UninstallShortcut" Directory="AppMenuFolder" Name="Uninstall %s" Target="[SystemFolder]msiexec.exe" Arguments="/x [ProductCode]" Description="Uninstall %s"/>\n' \
        "$indent" "$(printf '%s' "$LABEL" | xml_escape)" "$(printf '%s' "$LABEL" | xml_escape)"
    fi
    printf '%s</Component>\n' "$indent"
  } >>"$WXS.open"
  echo "            <ComponentRef Id=\"$cid\"/>" >>"$COMP_REFS_FILE"
}

# emit_tree <indent> <fs-dir> : Directory element wrapping the whole subtree
emit_tree() {
  local indent="$1" fsdir="$2" sub f
  while IFS= read -r -d '' sub; do
    DIR_COUNTER=$((DIR_COUNTER + 1))
    printf '%s<Directory Id="D%s" Name="%s">\n' "$indent" "$DIR_COUNTER" "$(basename "$sub" | xml_escape)" >>"$WXS.open"
    emit_tree "$indent  " "$sub"
    printf '%s</Directory>\n' "$indent" >>"$WXS.open"
  done < <(find "$fsdir" -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)
  while IFS= read -r -d '' f; do
    emit_file "$WXS.open" "$indent" "$f"
  done < <(find "$fsdir" -mindepth 1 -maxdepth 1 -type f -print0 | sort -z)
}

{
  printf '<?xml version="1.0" encoding="UTF-8"?>\n'
  printf '<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">\n'
  printf '  <Product Id="*" Name="%s" Language="1033" Version="%s" Manufacturer="Kumkaj" UpgradeCode="%s">\n' \
    "$(printf '%s' "$LABEL" | xml_escape)" "$VER" "$UPGRADE_CODE"
  printf '    <Package InstallerVersion="500" Compressed="yes" InstallScope="perUser" Description="%s installer" Manufacturer="Kumkaj"/>\n' \
    "$(printf '%s' "$LABEL" | xml_escape)"
  printf '    <MediaTemplate EmbedCab="yes"/>\n'
  printf '    <Property Id="ALLUSERS" Value="2"/>\n'
  printf '    <Property Id="MSIINSTALLPERUSER" Value="1"/>\n'
  # ARPNOMODIFY is defined by WixUI_Minimal - defining it again breaks wixl.
  printf '    <Property Id="ARPNOREPAIR" Value="1"/>\n'
  printf '    <MajorUpgrade DowngradeErrorMessage="A newer version of %s is already installed."/>\n' "$(printf '%s' "$LABEL" | xml_escape)"
  # Installer wizard (Welcome/License -> progress -> Finish). Without this the
  # MSI installs silently and the window just vanishes on completion.
  printf '    <UIRef Id="WixUI_Minimal"/>\n'
  # Uninstall-only data prompt. Conditioned on a full uninstall that is not
  # an upgrade, Execute=immediate + Return=ignore and a fully guarded script:
  # it cannot run during install and cannot fail the uninstall. (An earlier
  # attempt placed a prompt in the install path; that is what broke installs.)
  printf '    <Binary Id="UninstallDataJs" SourceFile="%s"/>\n' "$PKG/msi-uninstall-data.js"
  printf '    <CustomAction Id="PromptDeleteBooksData" BinaryKey="UninstallDataJs" JScriptCall="PromptDeleteBooksData" Execute="immediate" Return="ignore"/>\n'
  printf '    <InstallExecuteSequence>\n'
  printf '      <Custom Action="PromptDeleteBooksData" Before="RemoveFiles">REMOVE="ALL" AND NOT UPGRADINGPRODUCTCODE</Custom>\n'
  printf '    </InstallExecuteSequence>\n'
  if [[ -f "$PKG/$RES_DIR/resources/icon.ico" ]]; then
    printf '    <Icon Id="AppIcon.ico" SourceFile="%s"/>\n' "$PKG/$RES_DIR/resources/icon.ico"
    printf '    <Property Id="ARPPRODUCTICON" Value="AppIcon.ico"/>\n'
  fi
  printf '    <Directory Id="TARGETDIR" Name="SourceDir">\n'
  printf '      <Directory Id="LocalAppDataFolder">\n'
  printf '        <Directory Id="ProgramsFolder" Name="Programs">\n'
  printf '          <Directory Id="INSTALLFOLDER" Name="%s">\n' "$(printf '%s' "$LABEL" | xml_escape)"
} >"$WXS.open"

# exe (with shortcuts) + delete-data helper + <app>/ resources tree
emit_file "$WXS.open" "            " "$PKG/$BIN.exe" 1
emit_file "$WXS.open" "            " "$PKG/delete-books-data.ps1" 2
if [[ -f "$PKG/Quick start.txt" ]]; then
  emit_file "$WXS.open" "            " "$PKG/Quick start.txt" 3
fi
DIR_COUNTER=$((DIR_COUNTER + 1))
printf '            <Directory Id="D%s" Name="%s">\n' "$DIR_COUNTER" "$RES_DIR" >>"$WXS.open"
emit_tree "              " "$PKG/$RES_DIR"
printf '            </Directory>\n' >>"$WXS.open"

{
  printf '          </Directory>\n'
  printf '        </Directory>\n'
  printf '      </Directory>\n'
  printf '      <Directory Id="ProgramMenuFolder">\n'
  printf '        <Directory Id="AppMenuFolder" Name="%s">\n' "$(printf '%s' "$LABEL" | xml_escape)"
  printf '          <Component Id="AppMenuFolderComp" Guid="*">\n'
  printf '            <CreateFolder/>\n'
  printf '            <RemoveFolder Id="RemoveAppMenuFolder" On="uninstall"/>\n'
  printf '            <RegistryValue Root="HKCU" Key="SOFTWARE\\Kumkaj\\%s" Name="StartMenu" Type="string" Value="1" KeyPath="yes"/>\n' \
    "$(printf '%s' "$LABEL" | xml_escape)"
  printf '          </Component>\n'
  printf '        </Directory>\n'
  printf '      </Directory>\n'
  printf '    </Directory>\n'
  printf '    <Feature Id="Main" Title="%s" Level="1">\n' "$(printf '%s' "$LABEL" | xml_escape)"
  printf '      <ComponentRef Id="AppMenuFolderComp"/>\n'
  cat "$COMP_REFS_FILE"
  printf '    </Feature>\n'
  printf '  </Product>\n'
  printf '</Wix>\n'
} >>"$WXS.open"
mv "$WXS.open" "$WXS"

# --- locate wixl -------------------------------------------------------------
WIXL_BIN="${WIXL:-}"
WIXL_LD=""
if [[ -z "$WIXL_BIN" ]] && command -v wixl >/dev/null 2>&1; then
  WIXL_BIN="$(command -v wixl)"
fi
if [[ -z "$WIXL_BIN" ]]; then
  CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/kumkaj-msitools"
  if [[ ! -x "$CACHE/usr/bin/wixl" ]]; then
    echo "wixl not found - downloading msitools into $CACHE ..."
    mkdir -p "$CACHE"
    base="https://archive.archlinux.org/packages"
    curl -fsSL -o "$CACHE/msitools.pkg.tar.zst" "$base/m/msitools/msitools-0.106-3-x86_64.pkg.tar.zst"
    curl -fsSL -o "$CACHE/gcab.pkg.tar.zst" "$base/g/gcab/gcab-1.6-2-x86_64.pkg.tar.zst"
    tar -I zstd -xf "$CACHE/msitools.pkg.tar.zst" -C "$CACHE"
    tar -I zstd -xf "$CACHE/gcab.pkg.tar.zst" -C "$CACHE"
  fi
  WIXL_BIN="$CACHE/usr/bin/wixl"
  WIXL_LD="$CACHE/usr/lib"
fi

OUT="$ROOT/dist/${OUT_NAME}-${VER}.msi"
echo "Building MSI with $WIXL_BIN ..."
if [[ -n "$WIXL_LD" ]]; then
  export LD_LIBRARY_PATH="$WIXL_LD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
# Locate the wixl UI extension (system install or downloaded cache).
WIXL_PREFIX="$(cd "$(dirname "$WIXL_BIN")/.." && pwd)"
EXTDIR=""
for cand in "$WIXL_PREFIX/share/wixl-0.106/ext" "/usr/share/wixl-0.106/ext"; do
  if [[ -d "$cand/ui" ]]; then EXTDIR="$cand"; break; fi
done
[[ -n "$EXTDIR" ]] || { echo "wixl UI extension not found (msitools ext/ui)" >&2; exit 1; }
# WixUI_Minimal resolves License.rtf beside the .wxs / via -I: the app's own.
cp -f "$APP_DIR/scripts/License.rtf" "$PKG/License.rtf"
"$WIXL_BIN" --ext ui --extdir "$EXTDIR" --wxidir "$WIXL_PREFIX/share/wixl-0.106/include" -I "$PKG" -o "$OUT" "$WXS"
echo "MSI: $OUT ($(du -h "$OUT" | awk '{print $1}'))"
