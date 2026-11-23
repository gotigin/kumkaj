#!/usr/bin/env bash
# Apply Paddle sandbox Price ids onto the local plans table.
# Dashboard: Chrome profile **MEARAJ (Gotigin)** — Paddle sandbox.
#
# Usage:
#   export DATABASE_URL='postgres://tabiyat:tabiyat_dev@127.0.0.1:5436/tabiyat_cloud'
#   export PADDLE_PRICE_STEADY=pri_...
#   export PADDLE_PRICE_SWIFT=pri_...
#   export PADDLE_PRICE_PRIORITY=pri_...
#   ./scripts/apply-sandbox-price-ids.sh
set -euo pipefail
cd "$(dirname "$0")/.."

: "${DATABASE_URL:?DATABASE_URL required}"

psql "$DATABASE_URL" <<SQL
UPDATE plans SET paddle_price_id = NULLIF('${PADDLE_PRICE_STEADY:-}', '')
 WHERE plan_id = 'steady';
UPDATE plans SET paddle_price_id = NULLIF('${PADDLE_PRICE_SWIFT:-}', '')
 WHERE plan_id = 'swift';
UPDATE plans SET paddle_price_id = NULLIF('${PADDLE_PRICE_PRIORITY:-}', '')
 WHERE plan_id = 'priority';

SELECT plan_id, price_inr_paise, paddle_price_id
FROM plans
WHERE plan_id IN ('steady','swift','priority')
ORDER BY sort_order;
SQL

echo "OK — Paddle sandbox price ids applied."
