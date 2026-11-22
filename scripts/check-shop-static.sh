#!/usr/bin/env bash
# Static checks for /shop + /account checkout wiring (no Firebase deploy).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
for product in khaynjo moklanjo kirano tabiyat; do
  for f in \
    "sites/$product/static/shop/index.html" \
    "sites/$product/static/js/shop.js" \
    "sites/$product/static/css/shop.css" \
    "sites/$product/static/account/index.html" \
    "sites/$product/static/js/account.js"
  do
    if [[ ! -f "$ROOT/$f" ]]; then
      echo "MISSING $f"
      fail=1
    fi
  done
  if ! grep -q 'Cloud shop' "$ROOT/sites/$product/static/account/index.html"; then
    echo "FAIL $product account missing Cloud shop link"
    fail=1
  fi
  if ! grep -q 'card-cloud-plans' "$ROOT/sites/$product/static/account/index.html"; then
    echo "FAIL $product account missing plan card"
    fail=1
  fi
  if ! grep -q 'checkout' "$ROOT/sites/$product/static/js/account.js"; then
    echo "FAIL $product account.js missing checkout"
    fail=1
  fi
  if ! grep -q 'to_shop\|SHOP_PATH\|/shop' "$ROOT/sites/$product/src/main.rs"; then
    echo "FAIL $product site main missing shop open"
    fail=1
  fi
done
node "$ROOT/sites/_shared/shop/catalog.test.mjs"
if [[ "$fail" -ne 0 ]]; then
  echo "check-shop-static FAILED"
  exit 1
fi
echo "check-shop-static OK"
