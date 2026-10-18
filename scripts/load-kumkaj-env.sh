# Source from other scripts:  source "$ROOT/scripts/load-kumkaj-env.sh" development
# Exports keys from apps/courier/env/<name>.env and sets KUMKAJ_ENV.
#
# Product env files live with the courier crate (not the workspace root) so each
# future Kumkaj product can own its own ids/labels.
load_kumkaj_env() {
  local name="${1:?usage: load_kumkaj_env development|production}"
  local root="${KUMKAJ_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
  local file="${KUMKAJ_ENV_DIR:-$root/apps/courier/env}/${name}.env"
  if [[ ! -f "$file" ]]; then
    echo "Missing Kumkaj Courier env file: $file" >&2
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
  echo "Kumkaj Courier env: $name (APP_ID=${KUMKAJ_APP_ID:-?} LABEL=${KUMKAJ_APP_LABEL:-?})"
}
