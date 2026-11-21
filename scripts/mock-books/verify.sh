#!/usr/bin/env bash
# Integrity + math spot-checks on generated mock books.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${MOCK_BOOKS_OUT:-$ROOT/local/mock-books}"

apps=(khaynjo moklanjo kirano tabiyat)
if [[ $# -gt 0 ]]; then
  apps=("$@")
fi

fail=0
for app in "${apps[@]}"; do
  db="$OUT/$app/$app.sqlite3"
  echo "--- $app ---"
  if [[ ! -f "$db" ]]; then
    echo "MISSING $db"
    fail=1
    continue
  fi
  integrity="$(sqlite3 "$db" 'PRAGMA integrity_check;')"
  echo "integrity: $integrity"
  if [[ "$integrity" != "ok" ]]; then
    fail=1
  fi

  sqlite3 "$db" <<'SQL'
.mode column
.headers on
SELECT 'sales' AS t, COUNT(*) AS n FROM sales WHERE status='completed'
UNION ALL SELECT 'sale_lines', COUNT(*) FROM sale_lines
UNION ALL SELECT 'customers', COUNT(*) FROM customers
UNION ALL SELECT 'products', COUNT(*) FROM products
UNION ALL SELECT 'staff', COUNT(*) FROM staff
UNION ALL SELECT 'expenses', COUNT(*) FROM expenses
UNION ALL SELECT 'sale_days', COUNT(DISTINCT substr(created_at,1,10)) FROM sales WHERE status='completed';
SQL

  # Piece/menu apps: unit × qty_milli/1000 ≈ line_total (1 paise slack).
  # Moklanjo courier lines are slab-priced (first kg + extra/kg); the charged
  # amount is the slab total, not unit_price × weight — skip that check.
  if [[ "$app" == "moklanjo" ]]; then
    echo "line unit×qty check: skipped (weight/slab bookings)"
  elif sqlite3 "$db" "SELECT discount_paise FROM sales LIMIT 1;" &>/dev/null; then
    bad="$(sqlite3 "$db" "
SELECT COUNT(*) FROM sale_lines
WHERE ABS(
  line_total_paise
  - (unit_price_paise * CASE WHEN qty_milli > 0 THEN qty_milli ELSE qty * 1000 END / 1000)
) > 1
  AND sale_id IN (
    SELECT id FROM sales WHERE COALESCE(discount_paise, 0) = 0 AND status = 'completed'
  );
")"
    echo "line mismatches (>1 paise): $bad"
    if [[ "$bad" -gt 0 ]]; then
      fail=1
    fi
  else
    echo "line unit×qty check: skipped (no discount_paise column)"
  fi

  # Sale totals = sum of lines (+ rounding).
  tot_bad="$(sqlite3 "$db" "
SELECT COUNT(*) FROM sales s
WHERE s.status='completed'
  AND ABS(
    s.lines_total_paise
    - COALESCE((SELECT SUM(l.line_total_paise) FROM sale_lines l WHERE l.sale_id = s.id), 0)
  ) > 0;
")"
  echo "sales with lines_total != sum(lines): $tot_bad"
  if [[ "$tot_bad" -gt 0 ]]; then
    fail=1
  fi

  app_meta="$(sqlite3 "$db" "SELECT value FROM meta WHERE key='app';" 2>/dev/null || true)"
  echo "meta.app=$app_meta"
done

if [[ "$fail" -ne 0 ]]; then
  echo "VERIFY FAILED"
  exit 1
fi
echo "VERIFY OK"
