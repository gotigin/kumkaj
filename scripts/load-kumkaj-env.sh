# Source from an app's scripts:  source "$ROOT/scripts/load-kumkaj-env.sh"
# then: load_kumkaj_env development|production
# Exports keys from the app's env file and sets KUMKAJ_ENV.
#
# Product env files live with each app crate (apps/<app>/env/, not the
# workspace root) so every Kumkaj product owns its own ids/labels.
# Which app: KUMKAJ_ENV_DIR, else KUMKAJ_APP_DIR — one of them is required
# (no default app: a missing one must fail, not package another product).
load_kumkaj_env() {
  local name="${1:?usage: load_kumkaj_env development|production}"
  local dir="${KUMKAJ_ENV_DIR:-${KUMKAJ_APP_DIR:+$KUMKAJ_APP_DIR/env}}"
  if [[ -z "$dir" ]]; then
    echo "load_kumkaj_env: set KUMKAJ_APP_DIR (apps/<app>) first" >&2
    return 1
  fi
  local file="$dir/${name}.env"
  if [[ ! -f "$file" ]]; then
    echo "Missing Kumkaj env file: $file" >&2
    return 1
  fi
  # Parse KEY=VALUE (don't `source` — unquoted spaces break bash, e.g. Kumkaj Dev).
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line#"${line%%[![:space:]]*}"}"
    [[ -z "$line" || "$line" == \#* ]] && continue
    [[ "$line" != *=* ]] && continue
    local key="${line%%=*}"
    local value="${line#*=}"
    key="${key%"${key##*[![:space:]]}"}"
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    value="${value#\"}"
    value="${value%\"}"
    value="${value#\'}"
    value="${value%\'}"
    export "$key=$value"
  done <"$file"
  export KUMKAJ_ENV="$name"
  echo "Kumkaj env: $name (APP_ID=${KUMKAJ_APP_ID:-?} LABEL=${KUMKAJ_APP_LABEL:-?})"
}
