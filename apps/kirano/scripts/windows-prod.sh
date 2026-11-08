#!/usr/bin/env bash
# Production Kirano Windows package -> dist/kirano-windows/
exec "$(cd "$(dirname "$0")" && pwd)/windows-dev.sh" prod
