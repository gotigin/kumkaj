#!/usr/bin/env bash
# Production Khaynjo Windows package -> dist/khaynjo-windows/
exec "$(cd "$(dirname "$0")" && pwd)/windows-dev.sh" prod
