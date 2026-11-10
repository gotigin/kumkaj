#!/usr/bin/env bash
# Bulk translation workflow entrypoint (bash only).
#
# This does **not** call Python. After scaffolding English stand-ins, fill
# packs by editing TSVs (or an agent), then run the bash check.
#
#   ./scripts/fill-translations-pipeline.sh --app khaynjo
#   ./scripts/fill-translations-pipeline.sh --app all
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="all"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) APP="${2:-}"; shift 2 ;;
    -h|--help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

cd "$ROOT"
echo "Scaffolding stand-ins for: $APP"
./scripts/scaffold-ui-translations.sh --app "$APP" --standin-english
echo
echo "Stand-ins written. Translate apps/<app>/translations/*.tsv, then:"
echo "  ./scripts/check-ui-translations.sh --app $APP"
echo
./scripts/check-ui-translations.sh --app "$APP" || {
  echo
  echo "Check still failing (expected until stand-ins are replaced with real translations)." >&2
  echo "Set KUMKAJ_ALLOW_TRANSLATION_GAPS=1 only while filling — never for a release." >&2
  exit 1
}
