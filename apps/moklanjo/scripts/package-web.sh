#!/usr/bin/env bash
# Copy Moklanjo's website (web/, https://kumkaj.com) to dist/web for
# static hosting (Play Store companion / marketing).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="$ROOT/dist/web"
mkdir -p "$OUT"
cp -f "$ROOT/web/index.html" "$OUT/index.html"
cp -f "$ROOT/web/play-store-listing.txt" "$OUT/play-store-listing.txt"
cat > "$OUT/README.txt" <<'EOF'
Kumkaj website
===============
Static site for product landing + download pointers.

Serve locally (any static server), e.g.:
  miniserve dist/web -p 8080
  # or:  busybox httpd -f -p 8080 -h dist/web

Deploy: upload dist/web/ to https://kumkaj.com (Cloudflare Pages, Netlify, S3, nginx).
EOF
echo "Web package: $OUT/index.html"
