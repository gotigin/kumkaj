# Menu templates (`assets/templates/`)

Optional **public/demo** menu JSON folders for local experiments. They are
**not** DeviceSetup chips.

Production DeviceSetup has two optional templates only:

| | |
|---|---|
| **Public** | **Khaynjo Full Template** — CDN `khaynjo-items` shelf (`public/items/`) |
| **Private** | Client packs from `catalogue/private-src/<id>/` (Rajdhani today) published with a 12-digit code |

Private packs are **not** under this folder and are **not** discovered by
`menu_templates::all()`. Publish:

```bash
./apps/khaynjo/scripts/publish-catalogue.sh
./apps/khaynjo/scripts/publish-catalogue.sh --private-rajdhani --code=<12 digits>
```

See [`docs/CATALOGUE_URLS.md`](../docs/CATALOGUE_URLS.md).

## Layout (demo folders only)

```text
assets/templates/<id>/
  template.json
  images/*.webp   # optional
```

## Private authoring (Rajdhani)

```text
catalogue/private-src/rajdhani/
  template.json
  shop_defaults.json   # staff, tax chrome, pay methods, KOT — not hardcoded in the app
```

`shop_defaults.json` is required for private publish. The app applies that
file from the downloaded pack; it does not ship Zahir/Asif/Nawaz or the
Rajdhani menu inside the install.
