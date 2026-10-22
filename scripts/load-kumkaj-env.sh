# Source from other scripts:  source "$ROOT/scripts/load-kumkaj-env.sh" development
# Exports keys from the app's env file and sets KUMKAJ_ENV.
#
# Product env files live with each app crate (apps/<app>/env/, not the
# workspace root) so every Kumkaj product owns its own ids/labels.
# Which app: KUMKAJ_ENV_DIR wins, else KUMKAJ_APP_DIR, else courier.
load_kumkaj_env() {
  local name="${1:?usage: load_kumkaj_env development|production}"
  local root="${KUMKAJ_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
  local file="${KUMKAJ_ENV_DIR:-${KUMKAJ_APP_DIR:-$root/apps/courier}/env}/${name}.env"
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
