# Tabiyat visual baselines

Named PNG shots (same basename as smoke `current_step` / shot files). Seed after a
green peers/full cert run:

```bash
./scripts/product-cert.sh --tier=visual tabiyat
cp local/product-cert/tabiyat/<stamp>/ui/peers/tabiyat/000_unlocked.png apps/tabiyat/cert/baselines/
cp local/product-cert/tabiyat/<stamp>/ui/peers/tabiyat/000a_rail_collapsed.png apps/tabiyat/cert/baselines/
cp local/product-cert/tabiyat/<stamp>/ui/peers/tabiyat/000b_rail_expanded.png apps/tabiyat/cert/baselines/
```

ImageMagick AE compare runs when PNGs are present under this directory.
