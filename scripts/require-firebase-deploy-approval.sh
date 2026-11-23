#!/usr/bin/env bash
# Shared gate: stop *AI agents* from burning Firebase Hosting quota.
#
# Owner at a normal terminal may run:
#   ./sites/<site>/scripts/deploy-firebase.sh
# That is enough. This script must NOT block the owner.
#
# Agents (CURSOR_AGENT=1 or GOTIGIN_AI_AGENT=1) need explicit approval:
#   --i-approve-firebase-deploy
#   GOTIGIN_APPROVE_FIREBASE_DEPLOY=1
# --dry-run never needs approval (no Hosting upload).
#
# Source from a site's deploy-firebase.sh AFTER parsing args:
#   source "$ROOT/scripts/require-firebase-deploy-approval.sh"
#   require_firebase_deploy_approval "$DRY" "$@"

require_firebase_deploy_approval() {
  local dry="${1:-0}"
  shift || true
  if [[ "$dry" == "1" ]]; then
    return 0
  fi

  # Not an agent session → owner (or CI they control). Do not block.
  if [[ "${CURSOR_AGENT:-}" != "1" && "${GOTIGIN_AI_AGENT:-}" != "1" ]]; then
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
error: Firebase Hosting deploy blocked for AI agent (quota / cost).

This gate is for Cursor/agents only — not for the owner.
Owner: run the same script from your terminal (no extra flag).

Agents must:
  1. Finish local verify (preview-web / run-web / desktop) first
  2. Get explicit owner approval for THIS deploy
  3. Re-run with ONE of:
       ./sites/<site>/scripts/deploy-firebase.sh --i-approve-firebase-deploy
       GOTIGIN_APPROVE_FIREBASE_DEPLOY=1 ./sites/<site>/scripts/deploy-firebase.sh

--dry-run is always allowed (builds/stages only; no Hosting upload).
See docs/WEBSITES.md § Firebase deploy policy.
EOF
    exit 1
  fi
}
