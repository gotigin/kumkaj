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

The gate fails a shot whose AE (pixels that differ) is above
`CERT_VISUAL_MAX_AE` (default 50000 of 1280×900), or when compare prints no
number (size mismatch). The random shop id in the header and clock times differ per run (~120–250 AE);
anything bigger is a real change — look at `visual/diff-<name>.png` and the
old/new pair before copying a new shot in.

**2026-11-21:** baselines re-pinned after transparent-corner app icons (`879ddd4f`); peers from product-cert `20261121-152216`.
