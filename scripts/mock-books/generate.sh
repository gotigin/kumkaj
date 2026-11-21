#!/usr/bin/env bash
# Generate Gotigin-internal demo books (plain .sqlite3 backups) for all four apps.
# Output: local/mock-books/<app>/<app>.sqlite3
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${MOCK_BOOKS_OUT:-$ROOT/local/mock-books}"
mkdir -p "$OUT"/{khaynjo,moklanjo,kirano,tabiyat,.work}

export MOCK_BOOKS_OUT="$OUT"
export KUMKAJ_PRINT_SINK="${KUMKAJ_PRINT_SINK:-off}"

apps=(khaynjo moklanjo kirano tabiyat)
if [[ $# -gt 0 ]]; then
  apps=("$@")
fi

for app in "${apps[@]}"; do
  echo "=== generating $app ==="
  cargo test -p "kumkaj-$app" --test mock_books -- --ignored --nocapture
done

echo
echo "=== verify ==="
"$ROOT/scripts/mock-books/verify.sh" "${apps[@]}"
echo "Done. See $OUT/README.md"
