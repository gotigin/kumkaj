# Menu templates (`assets/templates/`)

> **Why templates exist:** they **configure** a shop’s face (menu,
> `shop_defaults`, which chrome is on/off) for the **same** Khaynjo app —
> not a fork per client. Agents: read
> [`docs/CLIENT_TEMPLATES_AND_SETUP.md`](../../../docs/CLIENT_TEMPLATES_AND_SETUP.md)
> before changing template import, onboarding, or “this client needs…” UI.

Optional **demo** menu JSON folders for local experiments. They are
**not** DeviceSetup chips and **not** offered on Menu (the Café / Khaynjo
starter chips were removed 2026-11-13). Menu → Item templates offers the
public catalogue list, at menu-items level: the **Khaynjo Items Template**
and any authored pack on that list (Rajdhani, once published) — items only.

Production DeviceSetup offers the same public list. Skip leaves an empty shop.

| | |
|---|---|
| **Khaynjo Full Template** | CDN `khaynjo-items` shelf (`public/items/`) |
| **Authored packs** | From `catalogue/private-src/<folder>/` (Rajdhani today), published as `public/templates/<id>/` |

Those packs are **not** under this folder and are **not** discovered by
`menu_templates::all()`. Publish:

```bash
./apps/khaynjo/scripts/publish-catalogue.sh
./apps/khaynjo/scripts/publish-catalogue.sh --public-rajdhani
./apps/khaynjo/scripts/publish-catalogue.sh --public=<folder>
```

See [`docs/CATALOGUE_URLS.md`](../docs/CATALOGUE_URLS.md).

## Layout (demo folders only)

```text
assets/templates/<id>/
  template.json
  images/*.webp   # optional
```

## Authoring (Rajdhani)

```text
catalogue/private-src/rajdhani/
  template.json
  shop_defaults.json   # staff, tax chrome, pay methods, KOT — not hardcoded in the app
```

`shop_defaults.json` is required when publishing an authored pack. The app
applies that file from the downloaded pack on setup; a Menu import adds items
only. The install does not ship Zahir/Asif/Nawaz or the Rajdhani menu.
