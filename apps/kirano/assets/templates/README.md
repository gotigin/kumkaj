# Kirano starter catalogue

Same shape as Khaynjo's dish shelf, for a neighbourhood grocery.

- **Photo shelf** `assets/items/<aisle>/<item>.webp` — WebP only. The
  builder downloads a Wikimedia Commons JPEG or PNG thumbnail and
  re-encodes it (max edge 800, quality 80). The app never stores the
  original PNG/JPEG in the shelf.
- **Named template** `neighbourhood/template.json` — English name, Hindi
  alias, example rupee price, unit, `shelf:` photo. Prices are starters,
  not a legal MRP.
- A new shop stays empty until the owner imports it (Items → Import
  complete template, or Import this menu).

Rebuild:

```bash
python3 apps/kirano/scripts/build-shelf.py
```

Attribution for each fetched photo is `neighbourhood/SOURCES.jsonl`
(Commons file page). Re-run skips a WebP that is already valid.
