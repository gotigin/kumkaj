#!/usr/bin/env python3
"""Build Tabiyat's starter chemist shelf.

Same pipeline as Khaynjo / Kirano: Wikimedia Commons JPEG or PNG thumbnail
in, WebP out (max edge 800). Generic names only. See template blurb.
"""

from __future__ import annotations

import importlib.util
import io
import json
import time
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SHELF = ROOT / "assets" / "items"
TEMPLATE_DIR = ROOT / "assets" / "templates" / "chemist-counter"
UA = "TabiyatCatalogue/1.0 (https://tabiyat.shop; starter shelf; attribution in SOURCES.jsonl)"
COMMONS = "https://commons.wikimedia.org/w/api.php"

_spec = importlib.util.spec_from_file_location("tabiyat_items", Path(__file__).with_name("items.py"))
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)
ITEMS = []
for folder, rows in _mod.ROWS.items():
    for slug, name, hindi, price, unit, query in rows:
        ITEMS.append((folder, slug, name, hindi, price, unit, query))

def commons_hits(query: str) -> list[dict]:
    params = {
        "action": "query",
        "format": "json",
        "generator": "search",
        "gsrsearch": query,
        "gsrnamespace": "6",
        "gsrlimit": "8",
        "prop": "imageinfo",
        "iiprop": "url|mime",
        "iiurlwidth": "800",
    }
    url = COMMONS + "?" + urllib.parse.urlencode(params)
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    with urllib.request.urlopen(req, timeout=25) as resp:
        data = json.load(resp)
    pages = (data.get("query") or {}).get("pages") or {}
    hits = []
    for page in pages.values():
        info = (page.get("imageinfo") or [None])[0] or {}
        mime = (info.get("mime") or "").lower()
        thumb = info.get("thumburl") or ""
        if mime not in {"image/jpeg", "image/png", "image/webp"}:
            continue
        if not thumb.startswith("http"):
            continue
        hits.append({
            "title": page.get("title") or "",
            "thumb": thumb,
            "page": "https://commons.wikimedia.org/wiki/" + urllib.parse.quote((page.get("title") or "").replace(" ", "_")),
            "mime": mime,
        })
    return hits


def to_webp(raw: bytes, dest: Path) -> None:
    im = Image.open(io.BytesIO(raw))
    im = im.convert("RGB")
    im.thumbnail((800, 800))
    dest.parent.mkdir(parents=True, exist_ok=True)
    im.save(dest, "WEBP", quality=80, method=4)


def fetch_one(item: tuple, folder_query: str) -> dict:
    folder, slug, name, hindi, price, unit, query = item
    dest = SHELF / folder / f"{slug}.webp"
    if dest.is_file() and dest.stat().st_size > 8000 and dest.read_bytes()[:4] == b"RIFF":
        return {"ok": True, "cached": True, "item": item, "source": None}
    last_err = "no hit"
    plain = query
    for word in (" tablet", " syrup", " capsule", " sachet", " drops", " cream", " gel", " pack"):
        plain = plain.replace(word, "")
    plain = " ".join(plain.split())
    for q in (query, plain, "medicine " + plain, folder_query, name):
        try:
            hits = commons_hits(q)
        except Exception as e:
            last_err = str(e)
            time.sleep(1.5)
            continue
        for hit in hits:
            try:
                req = urllib.request.Request(hit["thumb"], headers={"User-Agent": UA})
                with urllib.request.urlopen(req, timeout=25) as resp:
                    raw = resp.read()
                if len(raw) < 4000:
                    continue
                title = (hit.get("title") or "").lower()
                if any(b in title for b in ("formula", "structural", "diagram", "skeletal", "acid crp")):
                    continue
                to_webp(raw, dest)
                if dest.stat().st_size < 8000:
                    dest.unlink(missing_ok=True)
                    continue
                return {"ok": True, "cached": False, "item": item, "source": hit}
            except Exception as e:
                last_err = str(e)
                continue
        time.sleep(0.2)
    return {"ok": False, "item": item, "error": last_err}


def main() -> int:
    print(f"{len(ITEMS)} items", flush=True)
    if len(ITEMS) < 500:
        raise SystemExit(f"catalogue has {len(ITEMS)}, need at least 500")
    slugs = [i[1] for i in ITEMS]
    if len(slugs) != len(set(slugs)):
        raise SystemExit("duplicate slug")
    by_folder: dict[str, str] = {}
    for folder, *_rest, query in ((i[0], i[6]) for i in ITEMS):
        by_folder.setdefault(folder, query)

    results = []
    with ThreadPoolExecutor(max_workers=8) as pool:
        futs = [pool.submit(fetch_one, item, by_folder[item[0]]) for item in ITEMS]
        for n, fut in enumerate(as_completed(futs), 1):
            res = fut.result()
            results.append(res)
            name = res["item"][2]
            if res["ok"]:
                mark = "cache" if res.get("cached") else "got"
                print(f"[{n}/{len(ITEMS)}] {mark} {name}", flush=True)
            else:
                print(f"[{n}/{len(ITEMS)}] MISS {name}: {res.get('error')}", flush=True)

    ok = [r for r in results if r["ok"]]
    print(f"photos {len(ok)}/{len(ITEMS)}", flush=True)

    categories: dict[str, list] = {}
    for item in ITEMS:
        categories.setdefault(item[0], []).append(item)

    def title(folder: str) -> str:
        return " ".join(w[:1].upper() + w[1:] for w in folder.split("-"))

    tmpl_cats = []
    for folder, rows in categories.items():
        items_json = []
        for _folder, slug, name, hindi, price, unit, _q in rows:
            image = f"shelf:{folder}/{slug}.webp"
            path = SHELF / folder / f"{slug}.webp"
            if not path.is_file():
                image = ""
            entry = {"name": name, "price": price, "unit": unit, "image": image}
            if hindi:
                entry["aliases"] = [hindi]
            items_json.append(entry)
        first_img = next((i["image"] for i in items_json if i["image"]), "")
        tmpl_cats.append({
            "name": title(folder),
            "image": first_img,
            "items": items_json,
        })
    template = {
        "id": "chemist-counter",
        "title": "Chemist counter",
        "blurb": "Starter chemist list: generic names, Hindi aliases, example prices, photos. Not a drug database and not medical advice. Schedule and Rx flags stay off until you set them. Prices are examples, not MRP.",
        "categories": tmpl_cats,
    }
    TEMPLATE_DIR.mkdir(parents=True, exist_ok=True)
    (TEMPLATE_DIR / "template.json").write_text(
        json.dumps(template, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    sources = TEMPLATE_DIR / "SOURCES.jsonl"
    with sources.open("w", encoding="utf-8") as fh:
        fh.write(json.dumps({
            "note": "Each photo is a Wikimedia Commons thumbnail re-encoded to WebP (max edge 800, quality 80). Attribute the File page.",
        }, ensure_ascii=False) + "\n")
        for res in ok:
            folder, slug, _name, _hindi, _price, _unit, query = res["item"]
            src = res.get("source") or {}
            row = {"file": f"{folder}/{slug}.webp", "search": query, "via": "Wikimedia Commons thumbnail, re-encoded to WebP"}
            if src.get("page"):
                row["commons"] = src.get("title")
                row["page"] = src["page"]
            fh.write(json.dumps(row, ensure_ascii=False) + "\n")
    missing = len(ITEMS) - len(ok)
    print(f"template written, missing photos {missing}", flush=True)
    return 0 if missing < 40 else 1


if __name__ == "__main__":
    raise SystemExit(main())
