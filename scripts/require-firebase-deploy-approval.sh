#!/usr/bin/env bash
# Shared gate: Firebase Hosting deploys burn quota and cost money.
# Agents and humans must NOT upload until the owner explicitly approves
# AFTER local preview/testing is done.
#
# Source from a site's deploy-firebase.sh AFTER parsing args:
#   # shellcheck source=...
#   source "$ROOT/scripts/require-firebase-deploy-approval.sh"
#   require_firebase_deploy_approval "$DRY" "$@"
#
# Allowed only when ONE of:
#   --i-approve-firebase-deploy
#   GOTIGIN_APPROVE_FIREBASE_DEPLOY=1
# --dry-run never needs approval (no Hosting upload).

require_firebase_deploy_approval() {
  local dry="${1:-0}"
  shift || true
  if [[ "$dry" == "1" ]]; then
    return 0
  fi
  local approved=0
  if [[ "${GOTIGIN_APPROVE_FIREBASE_DEPLOY:-}" == "1" ]]; then
    approved=1
  fi
  for arg in "$@"; do
    case "$arg" in
      --i-approve-firebase-deploy) approved=1 ;;
    esac
  done
  if [[ "$approved" != "1" ]]; then
    cat >&2 <<'EOF'
error: Firebase Hosting deploy blocked (quota / cost).

Do NOT deploy websites on every small change. Required first:
  1. Local verify (preview-web / run-web / desktop) until the change is complete
  2. Explicit owner approval for THIS deploy

Then re-run with ONE of:
  ./sites/<site>/scripts/deploy-firebase.sh --i-approve-firebase-deploy
  GOTIGIN_APPROVE_FIREBASE_DEPLOY=1 ./sites/<site>/scripts/deploy-firebase.sh

--dry-run is always allowed (builds/stages only; no Hosting upload).
See docs/WEBSITES.md § Firebase deploy policy.
EOF
    exit 1
  fi
}
