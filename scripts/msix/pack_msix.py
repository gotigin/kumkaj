#!/usr/bin/env python3
"""Deprecated — MSIX packing is bash + Microsoft makemsix.

Use scripts/msix/pack_msix.sh (via scripts/build-msix.sh). The old Python
zip/BlockMap writer was rejected by Partner Center (APPX_E_INVALID_BLOCKMAP).
"""
from __future__ import annotations

import sys


def main() -> int:
    print(
        "pack_msix.py is retired. Use: scripts/msix/pack_msix.sh "
        "(or apps/<app>/scripts/msix.sh → build-msix.sh).",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
