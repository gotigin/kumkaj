#!/usr/bin/env bash
# Stage Khaynjo catalogue packs into the Makepad site static tree
# (sites/khaynjo/static/v1/catalogue/) — same Firebase deploy as the wasm site.
#
# Usage:
#   ./apps/khaynjo/scripts/publish-catalogue.sh
#       # sync public/items + rebuild public starters (khaynjo-starter, cafe-starter, khaynjo-items)
#   ./apps/khaynjo/scripts/publish-catalogue.sh --private-rajdhani --code=123456789012
#
# Private packs → sites/khaynjo/static/v1/catalogue/private/<code>/ (gitignored).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP="$ROOT/apps/khaynjo"
CDN="$ROOT/sites/khaynjo/static/v1/catalogue"
SHELF="$APP/assets/items"
SRC_TMPL="$APP/assets/templates/rajdhani/template.json"
STARTER_SRC="$APP/assets/templates/khaynjo-starter/template.json"
CAFE="$CDN/public/templates/cafe-starter"

PRIVATE_CODE=""
DO_RAJDHANI=0

for arg in "$@"; do
  case "$arg" in
    --private-rajdhani) DO_RAJDHANI=1 ;;
    --code=*) PRIVATE_CODE="${arg#--code=}" ;;
    -h|--help)
      sed -n '2,14p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown arg: $arg" >&2
      exit 1
      ;;
  esac
done

if [[ ! -d "$SHELF" ]]; then
  echo "Missing shelf $SHELF" >&2
  exit 1
fi

mkdir -p "$CDN/public/templates" "$CDN/public/items"

echo "1/3 Sync public/items from assets/items …"
rsync -a --delete --include='*/' --include='*.webp' --exclude='*' "$SHELF/" "$CDN/public/items/"

# Folder + file index for shelf download / site tooling.
python3 - "$CDN/public/items" <<'PY'
import json, sys
from pathlib import Path
root = Path(sys.argv[1])
folders = sorted(p.name for p in root.iterdir() if p.is_dir())
files = []
for folder in folders:
    for f in sorted((root / folder).glob("*.webp")):
        files.append(f"{folder}/{f.name}")
index = {"schema": 1, "folders": folders, "files": files, "count": len(files)}
(root / "index.json").write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")
print(f"  items index: {len(folders)} folders, {len(files)} webps")
PY

echo "2/3 Public templates (khaynjo-starter, cafe-starter, khaynjo-items) …"

# Self-contained multi-cuisine app starter from staged template.json + shelf images.
if [[ ! -f "$STARTER_SRC" ]]; then
  echo "Missing $STARTER_SRC — generate khaynjo-starter/template.json first." >&2
  exit 1
fi
python3 - "$STARTER_SRC" "$SHELF" "$CDN/public/templates/khaynjo-starter" <<'PY'
import json, shutil, sys
from pathlib import Path

src_tmpl, shelf, dest = map(Path, sys.argv[1:4])
dest.mkdir(parents=True, exist_ok=True)
images = dest / "images"
if images.exists():
    shutil.rmtree(images)
images.mkdir(parents=True)
data = json.loads(src_tmpl.read_text(encoding="utf-8"))
used = {}
missing = []

def map_image(raw: str) -> str:
    raw = (raw or "").strip()
    if not raw:
        return ""
    if not raw.startswith("shelf:"):
        return raw
    rel = raw[len("shelf:"):]
    if rel in used:
        return used[rel]
    src = shelf / rel
    out_rel = f"images/{rel}"
    out_path = dest / out_rel
    out_path.parent.mkdir(parents=True, exist_ok=True)
    if src.is_file():
        shutil.copy2(src, out_path)
    else:
        missing.append(rel)
    used[rel] = out_rel
    return out_rel

for cat in data.get("categories", []):
    cat["image"] = map_image(cat.get("image", ""))
    for item in cat.get("items", []):
        item["image"] = map_image(item.get("image", ""))

out_tmpl = dest / "template.json"
out_tmpl.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
bytes_n = out_tmpl.stat().st_size
manifest = {
    "schema": 1,
    "id": "khaynjo-starter",
    "title": "Khaynjo starter menu",
    "blurb": "World cuisines from the public Khaynjo shelf — edit prices after import.",
    "kind": "app_template",
    "visibility": "public",
    "applies_shop_defaults": False,
    "shop_defaults_id": "",
    "template": "template.json",
    "bytes": bytes_n,
}
(dest / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
if missing:
    print(f"  warn: {len(missing)} missing shelf images", file=sys.stderr)
print(f"  khaynjo-starter: {len(used)} images, {bytes_n} bytes")
PY

# Café starter (small text-only pack) — keep as a second public option.
if [[ -f "$CAFE/template.json" ]]; then
  CAFE_BYTES=$(wc -c < "$CAFE/template.json" | tr -d ' ')
  python3 - "$CAFE/manifest.json" "$CAFE_BYTES" <<'PY'
import json, sys
path, bytes_s = sys.argv[1], int(sys.argv[2])
with open(path, encoding="utf-8") as f:
    data = json.load(f)
data["bytes"] = bytes_s
data.setdefault("kind", "app_template")
data.setdefault("visibility", "public")
with open(path, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
PY
  echo "  cafe-starter: ${CAFE_BYTES} bytes"
fi

# Khaynjo Items Template — shelf kind pointing at public/items/.
ITEMS_COUNT=$(python3 -c "import json; print(json.load(open('$CDN/public/items/index.json'))['count'])")
ITEMS_BYTES=$(python3 -c "import json; print(sum(__import__('os').path.getsize('$CDN/public/items/'+f) for f in json.load(open('$CDN/public/items/index.json'))['files']))")
mkdir -p "$CDN/public/templates/khaynjo-items"
cat > "$CDN/public/templates/khaynjo-items/manifest.json" <<EOF
{
  "schema": 1,
  "id": "khaynjo-items",
  "title": "Khaynjo Items Template",
  "blurb": "Full public dish photo shelf (~${ITEMS_COUNT} dishes) for Import complete template.",
  "kind": "shelf",
  "visibility": "public",
  "applies_shop_defaults": false,
  "shop_defaults_id": "",
  "template": "",
  "items_index": "../../items/index.json",
  "bytes": ${ITEMS_BYTES}
}
EOF
# Marker so download_pack can treat this pack as a shelf pointer.
cat > "$CDN/public/templates/khaynjo-items/template.json" <<'EOF'
{
  "id": "khaynjo-items",
  "title": "Khaynjo Items Template",
  "blurb": "Full public dish photo shelf.",
  "categories": []
}
EOF
echo "  khaynjo-items: shelf (${ITEMS_COUNT} files, ${ITEMS_BYTES} bytes)"

# Public index listing (never private packs).
STARTER_BYTES=$(wc -c < "$CDN/public/templates/khaynjo-starter/template.json" | tr -d ' ')
CAFE_BYTES=$(wc -c < "$CAFE/template.json" 2>/dev/null | tr -d ' ' || echo 0)
python3 - "$CDN/public/index.json" "$STARTER_BYTES" "$CAFE_BYTES" "$ITEMS_BYTES" "$ITEMS_COUNT" <<'PY'
import json, sys
path, starter_b, cafe_b, items_b, items_n = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
templates = [
    {
        "id": "khaynjo-starter",
        "title": "Khaynjo starter menu",
        "blurb": "World cuisines from the public shelf — edit prices after import.",
        "kind": "app_template",
        "bytes": starter_b,
    },
    {
        "id": "cafe-starter",
        "title": "Café starter",
        "blurb": "Tea, coffee, and a few snacks — edit prices after import.",
        "kind": "app_template",
        "bytes": cafe_b,
    },
    {
        "id": "khaynjo-items",
        "title": "Khaynjo Items Template",
        "blurb": f"Full public dish photo shelf (~{items_n} dishes).",
        "kind": "shelf",
        "bytes": items_b,
    },
]
data = {"schema": 1, "templates": templates}
with open(path, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
print(f"  index.json: {len(templates)} public templates")
PY

echo "3/3 Private Rajdhani (optional) …"
if [[ "$DO_RAJDHANI" -eq 1 ]]; then
  if [[ ! "$PRIVATE_CODE" =~ ^[0-9]{12}$ ]]; then
    echo "--code= must be exactly 12 digits" >&2
    exit 1
  fi
  if [[ ! -f "$SRC_TMPL" ]]; then
    echo "Missing $SRC_TMPL" >&2
    exit 1
  fi
  DEST="$CDN/private/$PRIVATE_CODE"
  rm -rf "$DEST"
  mkdir -p "$DEST/images"
  python3 - "$SRC_TMPL" "$SHELF" "$DEST" <<'PY'
import json, shutil, sys
from pathlib import Path
src_tmpl, shelf, dest = map(Path, sys.argv[1:4])
data = json.loads(src_tmpl.read_text(encoding="utf-8"))
used = {}
missing = []

def map_image(raw: str) -> str:
    raw = (raw or "").strip()
    if not raw:
        return ""
    if not raw.startswith("shelf:"):
        return raw
    rel = raw[len("shelf:"):]
    if rel in used:
        return used[rel]
    src = shelf / rel
    out_rel = f"images/{rel}"
    out_path = dest / out_rel
    out_path.parent.mkdir(parents=True, exist_ok=True)
    if src.is_file():
        shutil.copy2(src, out_path)
    else:
        missing.append(rel)
    used[rel] = out_rel
    return out_rel

for cat in data.get("categories", []):
    cat["image"] = map_image(cat.get("image", ""))
    for item in cat.get("items", []):
        item["image"] = map_image(item.get("image", ""))
out_tmpl = dest / "template.json"
out_tmpl.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
if missing:
    print(f"warn: {len(missing)} missing shelf images", file=sys.stderr)
print(f"copied {len(used)} images")
PY
  BYTES=$(wc -c < "$DEST/template.json" | tr -d ' ')
  cat > "$DEST/manifest.json" <<EOF
{
  "schema": 1,
  "id": "rajdhani",
  "title": "Rajdhani Menu Template",
  "blurb": "Client menu + shop defaults (Zahir / Asif / Nawaz).",
  "kind": "app_template",
  "visibility": "private",
  "applies_shop_defaults": true,
  "shop_defaults_id": "rajdhani",
  "template": "template.json",
  "bytes": $BYTES
}
EOF
  echo "Private Rajdhani pack → $DEST (code $PRIVATE_CODE — keep off public git)"
else
  echo "  skipped (pass --private-rajdhani --code=… to stage)"
fi

echo "Done. Site static tree: $CDN"
echo "Deploy: ./sites/khaynjo/scripts/deploy-firebase.sh"
echo "Local app test: KHAYNJO_CATALOGUE_DIR=$CDN"
