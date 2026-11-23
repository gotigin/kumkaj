#!/usr/bin/env bash
# Isolated local E2E — do not mix modes:
#   Auth  = Firebase Auth Emulator (not production Auth)
#   API   = local khaynjo-cloud + Paddle *sandbox* keys
#   Pay   = Paddle sandbox checkout
#   Hook  = ngrok → local API (PADDLE_NTF_LOCAL_ID only; api.khaynjo.com untouched)
#
# Chrome Paddle dashboard: MEARAJ (Gotigin).
# Usage: ./scripts/local-paddle-e2e.sh
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"
WS="$(cd ../.. && pwd)"
PORT_API="${PORT_API:-18080}"
PORT_SITE="${PORT_SITE:-8012}"
AUTH_EMU_HOST="${AUTH_EMU_HOST:-127.0.0.1}"
AUTH_EMU_PORT="${AUTH_EMU_PORT:-9099}"
SITE_PUBLIC="$WS/sites/khaynjo/public"
CLOUDFLARED="${CLOUDFLARED:-$(command -v cloudflared || true)}"
[[ -x /tmp/cloudflared ]] && CLOUDFLARED=/tmp/cloudflared
[[ -n "${CLOUDFLARED:-}" ]] || { echo "need cloudflared"; exit 1; }
command -v ngrok >/dev/null || { echo "need ngrok"; exit 1; }
command -v npx >/dev/null || { echo "need npx/firebase-tools"; exit 1; }

set -a
# shellcheck disable=SC1091
source .env.development
set +a
: "${PADDLE_API_KEY:?}"
: "${PADDLE_WEBHOOK_SECRET:?}"
: "${PADDLE_NTF_LOCAL_ID:?set PADDLE_NTF_LOCAL_ID in .env.development}"

export HOST=127.0.0.1
export PORT="$PORT_API"
# Product UI path uses Auth Emulator tokens — not DEV_AUTH_BYPASS.
unset DEV_AUTH_BYPASS || true
export FIREBASE_AUTH_EMULATOR_HOST="${AUTH_EMU_HOST}:${AUTH_EMU_PORT}"
export BILLING_MODE=sandbox
export BILLING_ENABLED=1
export BILLING_DEFAULT_PROVIDER=paddle
export BILLING_ALLOW_MOCK=0
export BILLING_SUCCESS_URL="http://127.0.0.1:${PORT_SITE}/account?billing=success"
export BILLING_CANCEL_URL="http://127.0.0.1:${PORT_SITE}/account?billing=cancel"
export PUBLIC_API_BASE="http://127.0.0.1:${PORT_API}"
export RUST_LOG="${RUST_LOG:-info}"

echo "== Auth Emulator :${AUTH_EMU_PORT} =="
pkill -f "firebase.*emulators" 2>/dev/null || true
(
  cd "$WS/sites/khaynjo"
  npx -y firebase-tools@latest emulators:start --only auth --project khaynjo-2b2a2 \
    >/tmp/firebase-auth-emu.log 2>&1
) &
echo $! > /tmp/firebase-auth-emu.pid
for _ in $(seq 1 60); do
  if curl -sf "http://${AUTH_EMU_HOST}:${AUTH_EMU_PORT}/" >/dev/null 2>&1 \
    || curl -sf "http://${AUTH_EMU_HOST}:${AUTH_EMU_PORT}/identitytoolkit.googleapis.com/" >/dev/null 2>&1; then
    break
  fi
  # Emulator answers on /emulator/v1/... once up
  if rg -q "Auth Emulator|emulators.*ready|127.0.0.1:${AUTH_EMU_PORT}" /tmp/firebase-auth-emu.log 2>/dev/null; then
    break
  fi
  sleep 0.5
done
rg -n "Auth Emulator|Error|ready" /tmp/firebase-auth-emu.log | tail -8 || true

echo "== Cloud API :${PORT_API} =="
pkill -f 'target/debug/khaynjo-cloud' 2>/dev/null || true
cargo build -q
./target/debug/khaynjo-cloud > /tmp/khaynjo-cloud-local.log 2>&1 &
echo $! > /tmp/khaynjo-cloud-local.pid
for _ in $(seq 1 40); do curl -sf "http://127.0.0.1:${PORT_API}/healthz" >/dev/null && break; sleep 0.25; done
curl -sf "http://127.0.0.1:${PORT_API}/healthz" >/dev/null

echo "== ngrok → API (local notification only) =="
pkill -f 'ngrok http' 2>/dev/null || true
ngrok http "$PORT_API" --log=stdout >/tmp/ngrok-khaynjo.log 2>&1 &
echo $! > /tmp/ngrok-khaynjo.pid
API_TUN=""
for _ in $(seq 1 40); do
  API_TUN=$(curl -sS http://127.0.0.1:4040/api/tunnels 2>/dev/null | python3 -c 'import sys,json
d=json.load(sys.stdin)
for t in d.get("tunnels",[]):
  u=t.get("public_url","")
  if u.startswith("https:"):
    print(u); break' 2>/dev/null || true)
  [[ -n "$API_TUN" ]] && break
  sleep 0.25
done
[[ -n "$API_TUN" ]] || { echo "ngrok failed"; exit 1; }
echo "$API_TUN" > /tmp/ngrok-khaynjo.url

PATCH=$(curl -sS -X PATCH \
  -H "Authorization: Bearer ${PADDLE_API_KEY}" -H "Paddle-Version: 1" -H "Content-Type: application/json" \
  -d "{\"destination\":\"${API_TUN}/v1/billing/webhooks/paddle\"}" \
  "https://sandbox-api.paddle.com/notification-settings/${PADDLE_NTF_LOCAL_ID}")
echo "$PATCH" > /tmp/paddle_ntf_local_patch.json
NEW_SECRET=$(python3 -c 'import json; print((json.load(open("/tmp/paddle_ntf_local_patch.json")).get("data") or {}).get("endpoint_secret_key") or "")')
if [[ -n "$NEW_SECRET" ]]; then
  sed -i "s|^PADDLE_WEBHOOK_SECRET=.*|PADDLE_WEBHOOK_SECRET=${NEW_SECRET}|" .env.development
  export PADDLE_WEBHOOK_SECRET="$NEW_SECRET"
  kill "$(cat /tmp/khaynjo-cloud-local.pid)" 2>/dev/null || true
  sleep 0.4
  set -a; source .env.development; set +a
  unset DEV_AUTH_BYPASS || true
  export FIREBASE_AUTH_EMULATOR_HOST="${AUTH_EMU_HOST}:${AUTH_EMU_PORT}"
  export HOST=127.0.0.1 PORT="$PORT_API" BILLING_MODE=sandbox BILLING_ENABLED=1
  export BILLING_DEFAULT_PROVIDER=paddle BILLING_ALLOW_MOCK=0
  export BILLING_SUCCESS_URL="http://127.0.0.1:${PORT_SITE}/account?billing=success"
  export BILLING_CANCEL_URL="http://127.0.0.1:${PORT_SITE}/account?billing=cancel"
  export PUBLIC_API_BASE="http://127.0.0.1:${PORT_API}"
  ./target/debug/khaynjo-cloud > /tmp/khaynjo-cloud-local.log 2>&1 &
  echo $! > /tmp/khaynjo-cloud-local.pid
  for _ in $(seq 1 40); do curl -sf "http://127.0.0.1:${PORT_API}/healthz" >/dev/null && break; sleep 0.25; done
fi

echo "== Site :${PORT_SITE} (real public/ tree — not a static dump) =="
# Prefer staged preview/deploy tree (Makepad home + account + shop).
if [[ ! -f "$SITE_PUBLIC/index.html" ]]; then
  echo "missing $SITE_PUBLIC/index.html — run: ./sites/khaynjo/scripts/preview-web.sh" >&2
  exit 1
fi
(cd "$WS/sites/khaynjo" && npm run build:firebase >/tmp/khaynjo-build-firebase.log 2>&1 && npm run build:account >/tmp/khaynjo-build-account.log 2>&1 && npm run build:shop >/tmp/khaynjo-build-shop.log 2>&1)
rsync -a "$WS/sites/khaynjo/static/js/" "$SITE_PUBLIC/js/"
rsync -a "$WS/sites/khaynjo/static/account/" "$SITE_PUBLIC/account/"
rsync -a "$WS/sites/khaynjo/static/shop/" "$SITE_PUBLIC/shop/"
python3 - <<PY
from pathlib import Path
import re
p = Path("$SITE_PUBLIC/account/index.html")
t = re.sub(r"\s*<script>\s*window\.__KUMKAJ_CLOUD_API__[\s\S]*?</script>\s*", "\n", p.read_text())
inj = '''  <script>
    window.__KUMKAJ_CLOUD_API__ = "http://127.0.0.1:${PORT_API}";
    window.__KUMKAJ_AUTH_EMULATOR__ = "http://${AUTH_EMU_HOST}:${AUTH_EMU_PORT}";
  </script>
'''
t = t.replace(
  '<script src="/js/account.js" defer></script>',
  inj + '  <script src="/js/account.js" defer></script>',
)
p.write_text(t)
PY
pkill -f "http.server ${PORT_SITE}" 2>/dev/null || true
python3 -m http.server "$PORT_SITE" --bind 127.0.0.1 --directory "$SITE_PUBLIC" >/tmp/khaynjo-site-8012.log 2>&1 &
echo $! > /tmp/khaynjo-site-8012.pid

echo "== HTTPS return URL (Paddle requires https default_url) =="
pkill -f 'cloudflared tunnel --url' 2>/dev/null || true
"$CLOUDFLARED" tunnel --url "http://127.0.0.1:${PORT_SITE}" --no-autoupdate >/tmp/cloudflared-site.log 2>&1 &
echo $! > /tmp/cloudflared-site.pid
SITE_TUN=""
for _ in $(seq 1 50); do
  SITE_TUN=$(grep -oE 'https://[a-z0-9-]+\.trycloudflare\.com' /tmp/cloudflared-site.log | head -1 || true)
  [[ -n "$SITE_TUN" ]] && break
  sleep 0.3
done
[[ -n "$SITE_TUN" ]] || { echo "site tunnel failed"; exit 1; }
echo "$SITE_TUN" > /tmp/cloudflared-site.url
curl -sS -X PATCH \
  -H "Authorization: Bearer ${PADDLE_API_KEY}" -H "Paddle-Version: 1" -H "Content-Type: application/json" \
  -d "{\"default_url\":\"${SITE_TUN}/account\"}" \
  https://sandbox-api.paddle.com/settings/checkout-url >/tmp/paddle_checkout_url.json

cat <<EOF

Modes (this script):
  Auth     Firebase Auth Emulator  http://${AUTH_EMU_HOST}:${AUTH_EMU_PORT}  (UI http://127.0.0.1:4000)
  API      local                   http://127.0.0.1:${PORT_API}
  Site     real public/ tree       http://127.0.0.1:${PORT_SITE}/  (shop → account checkout)
  Paddle   sandbox                 webhook → ${API_TUN}/v1/billing/webhooks/paddle
  Return   ${SITE_TUN}/account

Product path: /shop → add Steady → Checkout → /account?checkout=steady → Paddle.
Deployed sandbox webhook api.khaynjo.com is separate (PADDLE_NTF_DEPLOYED_ID) — not modified here.
EOF
