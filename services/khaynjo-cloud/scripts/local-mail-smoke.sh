#!/usr/bin/env bash
# Local sandbox: mock pay → activate → SES product confirmation (Khaynjo).
# Uses Docker Postgres only. Never run against AWS with DEV_AUTH_BYPASS.
# Recipient must be SES-verified while the account is still in SES sandbox.
set -euo pipefail
cd "$(dirname "$0")/.."

TO="${MAIL_TEST_TO:-mearaj@gotigin.com}"
export DATABASE_URL="${DATABASE_URL:-postgres://khaynjo:khaynjo_dev@127.0.0.1:5433/khaynjo_cloud}"
export FIREBASE_PROJECT_ID='khaynjo-2b2a2'
export HOST=127.0.0.1
export PORT="${SMOKE_PORT:-18081}"
export DEV_AUTH_BYPASS=1
export BILLING_ENABLED=1
export BILLING_MODE=sandbox
export BILLING_DEFAULT_PROVIDER=mock
export BILLING_ALLOW_MOCK=1
export BILLING_SUCCESS_URL='http://127.0.0.1:8012/account?billing=success'
export BILLING_CANCEL_URL='http://127.0.0.1:8012/account?billing=cancel'
export PUBLIC_API_BASE="http://127.0.0.1:${PORT}"
export MAIL_ENABLED=1
export MAIL_FROM="${MAIL_FROM:-noreply@khaynjo.com}"
export MAIL_PRODUCT_NAME=Khaynjo
export MAIL_BRAND_NAME=Kumkaj
export MAIL_COMPANY_NAME='GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED'
export MAIL_ACCOUNT_URL='https://khaynjo.com/account'
export AWS_PROFILE="${AWS_PROFILE:-gotigin}"
export AWS_SDK_LOAD_CONFIG=1

# aws login / SSO session → env keys the Rust SDK understands (never commit keys)
if command -v aws >/dev/null; then
  if CREDS=""; then
    eval ""
  fi
fi
export AWS_DEFAULT_REGION="${AWS_DEFAULT_REGION:-ap-south-1}"
export RUST_LOG=info
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/khaynjo-cloud-target}"

cargo build -q
pkill -f "target/.*/khaynjo-cloud$" 2>/dev/null || true
sleep 0.3
"$CARGO_TARGET_DIR/debug/khaynjo-cloud" > /tmp/khaynjo-cloud-mail-smoke.log 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null || true' EXIT

for _ in $(seq 1 60); do
  if curl -sf "http://127.0.0.1:${PORT}/healthz" | grep -q '"ok":true'; then
    break
  fi
  if ! kill -0 "$PID" 2>/dev/null; then
    echo "API exited early:"; cat /tmp/khaynjo-cloud-mail-smoke.log; exit 1
  fi
  sleep 0.25
done

UID_SMOKE="mail-smoke-$(date +%s)-$$"
AUTH=(-H "Authorization: Bearer dev:${UID_SMOKE}:${TO}" -H 'Content-Type: application/json')

curl -sf "${AUTH[@]}" "http://127.0.0.1:${PORT}/v1/me" >/dev/null
CID=$(curl -sf "${AUTH[@]}" -d '{"plan_id":"steady","provider":"mock"}' \
  "http://127.0.0.1:${PORT}/v1/billing/checkout" | python3 -c 'import sys,json; print(json.load(sys.stdin)["checkout_id"])')
DONE=$(curl -sf "${AUTH[@]}" -d "{\"checkout_id\":\"${CID}\"}" \
  "http://127.0.0.1:${PORT}/v1/billing/sandbox/complete")
echo "$DONE" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d.get("ok") or d.get("plan_id")=="steady" or d.get("plan_id"); print("activated", d)'

echo "--- mail log lines ---"
grep -E 'plan confirmation|SES|MAIL_|sent plan' /tmp/khaynjo-cloud-mail-smoke.log || true
echo "OK local mail smoke finished (check ${TO} if domain verified)"
