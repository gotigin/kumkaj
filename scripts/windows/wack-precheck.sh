#!/usr/bin/env bash
# Static pre-check (Linux) of what the Windows App Certification Kit reads from
# a Kumkaj Windows build, so most WACK failures show up before a Windows box:
#
#   scripts/windows/wack-precheck.sh <app.exe> [<package.msix>]
#
# .exe  — PE32+ x64, GUI subsystem (production), ASLR + high-entropy ASLR + DEP
#         (DYNAMIC_BASE, HIGH_ENTROPY_VA, NX_COMPAT: WACK binary analyzer),
#         a resource section with the application manifest (asInvoker, Windows
#         10/11 supportedOS, PerMonitorV2), version info and an icon
#         (scripts/windows/win_resources.rs).
# .msix — AppxManifest.xml: Identity / Publisher, runFullTrust, desktop
#         MinVersion, every Logo the manifest names is in the package, no
#         debug-only files.
#
# The real run is on Windows: scripts/windows/run-wack.ps1 (DEV.md § Windows
# App Certification Kit). Exit 0 = every check passed.
set -uo pipefail
EXE="${1:?usage: wack-precheck.sh <app.exe> [<package.msix>]}"
MSIX="${2:-}"
fails=0
ok() { echo "PASS  $*"; }
bad() { echo "FAIL  $*"; fails=$((fails + 1)); }

OBJDUMP=$(command -v x86_64-w64-mingw32-objdump || command -v objdump)
[[ -f "$EXE" ]] || { echo "no such exe: $EXE" >&2; exit 2; }
hdr="$("$OBJDUMP" -p "$EXE" 2>/dev/null)"
grep -q 'pei-x86-64\|Magic.*020b' <<<"$hdr" && ok "PE32+ x64" || bad "not a PE32+ x64 image"
for flag in DYNAMIC_BASE HIGH_ENTROPY_VA NX_COMPAT; do
  grep -q "$flag" <<<"$hdr" && ok "DllCharacteristics $flag" || bad "DllCharacteristics lacks $flag"
done
sub=$(grep -m1 '^Subsystem' <<<"$hdr")
case "$sub" in
  *"Windows GUI"*) ok "GUI subsystem" ;;
  *) echo "NOTE  $sub (a dev package keeps the console; production must be GUI)" ;;
esac
"$OBJDUMP" -h "$EXE" | grep -q '\.rsrc' && ok "resource section" || bad "no .rsrc (manifest / version / icon missing — build.rs win_resources)"
strs="$(strings "$EXE")"
grep -q 'requestedExecutionLevel level="asInvoker"' <<<"$strs" && ok "manifest: asInvoker" || bad "manifest: no requestedExecutionLevel asInvoker"
grep -q '8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a' <<<"$strs" && ok "manifest: supportedOS Windows 10/11" || bad "manifest: no Windows 10 supportedOS"
grep -q 'PerMonitorV2' <<<"$strs" && ok "manifest: PerMonitorV2" || bad "manifest: no dpiAwareness"
wstrs="$(strings -el "$EXE")"
for key in CompanyName ProductName FileVersion ProductVersion OriginalFilename; do
  grep -qx "$key" <<<"$wstrs" && ok "version info: $key" || bad "version info: no $key"
done

if [[ -n "$MSIX" ]]; then
  [[ -f "$MSIX" ]] || { bad "no such msix: $MSIX"; MSIX=""; }
fi
if [[ -n "$MSIX" ]]; then
  tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
  if ! unzip -q -o "$MSIX" -d "$tmp" 2>/dev/null; then
    bad "msix is not a readable zip"
  else
    m="$tmp/AppxManifest.xml"
    [[ -f "$m" ]] && ok "AppxManifest.xml" || bad "no AppxManifest.xml"
    [[ -f "$tmp/AppxBlockMap.xml" ]] && ok "AppxBlockMap.xml" || bad "no AppxBlockMap.xml (pack with makemsix)"
    if [[ -f "$m" ]]; then
      grep -q '<Identity[^>]*Name="[^"]\+"[^>]*Publisher="CN=' "$m" && ok "Identity Name + Publisher CN" || bad "Identity without Name / Publisher CN"
      grep -q 'Name="runFullTrust"' "$m" && ok "runFullTrust" || bad "no runFullTrust capability"
      grep -q 'TargetDeviceFamily Name="Windows.Desktop" MinVersion="10\.' "$m" && ok "Windows.Desktop MinVersion 10.x" || bad "no Windows.Desktop TargetDeviceFamily"
      exe_in=$(grep -o 'Executable="[^"]*"' "$m" | head -1 | cut -d'"' -f2 | tr '\\' '/')
      [[ -n "$exe_in" && -f "$tmp/$exe_in" ]] && ok "Executable $exe_in present" || bad "Executable '$exe_in' missing from package"
      while read -r logo; do
        logo=${logo//\\\\/\/}; logo=${logo//\\/\/}
        [[ -z "$logo" ]] && continue
        base="${logo%.*}"; ext="${logo##*.}"
        if compgen -G "$tmp/$base*.$ext" >/dev/null; then ok "logo $logo"; else bad "logo $logo missing"; fi
      done < <(grep -oE '(Logo|Square44x44Logo|Square150x150Logo|Wide310x150Logo)="[^"]+"|<Logo>[^<]+' "$m" | sed -E 's/^[A-Za-z0-9]+="//; s/"$//; s/^<Logo>//' | sort -u)
      if find "$tmp" -iname '*.pdb' | grep -q .; then bad "debug .pdb files in the package"; else ok "no .pdb files"; fi
    fi
  fi
fi

echo
if (( fails )); then echo "WACK pre-check: $fails FAIL"; exit 1; fi
echo "WACK pre-check: all PASS (now run scripts/windows/run-wack.ps1 on Windows)"
