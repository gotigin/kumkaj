#!/usr/bin/env bash
# Fetch OFL Noto script fonts + Font Awesome into the app's
# resources/fonts/ for WorldFonts fallback (language names + UI
# symbols). App: KUMKAJ_APP_DIR (default apps/courier). Safe to re-run.
# Uses curl + jq (no Python).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${KUMKAJ_APP_DIR:-$ROOT/apps/courier}/resources/fonts"
mkdir -p "$OUT"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required (Arch: pacman -S jq)" >&2
  exit 1
fi

download_family() {
  local folder="$1" outfile="$2"
  if [[ -s "$outfile" ]]; then
    echo "have $(basename "$outfile")"
    return 0
  fi
  local api="https://api.github.com/repos/google/fonts/contents/ofl/${folder}"
  local json
  json="$(curl -fsSL -A 'kumkaj-font-fetch' "$api")"
  local url name
  # Prefer a Regular .ttf without bracketed variable-font names.
  url="$(echo "$json" | jq -r '
    [.[] | select(.download_url != null)
         | select(.name | test("\\.(ttf|otf)$"))
         | select(.name | test("Regular"))
         | select(.name | contains("[") | not)]
    | .[0].download_url // empty
  ')"
  name="$(echo "$json" | jq -r '
    [.[] | select(.download_url != null)
         | select(.name | test("\\.(ttf|otf)$"))
         | select(.name | test("Regular"))
         | select(.name | contains("[") | not)]
    | .[0].name // empty
  ')"
  if [[ -z "$url" ]]; then
    url="$(echo "$json" | jq -r '
      [.[] | select(.download_url != null) | select(.name | endswith(".ttf"))]
      | .[0].download_url // empty
    ')"
    name="$(echo "$json" | jq -r '
      [.[] | select(.download_url != null) | select(.name | endswith(".ttf"))]
      | .[0].name // empty
    ')"
  fi
  if [[ -z "$url" ]]; then
    url="$(echo "$json" | jq -r '[.[] | select(.download_url != null)] | .[0].download_url // empty')"
    name="$(echo "$json" | jq -r '[.[] | select(.download_url != null)] | .[0].name // empty')"
  fi
  if [[ -z "$url" ]]; then
    echo "no font in ofl/${folder}" >&2
    return 1
  fi
  echo "get ${name} -> ${outfile}"
  curl -fsSL -A 'kumkaj-font-fetch' -o "$outfile" "$url"
  # TrueType (\0\1\0\0) / OpenType (OTTO) / collection (ttcf)
  local sig
  sig="$(head -c 4 "$outfile")"
  if [[ "$sig" != $'\x00\x01\x00\x00' && "$sig" != "OTTO" && "$sig" != "ttcf" ]]; then
    rm -f "$outfile"
    echo "not a font: ${folder}" >&2
    return 1
  fi
  echo "ok $(wc -c < "$outfile") bytes"
}

# Core script coverage beyond the fonts already vendored with the app.
download_family notosanshebrew "$OUT/NotoSansHebrew-Regular.ttf"
download_family notosanssinhala "$OUT/NotoSansSinhala-Regular.ttf"
download_family notosansmyanmar "$OUT/NotoSansMyanmar-Regular.ttf"
download_family notosanskhmer "$OUT/NotoSansKhmer-Regular.ttf"
download_family notosanslao "$OUT/NotoSansLao-Regular.ttf"
download_family notoseriftibetan "$OUT/NotoSansTibetan-Regular.ttf"
download_family notosansethiopic "$OUT/NotoSansEthiopic-Regular.ttf"
download_family notosansthaana "$OUT/NotoSansThaana-Regular.ttf"
download_family notosansgeorgian "$OUT/NotoSansGeorgian-Regular.ttf"
download_family notosansarmenian "$OUT/NotoSansArmenian-Regular.ttf"

WIDGETS="$ROOT/vendor/makepad/widgets/resources"
# Single font tree under the app resources/fonts — never duplicate into
# a second package folder (makepad_widgets/resources).
for f in fa-solid-900.ttf LXGWWenKaiRegular.ttf LXGWWenKaiBold.ttf NotoColorEmoji.ttf; do
  if [[ -s "$OUT/$f" ]]; then
    echo "have $f"
    continue
  fi
  if [[ -f "$WIDGETS/$f" ]]; then
    cp -f "$WIDGETS/$f" "$OUT/$f"
    echo "have $f (from vendor/makepad)"
  else
    echo "warn: run ./scripts/setup-makepad.sh first for $f" >&2
  fi
done

echo "fonts ready in $OUT"
ls -lh "$OUT"
