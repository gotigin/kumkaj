#!/usr/bin/env bash
# Run billing mock smoke for every product cloud (local Docker Postgres).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
for product in khaynjo moklanjo kirano tabiyat; do
  echo "########## ${product}-cloud ##########"
  if ! (cd "$ROOT/services/${product}-cloud" && ./scripts/local-smoke.sh); then
    echo "FAIL ${product}-cloud"
    fail=1
  fi
done
if [[ "$fail" -ne 0 ]]; then
  echo "SMOKE-ALL FAILED"
  exit 1
fi
echo "SMOKE-ALL PASSED (4 clouds, mock-only local harness)"
