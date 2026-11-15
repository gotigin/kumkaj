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

The gate fails a shot whose AE (pixels that differ) is above
`CERT_VISUAL_MAX_AE` (default 50000 of 1280×900), or when compare prints no
number (size mismatch). The random shop id in the header and clock times differ per run (~120–250 AE);
anything bigger is a real change — look at `visual/diff-<name>.png` and the
old/new pair before copying a new shot in.

**2026-11-15:** `000_unlocked`, `000a_rail_collapsed`, `000b_rail_expanded` and `012_peer_Settings` re-pinned — the nav default order is now the names in dictionary order (rail alphabetical, unlock lands on the first screen by name) and Settings is one list of nav items (docs/DEV.md § Nav order).
