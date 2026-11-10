#!/usr/bin/env bash
# Alias for Product Certification Kit (historical name).
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/product-cert.sh" "$@"
