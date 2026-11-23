#!/usr/bin/env bash
# Hard rule: every Kumkaj app ships the same language catalogue.
# No app may miss a language another app has. Websites that claim or draw
# catalogue languages must use this same set (see docs/WEBSITES.md).
#
#   ./scripts/check-language-parity.sh
#
# Compares apps/{khaynjo,moklanjo,kirano,tabiyat}:
#   - LANGUAGES codes in src/locale.rs (must be identical, same order)
#   - translations/*.tsv stems (must be identical sets)
#   - Core WorldFonts Regular Noto faces under resources/fonts/
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPS=(khaynjo moklanjo kirano tabiyat)
REF=khaynjo

extract_codes() {
  local locale="$1"
  gawk '
    /pub const LANGUAGES/ { in_l = 1 }
    in_l && match($0, /code:[[:space:]]*"([^"]+)"/, m) { print m[1] }
    in_l && /^];/ { in_l = 0 }
  ' "$locale"
}

tsv_stems() {
  local dir="$1"
  find "$dir" -maxdepth 1 -name '*.tsv' -printf '%f\n' | sed 's/\.tsv$//' | sort
}

ref_codes="$(extract_codes "$ROOT/apps/$REF/src/locale.rs")"
ref_tsv="$(tsv_stems "$ROOT/apps/$REF/translations")"
ref_n="$(printf '%s\n' "$ref_codes" | grep -c . || true)"

if [[ "$ref_n" -lt 100 ]]; then
  echo "error: $REF LANGUAGES only has $ref_n codes — parse failed?" >&2
  exit 1
fi

fail=0
for app in "${APPS[@]}"; do
  codes="$(extract_codes "$ROOT/apps/$app/src/locale.rs")"
  if [[ "$codes" != "$ref_codes" ]]; then
    echo "error: apps/$app/src/locale.rs LANGUAGES differs from apps/$REF (codes or order)" >&2
    fail=1
  fi
  tsv="$(tsv_stems "$ROOT/apps/$app/translations")"
  if [[ "$tsv" != "$ref_tsv" ]]; then
    echo "error: apps/$app/translations set differs from apps/$REF" >&2
    only_ref="$(comm -23 <(printf '%s\n' "$ref_tsv") <(printf '%s\n' "$tsv") || true)"
    only_app="$(comm -13 <(printf '%s\n' "$ref_tsv") <(printf '%s\n' "$tsv") || true)"
    [[ -n "$only_ref" ]] && echo "  missing in $app: $only_ref" >&2
    [[ -n "$only_app" ]] && echo "  only in $app: $only_app" >&2
    fail=1
  fi
done

# Core script faces every app WorldFonts must keep (names relative to resources/fonts).
CORE_FONTS=(
  NotoSans-Regular.ttf
  NotoSansDevanagari-Regular.ttf
  NotoSansBengali-Regular.ttf
  NotoSansTamil-Regular.ttf
  NotoSansTelugu-Regular.ttf
  NotoSansGujarati-Regular.ttf
  NotoSansKannada-Regular.ttf
  NotoSansMalayalam-Regular.ttf
  NotoSansGurmukhi-Regular.ttf
  NotoSansOriya-Regular.ttf
  NotoSansArabic-Regular.ttf
  NotoSansHebrew-Regular.ttf
  NotoSansThai-Regular.ttf
  NotoSansSinhala-Regular.ttf
  NotoSansMyanmar-Regular.ttf
  NotoSansKhmer-Regular.ttf
  NotoSansLao-Regular.ttf
  NotoSansTibetan-Regular.ttf
  NotoSansEthiopic-Regular.ttf
  NotoSansThaana-Regular.ttf
  NotoSansGeorgian-Regular.ttf
  NotoSansArmenian-Regular.ttf
  NotoSansMeeteiMayek-Regular.ttf
  NotoSansOlChiki-Regular.ttf
  fa-solid-900.ttf
)

for app in "${APPS[@]}"; do
  for f in "${CORE_FONTS[@]}"; do
    if [[ ! -s "$ROOT/apps/$app/resources/fonts/$f" ]]; then
      echo "error: apps/$app/resources/fonts/$f missing (WorldFonts / language scripts)" >&2
      fail=1
    fi
  done
done

if [[ "$fail" -ne 0 ]]; then
  echo "language parity FAILED — every app must ship the union catalogue; copy from $REF" >&2
  exit 1
fi

# kumkaj.com §06 chips must list every catalogue language (native_name).
KUMKAJ_UI="$ROOT/sites/kumkaj/src/ui.rs"
if [[ -f "$KUMKAJ_UI" ]]; then
  chips="$(grep -c 'WordChip{' "$KUMKAJ_UI" || true)"
  if [[ "$chips" != "$ref_n" ]]; then
    echo "error: sites/kumkaj WordChip count is $chips, LANGUAGES has $ref_n — run ./scripts/sync-kumkaj-lang-chips.sh" >&2
    fail=1
  fi
fi

if [[ "$fail" -ne 0 ]]; then
  echo "language parity FAILED after site chip check" >&2
  exit 1
fi

echo "language parity OK: ${#APPS[@]} apps × $ref_n LANGUAGES codes, $(printf '%s\n' "$ref_tsv" | grep -c .) translation tables, core WorldFonts present, kumkaj chips=$ref_n"
