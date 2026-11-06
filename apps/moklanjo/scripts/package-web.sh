#!/usr/bin/env bash
# Build moklanjo.com (Zola) into dist/web/moklanjo.
# Brand site: KUMKAJ_SITE_DIR=web/kumkaj ./scripts/build-site.sh
# Khaynjo:    KUMKAJ_SITE_DIR=apps/khaynjo/web ./scripts/build-site.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
export KUMKAJ_SITE_DIR="$ROOT/apps/moklanjo/web"
exec "$ROOT/scripts/build-site.sh"
