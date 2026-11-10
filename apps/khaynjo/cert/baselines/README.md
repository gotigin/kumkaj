# Khaynjo visual baselines

Drop named PNG shots here (same basename as smoke `current_step` / shot files).
`./scripts/product-cert.sh --tier=visual khaynjo` compares ImageMagick AE against
shots under `local/product-cert/…`. Until files exist, the visual tier still
runs multi-size Xvfb peers and skips golden compare.
