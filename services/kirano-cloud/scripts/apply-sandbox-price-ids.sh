#!/usr/bin/env bash
# Apply Stripe / Paddle sandbox Price ids onto the local plans table.
# Chrome profile for dashboards: MEARAJ (Gotigin).
#
# Usage (after creating Prices in Stripe Test + Paddle sandbox):
#   export DATABASE_URL='postgres://kirano:kirano_dev@127.0.0.1:5435/kirano_cloud'
#   export STRIPE_PRICE_STEADY=price_...
#   export STRIPE_PRICE_SWIFT=price_...
#   export STRIPE_PRICE_PRIORITY=price_...
#   export PADDLE_PRICE_STEADY=pri_...
#   export PADDLE_PRICE_SWIFT=pri_...
#   export PADDLE_PRICE_PRIORITY=pri_...
#   ./scripts/apply-sandbox-price-ids.sh
set -euo pipefail
cd "$(dirname "$0")/.."

: "${DATABASE_URL:?DATABASE_URL required}"

psql "$DATABASE_URL" <<SQL
UPDATE plans SET stripe_price_id = NULLIF('${STRIPE_PRICE_STEADY:-}', ''),
                 paddle_price_id = NULLIF('${PADDLE_PRICE_STEADY:-}', '')
 WHERE plan_id = 'steady';
UPDATE plans SET stripe_price_id = NULLIF('${STRIPE_PRICE_SWIFT:-}', ''),
                 paddle_price_id = NULLIF('${PADDLE_PRICE_SWIFT:-}', '')
 WHERE plan_id = 'swift';
UPDATE plans SET stripe_price_id = NULLIF('${STRIPE_PRICE_PRIORITY:-}', ''),
                 paddle_price_id = NULLIF('${PADDLE_PRICE_PRIORITY:-}', '')
 WHERE plan_id = 'priority';

SELECT plan_id, price_inr_paise, stripe_price_id, paddle_price_id
FROM plans
WHERE plan_id IN ('steady','swift','priority')
ORDER BY sort_order;
SQL

echo "OK — sandbox price ids applied (empty env vars clear the column)."
