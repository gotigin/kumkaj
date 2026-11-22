#!/usr/bin/env bash
# Curl shareable URLs and require full OG + Twitter + canonical + 1200×630 JPEG.
# Homes and static shells must expose tags in the first HTML (no wasm).
#
#   ./scripts/check-link-previews.sh            # live sites
#   ./scripts/check-link-previews.sh --local    # local preview ports (see RESULTS)
#
# Exit 1 if any required tag or image dimension fails.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
UA='facebookexternalhit/1.1 (+http://www.facebook.com/externalhit_uatext.php)'
MODE=live
if [[ "${1:-}" == "--local" ]]; then MODE=local; fi

REQ_PROP=(og:title og:description og:url og:image og:image:width og:image:height og:image:alt og:type og:site_name)
REQ_NAME=(twitter:card twitter:title twitter:description twitter:image twitter:image:alt)

# site|path|base-url
paths_live() {
  cat <<'EOF'
gotigin.com|/|https://gotigin.com
gotigin.com|/privacy|https://gotigin.com
gotigin.com|/terms|https://gotigin.com
gotigin.com|/about|https://gotigin.com
gotigin.com|/support|https://gotigin.com
gotigin.com|/help|https://gotigin.com
gotigin.com|/cookies|https://gotigin.com
gotigin.com|/security|https://gotigin.com
gotigin.com|/data-deletion|https://gotigin.com
wadhu.com|/|https://wadhu.com
wadhu.com|/privacy|https://wadhu.com
wadhu.com|/terms|https://wadhu.com
wadhu.com|/about|https://wadhu.com
wadhu.com|/support|https://wadhu.com
wadhu.com|/data-deletion|https://wadhu.com
wadhu.com|/account|https://wadhu.com
kumkaj.com|/|https://kumkaj.com
kumkaj.com|/privacy|https://kumkaj.com
kumkaj.com|/terms|https://kumkaj.com
kumkaj.com|/shop|https://kumkaj.com
kumkaj.com|/cancellation|https://kumkaj.com
khaynjo.com|/|https://khaynjo.com
khaynjo.com|/privacy|https://khaynjo.com
khaynjo.com|/terms|https://khaynjo.com
khaynjo.com|/account|https://khaynjo.com
khaynjo.com|/shop|https://khaynjo.com
khaynjo.com|/cancellation|https://khaynjo.com
moklanjo.com|/|https://moklanjo.com
moklanjo.com|/privacy|https://moklanjo.com
moklanjo.com|/terms|https://moklanjo.com
moklanjo.com|/account|https://moklanjo.com
moklanjo.com|/shop|https://moklanjo.com
moklanjo.com|/cancellation|https://moklanjo.com
kirano.shop|/|https://kirano.shop
kirano.shop|/privacy|https://kirano.shop
kirano.shop|/terms|https://kirano.shop
kirano.shop|/account|https://kirano.shop
kirano.shop|/shop|https://kirano.shop
kirano.shop|/cancellation|https://kirano.shop
tabiyat.shop|/|https://tabiyat.shop
tabiyat.shop|/privacy|https://tabiyat.shop
tabiyat.shop|/terms|https://tabiyat.shop
tabiyat.shop|/account|https://tabiyat.shop
tabiyat.shop|/shop|https://tabiyat.shop
tabiyat.shop|/cancellation|https://tabiyat.shop
ghalbol.com|/|https://ghalbol.com
ghalbol.com|/privacy|https://ghalbol.com
ghalbol.com|/terms|https://ghalbol.com
ghalbol.com|/support|https://ghalbol.com
ghalbol.com|/about|https://ghalbol.com
ghalbol.com|/data-deletion|https://ghalbol.com
EOF
}

# Local preview ports match local/web-quality-20261122/RESULTS.md
paths_local() {
  cat <<'EOF'
kumkaj|/|http://127.0.0.1:8010
moklanjo|/|http://127.0.0.1:8011
khaynjo|/|http://127.0.0.1:8012
kirano|/|http://127.0.0.1:8013
tabiyat|/|http://127.0.0.1:8014
wadhu|/|http://127.0.0.1:8020
gotigin|/|http://127.0.0.1:8030
ghalbol|/|http://127.0.0.1:8080
EOF
}

image_dims() {
  local url="$1" tmp
  tmp="$(mktemp --suffix=.img)"
  if ! curl -fsSL -A "$UA" --max-time 20 "$url" -o "$tmp"; then
    rm -f "$tmp"
    echo "fetch-fail"
    return
  fi
  if command -v magick >/dev/null 2>&1; then
    magick identify -format '%wx%h' "$tmp" 2>/dev/null || echo "identify-fail"
  elif command -v identify >/dev/null 2>&1; then
    identify -format '%wx%h' "$tmp" 2>/dev/null || echo "identify-fail"
  else
    # JPEG SOF0 rough fallback
    python3 - "$tmp" <<'PY' 2>/dev/null || echo "no-identify"
import struct,sys
p=sys.argv[1]
d=open(p,'rb').read()
i=2
while i<len(d)-8:
  if d[i]!=0xFF: break
  m=d[i+1]
  if m in (0xC0,0xC1,0xC2):
    h,w=struct.unpack('>HH', d[i+5:i+9]); print(f'{w}x{h}'); break
  if m in (0xD8,0xD9): i+=2; continue
  ln=struct.unpack('>H', d[i+2:i+4])[0]; i+=2+ln
else:
  print('unknown')
PY
  fi
  rm -f "$tmp"
}

fail=0
ok=0
rows=()
if [[ "$MODE" == local ]]; then
  mapfile -t rows < <(paths_local)
else
  mapfile -t rows < <(paths_live)
fi

echo "check-link-previews ($MODE)"
echo

for row in "${rows[@]}"; do
  IFS='|' read -r site path base <<<"$row"
  url="${base}${path}"
  html="$(curl -sL -A "$UA" --max-time 20 -w '\n__HTTP__:%{http_code}' "$url" || true)"
  code="$(sed -n 's/^__HTTP__://p' <<<"$html" | tail -1)"
  body="$(sed '/^__HTTP__:/d' <<<"$html")"
  miss=()
  if [[ "$code" != "200" ]]; then
    echo "FAIL $url  HTTP $code"
    fail=$((fail + 1))
    continue
  fi
  grep -qi 'name="description"' <<<"$body" || miss+=(description)
  grep -qi 'rel="canonical"' <<<"$body" || miss+=(canonical)
  for t in "${REQ_PROP[@]}"; do
    grep -qiE "property=\"$t\"" <<<"$body" || miss+=("$t")
  done
  for t in "${REQ_NAME[@]}"; do
    grep -qiE "name=\"$t\"" <<<"$body" || miss+=("$t")
  done
  if grep -qiE 'property="og:image" content="(/|\./)' <<<"$body"; then
    miss+=(relative-og-image)
  fi
  img="$(grep -oiE 'property="og:image"[[:space:]]+content="[^"]+"' <<<"$body" | head -1 | sed 's/.*content="//;s/"$//' || true)"
  dims="n/a"
  if [[ -n "$img" ]]; then
    # Local pages may point at production image URLs — still check that URL.
    dims="$(image_dims "$img")"
    if [[ "$dims" != "1200x630" ]]; then
      miss+=("image-dims:$dims")
    fi
  else
    miss+=(no-og-image-url)
  fi
  if ((${#miss[@]})); then
    echo "FAIL $url  missing: ${miss[*]}  img=$img"
    fail=$((fail + 1))
  else
    echo "OK   $url  ($dims)"
    ok=$((ok + 1))
  fi
done

echo
echo "ok=$ok fail=$fail"
if (( fail > 0 )); then
  exit 1
fi
