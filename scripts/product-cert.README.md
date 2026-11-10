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
| **store** | AAB+symbols (Khaynjo), fonts, MSIX identity docs, UI tier must be green |

Reports: `local/product-cert/`. Docs: `docs/DEV.md` § Product Certification Kit.

**Not automated:** Play Console / Partner Center human review, full OCR of every language, real-device GPU farm, full Apple App Store binary pipeline.
