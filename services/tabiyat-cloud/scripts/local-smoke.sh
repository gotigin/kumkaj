#!/usr/bin/env bash
# Local smoke — Docker Postgres only. Do not point at AWS RDS.
# Never set DEV_AUTH_BYPASS=1 on AWS.
set -euo pipefail
cd "$(dirname "$0")/.."
export DATABASE_URL='postgres://tabiyat:tabiyat_dev@127.0.0.1:5436/tabiyat_cloud'
export FIREBASE_PROJECT_ID='tabiyat-shop'
export HOST=127.0.0.1
export PORT=8083
export DEV_AUTH_BYPASS=1
export RUST_LOG=info

cargo build -q
pkill -f 'target/debug/tabiyat-cloud' 2>/dev/null || true
sleep 0.3
./target/debug/tabiyat-cloud > /tmp/tabiyat-cloud-smoke.log 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do
  if curl -sf "http://127.0.0.1:${PORT}/healthz" >/dev/null; then break; fi
  sleep 0.25
done

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

echo "=== tabiyat-cloud local smoke ==="
req healthz 200 "http://127.0.0.1:${PORT}/healthz"
req me_no_auth 401 "http://127.0.0.1:${PORT}/v1/me"
req plans_no_auth 401 "http://127.0.0.1:${PORT}/v1/plans"
req me_bad_token 401 -H 'Authorization: Bearer not-a-jwt' "http://127.0.0.1:${PORT}/v1/me"

ME=$(curl -sf -H 'Authorization: Bearer dev:smoke-user-1:smoke@example.com' "http://127.0.0.1:${PORT}/v1/me")
echo "$ME" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["product_code"]=="tabiyat"; assert d["username"]; assert d["account_id"]; print("OK   me_ensure", d["username"], d["account_id"][:8]+"...")'

ME2=$(curl -sf -H 'Authorization: Bearer dev:smoke-user-1:smoke@example.com' "http://127.0.0.1:${PORT}/v1/me")
AID1=$(echo "$ME" | python3 -c 'import sys,json; print(json.load(sys.stdin)["account_id"])')
AID2=$(echo "$ME2" | python3 -c 'import sys,json; print(json.load(sys.stdin)["account_id"])')
[[ "$AID1" == "$AID2" ]]
echo "OK   me_idempotent ${AID1:0:8}..."

PLANS=$(curl -sf -H 'Authorization: Bearer dev:smoke-user-1:smoke@example.com' "http://127.0.0.1:${PORT}/v1/plans")
echo "$PLANS" | python3 -c 'import sys,json; p=json.load(sys.stdin); ids=[x["plan_id"] for x in p]; assert ids==["free","steady","swift","priority","custom"], ids; print("OK   plans", ids)'

SUB=$(curl -sf -H 'Authorization: Bearer dev:smoke-user-1:smoke@example.com' "http://127.0.0.1:${PORT}/v1/me/subscription")
echo "$SUB" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["plan_id"]=="free"; assert d["status"]=="active"; print("OK   subscription", d["plan_id"], d["title"])'

if [[ $fail -ne 0 ]]; then echo "SMOKE FAILED"; exit 1; fi
echo "SMOKE PASSED for tabiyat-cloud"
