# Khaynjo visual baselines

Named PNG shots (same basename as smoke `current_step` / shot files). Seeded from a
green `product-cert` peers/full run on this machine (launch, rail collapse/expand,
Floor, Settings, open cart line).

```bash
./scripts/product-cert.sh --tier=visual khaynjo
```

Compares ImageMagick AE against shots under `local/product-cert/…`. Re-seed after
intentional chrome changes:

```bash
cp local/product-cert/khaynjo/<stamp>/ui/full/khaynjo/<step>.png apps/khaynjo/cert/baselines/
```
