#!/usr/bin/env bash
# Stage a Windows payload + AppxManifest/Assets, pack with Microsoft makemsix
# (valid BlockMap for Partner Center). Bash + ImageMagick — no Python.
#
# Usage (normally via scripts/build-msix.sh):
#   pack_msix.sh --payload DIR --out FILE.msix --layout DIR \
#     --name Identity --publisher 'CN=…' --publisher-display NAME \
#     --display LABEL --version 1.0.0 --exe app.exe --description TEXT \
#     --icon-png PATH
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

PAYLOAD="" OUT="" LAYOUT="" NAME="" PUBLISHER="" PUBLISHER_DISPLAY="Mearaj Bhagad"
DISPLAY="" VERSION="" EXE="" DESCRIPTION="" ICON=""

usage() {
  sed -n '2,12p' "$0" | tr -d '#'
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --payload) PAYLOAD="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --layout) LAYOUT="$2"; shift 2 ;;
    --name) NAME="$2"; shift 2 ;;
    --publisher) PUBLISHER="$2"; shift 2 ;;
    --publisher-display) PUBLISHER_DISPLAY="$2"; shift 2 ;;
    --display) DISPLAY="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --exe) EXE="$2"; shift 2 ;;
    --description) DESCRIPTION="$2"; shift 2 ;;
    --icon-png) ICON="$2"; shift 2 ;;
    -h|--help) usage ;;
    *) echo "unknown arg: $1" >&2; usage ;;
  esac
done

: "${PAYLOAD:?}" "${OUT:?}" "${LAYOUT:?}" "${NAME:?}" "${PUBLISHER:?}"
: "${DISPLAY:?}" "${VERSION:?}" "${EXE:?}" "${ICON:?}"
DESCRIPTION="${DESCRIPTION:-$DISPLAY}"

if [[ ! -d "$PAYLOAD" ]]; then
  echo "payload not a directory: $PAYLOAD" >&2
  exit 1
fi
if [[ ! -f "$PAYLOAD/$EXE" ]]; then
  echo "missing $PAYLOAD/$EXE" >&2
  exit 1
fi
if [[ ! -f "$ICON" ]]; then
  echo "icon missing: $ICON" >&2
  exit 1
fi
command -v magick >/dev/null || { echo "ImageMagick 'magick' required" >&2; exit 1; }

# a.b.c → a.b.c.0
four_part() {
  local v="$1" n
  IFS=. read -r -a parts <<<"$v"
  n="${#parts[@]}"
  if [[ "$n" -eq 3 ]]; then
    echo "${parts[0]}.${parts[1]}.${parts[2]}.0"
  elif [[ "$n" -eq 4 ]]; then
    echo "${parts[0]}.${parts[1]}.${parts[2]}.${parts[3]}"
  else
    echo "version must be a.b.c or a.b.c.d, got: $v" >&2
    exit 1
  fi
}
VER="$(four_part "$VERSION")"

xml_esc() {
  # Escape &, <, >, " for attribute/text nodes.
  local s=$1
  s=${s//&/&amp;}
  s=${s//</&lt;}
  s=${s//>/&gt;}
  s=${s//\"/&quot;}
  printf '%s' "$s"
}

N_ESC="$(xml_esc "$NAME")"
P_ESC="$(xml_esc "$PUBLISHER")"
D_ESC="$(xml_esc "$DISPLAY")"
PD_ESC="$(xml_esc "$PUBLISHER_DISPLAY")"
E_ESC="$(xml_esc "$EXE")"
DESC_ESC="$(xml_esc "$DESCRIPTION")"

STAGE="$(mktemp -d "${TMPDIR:-/tmp}/kumkaj-msix-stage.XXXXXX")"
cleanup() { rm -rf "$STAGE"; }
trap cleanup EXIT

# --- copy payload (skip MSI/folder install helpers) ---
rsync -a \
  --exclude='Setup.bat' \
  --exclude='Uninstall.bat' \
  --exclude='install.ps1' \
  --exclude='uninstall.ps1' \
  --exclude='package.wxs' \
  --exclude='License.rtf' \
  --exclude='Quick start.txt' \
  --exclude='README.txt' \
  --exclude='msi-post-install.js' \
  --exclude='msi-uninstall-data.js' \
  --exclude='desktop-ids.sh' \
  --exclude='windows-wipe-prompt.ps1' \
  --exclude='delete-books-data.ps1' \
  --exclude='msi-stage' \
  --exclude='.git' \
  --exclude='*.wxs' \
  --exclude='*.msi' \
  --exclude='*.wixpdb' \
  "$PAYLOAD"/ "$STAGE"/

# makemsix regenerates these — drop any leftovers from old Python packs
rm -f "$STAGE/AppxBlockMap.xml" "$STAGE/[Content_Types].xml" "$STAGE/_Content_Types.xml"

# --- Store logos (ImageMagick) ---
ASSETS="$STAGE/Assets"
mkdir -p "$ASSETS"
logo() {
  local name=$1 w=$2 h=$3 mode=${4:-}
  if [[ "$mode" == letterbox ]]; then
    magick "$ICON" -background none -resize "${w}x${h}" -gravity center \
      -extent "${w}x${h}" "$ASSETS/$name"
  else
    magick "$ICON" -resize "${w}x${h}!" "$ASSETS/$name"
  fi
}
logo StoreLogo.png 50 50
logo Square44x44Logo.png 44 44
logo Square150x150Logo.png 150 150
logo Wide310x150Logo.png 310 150 letterbox
logo SplashScreen.png 620 300 letterbox

# --- AppxManifest.xml ---
cat >"$STAGE/AppxManifest.xml" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:uap10="http://schemas.microsoft.com/appx/manifest/uap/windows10/10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap uap10 rescap">
  <Identity Name="${N_ESC}" Publisher="${P_ESC}" Version="${VER}" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>${D_ESC}</DisplayName>
    <PublisherDisplayName>${PD_ESC}</PublisherDisplayName>
    <Description>${DESC_ESC}</Description>
    <Logo>Assets\\StoreLogo.png</Logo>
  </Properties>
  <Resources>
    <Resource Language="en-us" />
  </Resources>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.19041.0" MaxVersionTested="10.0.22621.0" />
  </Dependencies>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
    <Capability Name="internetClient" />
    <Capability Name="internetClientServer" />
    <Capability Name="privateNetworkClientServer" />
  </Capabilities>
  <Applications>
    <Application Id="App"
      Executable="${E_ESC}"
      EntryPoint="Windows.FullTrustApplication"
      uap10:TrustLevel="mediumIL"
      uap10:RuntimeBehavior="packagedClassicApp">
      <uap:VisualElements
        DisplayName="${D_ESC}"
        Description="${DESC_ESC}"
        BackgroundColor="transparent"
        Square150x150Logo="Assets\\Square150x150Logo.png"
        Square44x44Logo="Assets\\Square44x44Logo.png">
        <uap:DefaultTile Wide310x150Logo="Assets\\Wide310x150Logo.png" ShortName="${D_ESC}" />
        <uap:SplashScreen Image="Assets\\SplashScreen.png" BackgroundColor="transparent" />
      </uap:VisualElements>
    </Application>
  </Applications>
</Package>
EOF

# --- publish layout copy for debugging / MakeAppx fallback ---
rm -rf "$LAYOUT"
mkdir -p "$(dirname "$LAYOUT")"
rsync -a "$STAGE"/ "$LAYOUT"/

# --- pack with Microsoft makemsix ---
MAKE_MSIX="$("$ROOT/scripts/msix/ensure-makemsix.sh")"
LIBDIR="$(dirname "$MAKE_MSIX")"
export LD_LIBRARY_PATH="$LIBDIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

mkdir -p "$(dirname "$OUT")"
rm -f "$OUT"
echo "==> makemsix pack → $OUT"
"$MAKE_MSIX" pack -d "$STAGE" -p "$OUT"

SIZE="$(du -h "$OUT" | cut -f1)"
echo "MSIX → $OUT ($SIZE)"
echo "Identity Name=$NAME Publisher=$PUBLISHER Version=$VER"
echo "Packed with makemsix (Store-valid BlockMap). Unsigned OK for Partner Center upload."
