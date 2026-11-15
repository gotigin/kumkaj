# Product Certification Kit

```bash
./scripts/product-cert.sh --tier=fast [app…]
./scripts/product-cert.sh --tier=full [app…]
./scripts/product-cert.sh --tier=store [app…]
./scripts/linux-app-cert.sh   # alias
```

| Tier | Covers |
|------|--------|
| **fast** | lock-guards, UI translations, never-print, cargo lib/bins, user-journeys, Play crash mitigations in source |
| **ui** | Xvfb peers (launch + sidebar collapse/expand + chrome) + full action smoke |
| **visual** | optional multi-size (`CERT_MULTI_SIZE=1`); golden PNGs in `apps/<app>/cert/baselines/` when present |
| **resilience** | soak peer thrash; fail on panic / `[E]` |
| **xcomp** | Play `version_code` rules; package scripts present |
| **linux** | Short Xvfb launch of `dist/<app>-linux` binary (`CERT_BUILD_LINUX=1` to package) |
| **wack** | Production `x86_64-pc-windows-gnu` build + `scripts/windows/wack-precheck.sh` (PE flags, manifest, version info, icon, newest `dist/<Label>-*.msix`); the real WACK run is `scripts/windows/run-wack.ps1` on Windows |
| **android** | `adb` install+launch when a device/emulator is present (`CERT_REQUIRE_ADB=1` to require) |
| **store** | AAB+symbols (Khaynjo), fonts, MSIX identity docs, UI tier must be green; runs linux+android |

Reports: `local/product-cert/`. Docs: `docs/DEV.md` § Product Certification Kit.

**Not automated:** Play Console / Partner Center human review, full OCR of every language, GPU farm, full Apple binary pipeline.
