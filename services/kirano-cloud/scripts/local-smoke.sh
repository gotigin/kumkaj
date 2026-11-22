#!/usr/bin/env bash
# Local smoke — Docker Postgres only. Do not point at AWS RDS.
# Never set DEV_AUTH_BYPASS=1 on AWS.
# Billing: mock sandbox path (no Stripe/Paddle keys required).
set -euo pipefail
cd "$(dirname "$0")/.."
export DATABASE_URL='postgres://kirano:kirano_dev@127.0.0.1:5435/kirano_cloud'
export FIREBASE_PROJECT_ID='kirano-shop'
export HOST=127.0.0.1
# Avoid clashing with whatever else may already own :8080 on a busy host.
export PORT="${PORT:-18082}"
export DEV_AUTH_BYPASS=1
export BILLING_ENABLED=1
export BILLING_MODE=sandbox
export BILLING_DEFAULT_PROVIDER=mock
export BILLING_SUCCESS_URL='http://127.0.0.1:8013/account?billing=success'
export BILLING_CANCEL_URL='http://127.0.0.1:8013/account?billing=cancel'
export PUBLIC_API_BASE="http://127.0.0.1:${PORT}"
export RUST_LOG=info

cargo build -q
pkill -f 'target/debug/kirano-cloud' 2>/dev/null || true
sleep 0.3
./target/debug/kirano-cloud > /tmp/kirano-cloud-smoke.log 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null || true' EXIT
ready=0
for _ in $(seq 1 40); do
  if ! kill -0 "$PID" 2>/dev/null; then
    echo "kirano-cloud exited early — log:"
    cat /tmp/kirano-cloud-smoke.log
    exit 1
  fi
  if curl -sf "http://127.0.0.1:${PORT}/healthz" | grep -q '"ok":true'; then
    ready=1
    break
  fi
  sleep 0.25
done
if [[ "$ready" != 1 ]]; then
  echo "healthz never became ready on :${PORT}"
  cat /tmp/kirano-cloud-smoke.log
  exit 1
fi

fail=0
req() {
  local name="$1" want="$2"
  shift 2
  local tmp code body
  tmp=$(mktemp)
  code=$(curl -sS "$@" -o "$tmp" -w '%{http_code}' || true)
  body=$(cat "$tmp")
  rm -f "$tmp"
  if [[ "$code" != "$want" ]]; then
    echo "FAIL $name status=$code want=$want body=$body"
    fail=1
  else
    echo "OK   $name ($code)"
  fi
}

# Fresh uid each run so a prior Steady activation does not break the Free check.
SMOKE_UID="smoke-user-$(date +%s)-$$"
AUTH=(-H "Authorization: Bearer dev:${SMOKE_UID}:smoke@example.com" -H 'Content-Type: application/json')

echo "=== kirano-cloud local smoke (incl. billing mock) ==="
req healthz 200 "http://127.0.0.1:${PORT}/healthz"
req me_no_auth 401 "http://127.0.0.1:${PORT}/v1/me"
req plans_no_auth 401 "http://127.0.0.1:${PORT}/v1/plans"
req me_bad_token 401 -H 'Authorization: Bearer not-a-jwt' "http://127.0.0.1:${PORT}/v1/me"

ME=$(curl -sf "${AUTH[@]}" "http://127.0.0.1:${PORT}/v1/me")
echo "$ME" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["product_code"]=="kirano"; assert d["username"]; assert d["account_id"]; print("OK   me_ensure", d["username"], d["account_id"][:8]+"...")'

ME2=$(curl -sf "${AUTH[@]}" "http://127.0.0.1:${PORT}/v1/me")
AID1=$(echo "$ME" | python3 -c 'import sys,json; print(json.load(sys.stdin)["account_id"])')
AID2=$(echo "$ME2" | python3 -c 'import sys,json; print(json.load(sys.stdin)["account_id"])')
[[ "$AID1" == "$AID2" ]]
echo "OK   me_idempotent ${AID1:0:8}..."

PLANS=$(curl -sf "${AUTH[@]}" "http://127.0.0.1:${PORT}/v1/plans")
echo "$PLANS" | python3 -c 'import sys,json; p=json.load(sys.stdin); ids=[x["plan_id"] for x in p]; assert ids==["free","steady","swift","priority","custom"], ids; assert any(x["plan_id"]=="steady" and x["checkout_ready"] for x in p); print("OK   plans", ids)'

SUB=$(curl -sf "${AUTH[@]}" "http://127.0.0.1:${PORT}/v1/me/subscription")
echo "$SUB" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["plan_id"]=="free"; assert d["status"]=="active"; print("OK   subscription", d["plan_id"], d["title"])'

CFG=$(curl -sf "${AUTH[@]}" "http://127.0.0.1:${PORT}/v1/billing/config")
echo "$CFG" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["enabled"] is True; assert d["mode"]=="sandbox"; assert "mock" in d["providers"]; print("OK   billing_config", d["default_provider"], d["providers"])'

CO=$(curl -sf "${AUTH[@]}" -d '{"plan_id":"steady","provider":"mock"}' "http://127.0.0.1:${PORT}/v1/billing/checkout")
echo "$CO" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["provider"]=="mock"; assert d["plan_id"]=="steady"; assert d["checkout_id"]; open("/tmp/kirano-checkout-id","w").write(d["checkout_id"]); print("OK   checkout_mock", d["checkout_id"][:8]+"...")'
CID=$(cat /tmp/kirano-checkout-id)

DONE=$(curl -sf "${AUTH[@]}" -d "{\"checkout_id\":\"${CID}\"}" "http://127.0.0.1:${PORT}/v1/billing/sandbox/complete")
echo "$DONE" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["ok"] is True; assert d["plan_id"]=="steady"; assert d["ends_at"]; print("OK   sandbox_complete", d["plan_id"], "ends", d["ends_at"][:10])'

SUB2=$(curl -sf "${AUTH[@]}" "http://127.0.0.1:${PORT}/v1/me/subscription")
echo "$SUB2" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["plan_id"]=="steady"; assert d["includes_live_cloud"] is True; print("OK   subscription_paid", d["plan_id"], d["title"])'

# Idempotent complete
req sandbox_complete_replay 200 "${AUTH[@]}" -d "{\"checkout_id\":\"${CID}\"}" "http://127.0.0.1:${PORT}/v1/billing/sandbox/complete"

if [[ $fail -ne 0 ]]; then echo "SMOKE FAILED"; exit 1; fi
echo "SMOKE PASSED for kirano-cloud (billing mock sandbox)"
