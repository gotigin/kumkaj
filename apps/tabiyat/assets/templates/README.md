# Tabiyat starter catalogue

Same shape as Khaynjo's dish shelf, for a chemist counter.

- **Photo shelf** `assets/items/<group>/<item>.webp` — WebP only. The
  builder downloads a Wikimedia Commons JPEG or PNG thumbnail and
  re-encodes it (max edge 800, quality 80).
- **Named template** `chemist-counter/template.json` — generic English
  name (no brand marks), Hindi alias, example price, unit, `shelf:` photo.
- Schedule H and Rx flags are **not** set by the template. The owner
  turns those on. This list is not a drug database and not medical advice.
- A new shop stays empty until the owner imports it.

Rebuild:

```bash
python3 apps/tabiyat/scripts/build-shelf.py
```

Attribution: `chemist-counter/SOURCES.jsonl`.
