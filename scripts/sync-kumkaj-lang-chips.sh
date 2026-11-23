#!/usr/bin/env bash
# Regenerate kumkaj.com §06 WordChips from apps/khaynjo LANGUAGES.
# Run after changing the catalogue. check-language-parity.sh enforces app equality.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 - "$ROOT" <<'PY'
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
locale = (root / "apps/khaynjo/src/locale.rs").read_text()
start = locale.find("pub const LANGUAGES")
end = locale.find("\n];", start)
chunk = locale[start:end]
rows = re.findall(
    r'code:\s*"([^"]+)"\s*,\s*name_en:\s*"([^"]*)"\s*,\s*native_name:\s*"([^"]*)"',
    chunk,
)
if len(rows) < 100:
    raise SystemExit(f"expected ~115 languages, got {len(rows)}")

def esc(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')

food = {
    "en": "Food ready",
    "hi": "खाना तैयार",
    "gu": "જમવાનું તૈયાર",
    "ta": "உணவு தயார்",
    "bn": "খাবার তৈরি",
    "ur": "کھانا تیار",
}
lines = []
for code, _name_en, native in rows:
    word = food.get(code, native)
    lines.append(
        f'                                WordChip{{lang.body: "{esc(native)}" word.body: "{esc(word)}"}}'
    )
block = "\n".join(lines)
ui = root / "sites/kumkaj/src/ui.rs"
text = ui.read_text()
begin, endm = "// BEGIN CATALOGUE_LANG_CHIPS", "// END CATALOGUE_LANG_CHIPS"
if begin not in text or endm not in text:
    raise SystemExit("markers missing in sites/kumkaj/src/ui.rs — run once by hand")
text = re.sub(
    re.escape(begin) + r".*?" + re.escape(endm),
    begin + "\n" + block + "\n                                " + endm,
    text,
    count=1,
    flags=re.S,
)
ui.write_text(text)
print(f"synced {len(rows)} WordChips into sites/kumkaj/src/ui.rs")
PY
