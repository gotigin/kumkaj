# Moklanjo visual baselines

Named PNG shots (same basename as smoke `current_step` / shot files). Seed after a
green peers/full cert run:

```bash
./scripts/product-cert.sh --tier=visual moklanjo
cp local/product-cert/moklanjo/<stamp>/ui/peers/moklanjo/000_unlocked.png apps/moklanjo/cert/baselines/
cp local/product-cert/moklanjo/<stamp>/ui/peers/moklanjo/000a_rail_collapsed.png apps/moklanjo/cert/baselines/
cp local/product-cert/moklanjo/<stamp>/ui/peers/moklanjo/000b_rail_expanded.png apps/moklanjo/cert/baselines/
```

ImageMagick AE compare runs when PNGs are present under this directory.
