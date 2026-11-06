# Websites

| Domain | Source | Build |
|---|---|---|
| **kumkaj.com** | [`kumkaj/`](kumkaj/) | `KUMKAJ_SITE_DIR=web/kumkaj ./scripts/build-site.sh` → `dist/web/kumkaj/` |
| **moklanjo.com** | [`../apps/moklanjo/web/`](../apps/moklanjo/web/) | `./apps/moklanjo/scripts/package-web.sh` → `dist/web/moklanjo/` |
| **khaynjo.com** | [`../apps/khaynjo/web/`](../apps/khaynjo/web/) | `KUMKAJ_SITE_DIR=apps/khaynjo/web ./scripts/build-site.sh` → `dist/web/khaynjo/` |

Plan: [`docs/WEBSITES.md`](../docs/WEBSITES.md). Requires [Zola](https://www.getzola.org/) (`zola` on `PATH`).
