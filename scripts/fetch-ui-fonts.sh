#!/usr/bin/env bash
# Fetch OFL Noto script fonts + Font Awesome into apps/kumkaj/resources/fonts/
# for WorldFonts fallback (language names + UI symbols). Safe to re-run.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/apps/kumkaj/resources/fonts"
mkdir -p "$OUT"

download_family() {
  local folder="$1" outfile="$2"
  if [[ -s "$outfile" ]]; then
    echo "have $(basename "$outfile")"
    return 0
  fi
  python3 - "$folder" "$outfile" <<'PY'
import json, sys, urllib.request
folder, outfile = sys.argv[1], sys.argv[2]
req = urllib.request.Request(
    f"https://api.github.com/repos/google/fonts/contents/ofl/{folder}",
    headers={"User-Agent": "kumkaj-font-fetch"},
)
with urllib.request.urlopen(req, timeout=60) as r:
    entries = json.load(r)
cands = [
    (x["name"], x["download_url"])
    for x in entries
    if x.get("download_url") and (x["name"].endswith(".ttf") or x["name"].endswith(".otf"))
]
pick = next((c for c in cands if "Regular" in c[0] and "[" not in c[0]), None)
if pick is None:
    pick = next((c for c in cands if c[0].endswith(".ttf")), None)
if pick is None and cands:
    pick = cands[0]
if pick is None:
    raise SystemExit(f"no font in ofl/{folder}")
print(f"get {pick[0]} -> {outfile}")
data = urllib.request.urlopen(
    urllib.request.Request(pick[1], headers={"User-Agent": "kumkaj-font-fetch"}),
    timeout=180,
).read()
if data[:4] not in (b"\x00\x01\x00\x00", b"OTTO", b"ttcf"):
    raise SystemExit(f"not a font: {folder}")
open(outfile, "wb").write(data)
print(f"ok {len(data)} bytes")
PY
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

FA_SRC="$ROOT/vendor/makepad/widgets/resources/fa-solid-900.ttf"
if [[ -f "$FA_SRC" ]]; then
  cp -f "$FA_SRC" "$OUT/fa-solid-900.ttf"
  echo "have fa-solid-900.ttf (from vendor/makepad)"
else
  echo "warn: run ./scripts/setup-makepad.sh first for fa-solid-900.ttf" >&2
fi

echo "fonts ready in $OUT"
ls -lh "$OUT"
