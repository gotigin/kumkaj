# Khaynjo visual baselines

Named PNG shots (same basename as smoke `current_step` / shot files). Seeded from
green peers/full cert runs; ImageMagick AE compare runs when these PNGs exist.

```bash
./scripts/product-cert.sh --tier=visual khaynjo
# copy shots you want to pin:
cp local/product-cert/khaynjo/<stamp>/ui/peers/khaynjo/000_unlocked.png apps/khaynjo/cert/baselines/
```

Refresh after intentional chrome/layout changes; do not treat AE noise from
fonts/DPI as a product fail without checking the diff PNG.
