#!/usr/bin/env bash
# Production Tabiyat Windows package -> dist/tabiyat-windows/
exec "$(cd "$(dirname "$0")" && pwd)/windows-dev.sh" prod
