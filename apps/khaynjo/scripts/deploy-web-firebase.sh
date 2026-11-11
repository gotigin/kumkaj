#!/usr/bin/env bash
# Compatibility wrapper — khaynjo.com is the Makepad site under sites/khaynjo.
exec "$(cd "$(dirname "$0")/../../.." && pwd)/sites/khaynjo/scripts/deploy-firebase.sh" "$@"
