#!/usr/bin/env bash
# Fail if any catalogue language pack is missing an en() key.
# Mirrors apps/<app>/src/locale.rs::every_catalogue_language_translates_every_key.
# Aliases (pt-BR, es-MX, yue, gom) fold to their parent table — no TSV of their own.
#
#   ./scripts/check-ui-translations.sh              # every app
#   ./scripts/check-ui-translations.sh --app khaynjo
#
# Bash + gawk only — do not reintroduce Python for this gate.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="all"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) APP="${2:-}"; shift 2 ;;
    -h|--help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

check_app() {
  local app="$1"
  local locale="$ROOT/apps/$app/src/locale.rs"
  local tr="$ROOT/apps/$app/translations"
  [[ -f "$locale" ]] || { echo "missing $locale" >&2; return 1; }

  gawk -v app="$app" -v tr="$tr" '
    function table_code(c) {
      if (c == "pt-BR") return "pt"
      if (c == "es-MX") return "es"
      if (c == "yue") return "zh-Hant"
      if (c == "gom") return "kok"
      return c
    }
    function trim(s) {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", s)
      return s
    }
    BEGIN {
      # ---- parse locale.rs ----
      while ((getline line < ARGV[1]) > 0) {
        if (line ~ /fn en\(key: &str\) -> String \{/) in_en = 1
        else if (in_en && line ~ /_ => key,/) in_en = 0
        else if (in_en && match(line, /^[[:space:]]*"([^"]+)"[[:space:]]*=>/, m)) {
          nkeys++
          keys[nkeys] = m[1]
          key_set[m[1]] = 1
        }
        if (line ~ /pub const LANGUAGES/) in_l = 1
        if (in_l && match(line, /code:[[:space:]]*"([^"]+)"/, m)) {
          c = m[1]
          if (c != "en") {
            t = table_code(c)
            if (!(t in seen)) {
              seen[t] = 1
              ntables++
              tables[ntables] = t
            }
          }
        }
        if (in_l && line ~ /^\];/) in_l = 0
      }
      close(ARGV[1])

      for (ti = 1; ti <= ntables; ti++) {
        code = tables[ti]
        path = tr "/" code ".tsv"
        delete table
        ntable = 0
        if ((getline line < path) < 0) {
          gaps[++ng] = app ": missing translations/" code ".tsv"
          close(path)
          continue
        }
        # first line already read; process it then the rest
        do {
          if (line ~ /^[[:space:]]*$/ || line ~ /^#/) continue
          if (index(line, "\t") == 0) continue
          split(line, parts, "\t")
          k = parts[1]
          v = substr(line, length(k) + 2)
          table[k] = v
          ntable++
        } while ((getline line < path) > 0)
        close(path)

        for (ki = 1; ki <= nkeys; ki++) {
          k = keys[ki]
          if (!(k in table)) {
            gaps[++ng] = app "/" code " left " k " untranslated"
          } else {
            v = trim(table[k])
            if (v == "") gaps[++ng] = app "/" code " " k " is empty"
            else if (v == k) gaps[++ng] = app "/" code " " k " shows the raw key"
          }
        }
        for (k in table) {
          if (!(k in key_set)) gaps[++ng] = app "/" code ": stale key " k
        }
      }

      for (i = 1; i <= ng; i++) print gaps[i]
      exit (ng > 0 ? 1 : 0)
    }
  ' "$locale"
}

apps=()
if [[ "$APP" == "all" ]]; then
  apps=(khaynjo moklanjo kirano tabiyat)
else
  apps=("$APP")
fi

all_gaps="$(mktemp)"
trap 'rm -f "$all_gaps"' EXIT
ok=1
for app in "${apps[@]}"; do
  if ! check_app "$app" >>"$all_gaps"; then
    ok=0
  fi
done

n="$(wc -l <"$all_gaps" | tr -d ' ')"
if [[ "$ok" -ne 0 && "$n" -eq 0 ]]; then
  echo "UI translations complete for: ${apps[*]}"
  exit 0
fi

echo "$n translation gaps:" >&2
head -n 80 "$all_gaps" | sed 's/^/  /' >&2
if [[ "$n" -gt 80 ]]; then
  echo "  … $((n - 80)) more" >&2
fi
exit 1
