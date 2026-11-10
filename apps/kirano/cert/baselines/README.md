# Kirano visual baselines

Named PNG shots (same basename as smoke `current_step` / shot files). Seed after a
green peers/full cert run:

```bash
./scripts/product-cert.sh --tier=visual kirano
cp local/product-cert/kirano/<stamp>/ui/peers/kirano/000_unlocked.png apps/kirano/cert/baselines/
cp local/product-cert/kirano/<stamp>/ui/peers/kirano/000a_rail_collapsed.png apps/kirano/cert/baselines/
cp local/product-cert/kirano/<stamp>/ui/peers/kirano/000b_rail_expanded.png apps/kirano/cert/baselines/
```

ImageMagick AE compare runs when PNGs are present under this directory.
