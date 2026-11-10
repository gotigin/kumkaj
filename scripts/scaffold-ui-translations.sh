#!/usr/bin/env bash
# Keep translation packs in sync with en() after you add UI strings.
# Creates missing translations/<code>.tsv for catalogue languages
# (aliases pt-BR / es-MX / yue / gom need no file).
#
#   ./scripts/scaffold-ui-translations.sh --app khaynjo
#   ./scripts/scaffold-ui-translations.sh --app khaynjo --standin-english
#
# With --standin-english, missing rows get the English source temporarily;
# replace those with real translations, then:
#   ./scripts/check-ui-translations.sh --app <app>
# Bash only — do not reintroduce Python for this.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="all"
STANDIN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) APP="${2:-}"; shift 2 ;;
    --standin-english) STANDIN=1; shift ;;
    -h|--help)
      sed -n '2,14p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

table_code() {
  case "$1" in
    pt-BR) echo pt ;;
    es-MX) echo es ;;
    yue) echo zh-Hant ;;
    gom) echo kok ;;
    *) echo "$1" ;;
  esac
}

# Emit KEY<TAB>VALUE for every en() arm (gawk; handles \" \\ \n \t).
en_pairs() {
  gawk '
    function unescape(s,   out, i, c, n) {
      out = ""
      n = length(s)
      for (i = 1; i <= n; i++) {
        c = substr(s, i, 1)
        if (c == "\\" && i < n) {
          i++
          c = substr(s, i, 1)
          if (c == "n") out = out "\n"
          else if (c == "t") out = out "\t"
          else if (c == "\"") out = out "\""
          else if (c == "\\") out = out "\\"
          else out = out c
        } else out = out c
      }
      return out
    }
    /fn en\(key: &str\) -> String \{/ { in_en=1; next }
    in_en && /_ => key,/ { exit }
    in_en {
      if (match($0, /^[[:space:]]*"([^"]+)"[[:space:]]*=>[[:space:]]*"((?:\\.|[^"\\])*)"/, m)) {
        print m[1] "\t" unescape(m[2])
      }
    }
  ' "$1"
}

lang_codes() {
  gawk '
    /pub const LANGUAGES/ { in_l=1 }
    in_l {
      if (match($0, /code:[[:space:]]*"([^"]+)"/, m)) print m[1]
      if (/^\];/) exit
    }
  ' "$1"
}

needed_tables() {
  local c t
  declare -A seen=()
  while IFS= read -r c; do
    case "$c" in en|pt-BR|es-MX|yue|gom) continue ;; esac
    t="$(table_code "$c")"
    [[ -n "${seen[$t]:-}" ]] && continue
    seen[$t]=1
    printf '%s\n' "$t"
  done < <(lang_codes "$1")
}

scaffold_app() {
  local app="$1"
  local locale="$ROOT/apps/$app/src/locale.rs"
  local tr="$ROOT/apps/$app/translations"
  local en_file keys_file code path header k v missing n_missing
  mkdir -p "$tr"
  en_file="$(mktemp)"
  keys_file="$(mktemp)"
  en_pairs "$locale" >"$en_file"
  cut -f1 "$en_file" >"$keys_file"
  local n_en
  n_en="$(wc -l <"$en_file" | tr -d ' ')"
  local codes=()
  mapfile -t codes < <(needed_tables "$locale")
  echo "== $app: $n_en en keys, ${#codes[@]} packs =="

  local gaps=0
  for code in "${codes[@]}"; do
    path="$tr/$code.tsv"
    declare -A table=()
    header=""
    if [[ -f "$path" ]]; then
      while IFS= read -r line || [[ -n "${line:-}" ]]; do
        [[ -z "${line:-}" ]] && continue
        if [[ "$line" == \#* ]]; then
          [[ -z "$header" ]] && header="$line"
          continue
        fi
        [[ "$line" != *$'\t'* ]] && continue
        k="${line%%$'\t'*}"
        v="${line#*$'\t'}"
        table["$k"]="$v"
      done <"$path"
    fi

    missing=()
    while IFS=$'\t' read -r k v; do
      [[ -n "$k" ]] || continue
      if [[ ! -v "table[$k]" || -z "${table[$k]//[[:space:]]/}" ]]; then
        missing+=("$k")
      fi
    done <"$en_file"
    n_missing="${#missing[@]}"
    gaps=$((gaps + n_missing))

    if [[ "$STANDIN" -eq 1 ]]; then
      declare -A enmap=()
      while IFS=$'\t' read -r k v; do
        enmap["$k"]="$v"
      done <"$en_file"
      for k in "${missing[@]}"; do
        table["$k"]="${enmap[$k]}"
      done
      [[ -n "$header" ]] || header="# $code. One key<TAB>text per line, in en() order; keep {placeholders}."
      {
        printf '%s\n' "$header"
        while IFS=$'\t' read -r k v; do
          printf '%s\t%s\n' "$k" "${table[$k]:-${enmap[$k]}}"
        done <"$en_file"
      } >"$path"
      echo "  $code: scaffolded $n_missing → $n_en rows"
      unset enmap
    else
      if [[ ! -f "$path" ]]; then
        printf '%s\n' "# $code. One key<TAB>text per line, in en() order; keep {placeholders}." >"$path"
        echo "  $code: created empty pack ($n_missing keys to fill)"
      elif [[ "$n_missing" -gt 0 ]]; then
        echo "  $code: $n_missing missing (e.g. ${missing[0]:-} ${missing[1]:-} ${missing[2]:-})"
      else
        echo "  $code: ok"
      fi
    fi
    unset table
  done

  rm -f "$en_file" "$keys_file"
  echo "$gaps"
}

apps=()
if [[ "$APP" == "all" ]]; then
  apps=(khaynjo moklanjo kirano tabiyat)
else
  apps=("$APP")
fi

total=0
for app in "${apps[@]}"; do
  out="$(scaffold_app "$app")"
  printf '%s\n' "$out"
  g="$(printf '%s\n' "$out" | tail -n1)"
  total=$((total + g))
done

if [[ "$STANDIN" -eq 1 ]]; then
  echo
  echo "Next: replace English stand-ins with real translations, then:"
  echo "  ./scripts/check-ui-translations.sh --app $APP"
  exit 0
fi

if [[ "$total" -gt 0 ]]; then
  echo
  echo "$total gaps. Scaffold with --standin-english, translate, then check:" >&2
  echo "  ./scripts/scaffold-ui-translations.sh --app $APP --standin-english" >&2
  echo "  ./scripts/check-ui-translations.sh --app $APP" >&2
  exit 1
fi
