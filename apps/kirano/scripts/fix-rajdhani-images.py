#!/usr/bin/env python3
"""Re-assign Rajdhani photos: protein-aware shelf match, then short Openverse fetch.

Same-family reuse is fine (several chicken tandoori variants). Cross-protein
reuse (chicken on fish/egg/mutton) is not. Fetches use a short timeout and
fall back to the best same-protein shelf dish when the net fails.
"""

from __future__ import annotations

import json
import re
import sys
import time
import urllib.parse
import urllib.request
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEMPLATE = ROOT / "assets/templates/rajdhani/template.json"
SHELF = ROOT / "assets/items"
IMAGES = ROOT / "assets/templates/rajdhani/images"
UA = "KhaynjoMenuBuilder/1.0 (local template tooling)"

PROTEIN = [
    ("prawn", ("prawn", "jinga", "shrimp", "chingri")),
    ("chicken", ("chicken", "murg", "murgh")),
    ("mutton", ("mutton", "gosht", "lamb", "raan", "kaleji", "bheja", "chaap", "choki")),
    ("egg", ("egg", "eggs", "inda", "omlet", "omelette", "anda")),
    ("fish", ("fish", "paplet", "pomfret", "mach", "ilish", "surmai")),
    ("veg", ("paneer", "veg", "aloo", "dal", "gobi", "mushroom")),
]

ALIASES = {
    "inda": "egg",
    "anda": "egg",
    "omlet": "omelette",
    "jinga": "prawn",
    "paplet": "fish pomfret",
    "raan": "mutton leg",
    "kaleji": "mutton liver",
    "bheja": "mutton brain",
    "choki": "mutton chop",
    "chaap": "mutton chop",
    "kali": "egg curry",
    "keema": "mince",
    "bhurji": "scrambled egg",
    "papad": "papadum",
    "pav": "bun",
    "roti": "roti bread",
    "lasaniya": "garlic egg",
    "surati": "egg curry",
    "rasawara": "rice gravy",
    "gotalo": "egg scramble",
    "hangama": "egg scramble",
    "moglai": "mughlai",
    "seena": "breast tandoori",
}


def norm(s: str) -> str:
    s = s.lower().replace("&", " and ")
    s = re.sub(r"[^a-z0-9]+", " ", s)
    return re.sub(r"\s+", " ", s).strip()


def expand(name: str) -> str:
    return " ".join(ALIASES.get(t, t) for t in norm(name).split())


def tokens(s: str) -> set[str]:
    stop = {
        "and", "with", "the", "red", "green", "golden", "white", "number",
        "piece", "spe", "special", "simple", "live", "fry", "gravy",
    }
    return {t for t in expand(s).split() if t and t not in stop and not t.isdigit()}


def protein_of(name: str) -> str | None:
    n = expand(name)
    for key, words in PROTEIN:
        if any(re.search(rf"\b{re.escape(w)}\b", n) for w in words):
            return key
    return None


def load_shelf() -> list[tuple[str, str, Path, str | None]]:
    out = []
    for folder in sorted(SHELF.iterdir()):
        if not folder.is_dir():
            continue
        for f in sorted(folder.glob("*.webp")):
            label = f.stem.replace("-", " ")
            out.append((folder.name, label, f, protein_of(label)))
    return out


def score(name: str, shelf_label: str, spro: str | None, want: str | None) -> float:
    nt, st = tokens(name), tokens(shelf_label)
    if not nt or not st:
        return -1.0
    inter = len(nt & st)
    if inter == 0:
        nn, sn = expand(name), expand(shelf_label)
        if sn in nn or nn in sn:
            inter = 1
        else:
            return -1.0
    if want and spro and want != spro:
        return -1.0
    j = inter / len(nt | st)
    bonus = 0.25 if want and spro == want else 0.0
    for t in nt & st:
        if len(t) >= 5:
            bonus += 0.05
    return j + bonus


def slug(s: str) -> str:
    s = re.sub(r"[^a-z0-9]+", "-", expand(s)).strip("-")
    return (s[:70] or "dish").strip("-")


def search_openverse(query: str) -> str | None:
    q = urllib.parse.urlencode(
        {
            "q": query,
            "license": "cc0,pdm,by,by-sa",
            "category": "photograph",
            "page_size": "5",
            "mature": "false",
        }
    )
    req = urllib.request.Request(
        f"https://api.openverse.org/v1/images/?{q}",
        headers={"User-Agent": UA, "Accept": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            data = json.load(resp)
    except Exception as e:
        print(f"  openverse miss: {e}", flush=True)
        return None
    for hit in data.get("results") or []:
        src = hit.get("url") or hit.get("thumbnail")
        if src and src.startswith("http"):
            return src
    return None


def download_webp(url: str, dest: Path) -> bool:
    dest.parent.mkdir(parents=True, exist_ok=True)
    raw = dest.with_suffix(".download")
    try:
        req = urllib.request.Request(url, headers={"User-Agent": UA})
        with urllib.request.urlopen(req, timeout=20) as resp:
            raw.write_bytes(resp.read())
        from PIL import Image

        im = Image.open(raw).convert("RGB")
        im.thumbnail((1024, 1024))
        im.save(dest, "WEBP", quality=82, method=4)
        raw.unlink(missing_ok=True)
        return True
    except Exception as e:
        print(f"  download miss: {e}", flush=True)
        raw.unlink(missing_ok=True)
        return False


def best_shelf(
    name: str,
    shelf: list[tuple[str, str, Path, str | None]],
    used: dict[str, int],
    *,
    min_score: float,
    prefer_unused: bool,
) -> str | None:
    want = protein_of(name)
    cands: list[tuple[float, float, str]] = []
    for folder, label, path, spro in shelf:
        sc = score(name, label, spro, want)
        if sc < min_score:
            continue
        ref = f"shelf:{folder}/{path.name}"
        adj = sc - 0.08 * used[ref]
        cands.append((adj, sc, ref))
    if not cands:
        return None
    cands.sort(reverse=True)
    if prefer_unused:
        unused = [c for c in cands if used[c[2]] == 0 and c[1] >= min_score + 0.05]
        if unused:
            return unused[0][2]
    return cands[0][2]


def main() -> int:
    print("loading template + shelf…", flush=True)
    data = json.loads(TEMPLATE.read_text(encoding="utf-8"))
    shelf = load_shelf()
    used: dict[str, int] = defaultdict(int)
    assignments: dict[str, str] = {}
    fetch_queue: list[tuple[str, str]] = []

    def place(key: str, name: str, is_category: bool) -> None:
        min_sc = 0.2 if is_category else 0.35
        ref = best_shelf(name, shelf, used, min_score=min_sc, prefer_unused=True)
        if ref:
            assignments[key] = ref
            used[ref] += 1
            return
        fetch_queue.append((key, name))

    for ci, cat in enumerate(data["categories"]):
        place(f"c:{ci}", cat["name"], True)
        for ii, item in enumerate(cat["items"]):
            place(f"i:{ci}:{ii}", item["name"], False)

    print(f"shelf-assigned {len(assignments)}; need fetch {len(fetch_queue)}", flush=True)
    IMAGES.mkdir(parents=True, exist_ok=True)

    for key, name in fetch_queue:
        fname = f"{slug(key + '-' + name)}.webp"
        dest = IMAGES / fname
        query = f"{expand(name)} indian food"
        print(f"fetch {name!r} → {fname}", flush=True)
        ok = False
        if dest.is_file():
            ok = True
        else:
            url = search_openverse(query)
            time.sleep(3.1)
            if url and download_webp(url, dest):
                ok = True
        if ok:
            assignments[key] = f"images/{fname}"
            used[assignments[key]] += 1
            continue
        # Same-protein shelf fallback (even if weak) — never cross protein.
        ref = best_shelf(name, shelf, used, min_score=0.0, prefer_unused=True)
        if ref is None:
            ref = best_shelf(name, shelf, used, min_score=-0.5, prefer_unused=False)
        if ref:
            print(f"  fallback {ref}", flush=True)
            assignments[key] = ref
            used[ref] += 1
        else:
            print(f"  WARN no image for {name!r}", flush=True)

    for ci, cat in enumerate(data["categories"]):
        if f"c:{ci}" in assignments:
            cat["image"] = assignments[f"c:{ci}"]
        for ii, item in enumerate(cat["items"]):
            k = f"i:{ci}:{ii}"
            if k in assignments:
                item["image"] = assignments[k]

    TEMPLATE.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    imgs = []
    for cat in data["categories"]:
        imgs.append(cat.get("image", ""))
        for item in cat["items"]:
            imgs.append(item.get("image", ""))
    ctr = Counter(imgs)
    dups = sorted(((k, v) for k, v in ctr.items() if v > 1), key=lambda x: -x[1])
    print(f"unique={len(ctr)} dups={len(dups)} total={len(imgs)}", flush=True)
    for k, v in dups[:12]:
        print(f"  {v}x {k}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
