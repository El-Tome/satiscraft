#!/usr/bin/env bash
# Deploys to the TEST server only, over SFTP and the Pterodactyl client API.
#
# Usage:
#   scripts/deploy-test.sh [jar...]
#       Replaces every satiscraft*.jar of the mods folder, uploads the dependency mods (other
#       jars) that are missing or outdated, then restarts the server and checks the core loaded.
#       Without arguments: the latest build of each mod (mods/*/build/libs) and the
#       dependencies staged by `./gradlew :modpack:serverMods` (modpack/build/serverMods).
#       The engine jar (satiscraft-<version>.jar) is required; Satiscraft jars share its version.
#   scripts/deploy-test.sh --check             changes nothing: mods folder, server state, key scope
#   scripts/deploy-test.sh --power SIGNAL      start, stop, restart or kill, then waits for the result
#   scripts/deploy-test.sh --log [LINES]       end of the server console (logs/latest.log)
#
# Config: .env.deploy at the repo root (see .env.deploy.example) or environment variables;
# the environment wins. Pterodactyl scopes an SFTP account to a single server.
# Production is updated by hand, never by this script.
# Runs on the bash 3.2 that ships with macOS.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

fail() {
  echo "error: $*" >&2
  exit 1
}

# Reads KEY=VALUE lines literally, without shell expansion: passwords may contain $ or #.
# Surrounding quotes are dropped. Values are not exported, so child processes never see them.
load_env_file() {
  local line key value ends
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%$'\r'}"
    [[ "$line" =~ ^[[:space:]]*(#|$) || "$line" != *=* ]] && continue
    key="${line%%=*}"
    value="${line#*=}"
    [[ "$key" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || continue
    if ((${#value} >= 2)); then
      ends="${value:0:1}${value: -1}"
      if [[ "$ends" == '""' || "$ends" == "''" ]]; then
        value="${value:1:${#value}-2}"
      fi
    fi
    declare -p "$key" >/dev/null 2>&1 || printf -v "$key" '%s' "$value"
  done <"$1"
}

if [[ -f "$repo_root/.env.deploy" ]]; then
  load_env_file "$repo_root/.env.deploy"
fi

: "${PTERODACTYL_SFTP:?missing, see .env.deploy.example}"
: "${PTERODACTYL_SFTP_USER:?missing, see .env.deploy.example}"
: "${PTERODACTYL_SFTP_PASSWORD:?missing, see .env.deploy.example}"
mods_dir="${PTERODACTYL_MODS_DIR:-/mods}"
# Keeps the SFTP batches below free of quoting issues.
[[ "$mods_dir" =~ ^/[A-Za-z0-9._/-]*$ ]] ||
  fail "PTERODACTYL_MODS_DIR must be an absolute path without spaces"

for tool in sftp sshpass unzip; do
  command -v "$tool" >/dev/null || fail "$tool is required"
done

# --- SFTP and API -------------------------------------------------------------------------

sftp_target="${PTERODACTYL_SFTP#sftp://}"
sftp_target="${sftp_target%%/*}"
sftp_host="${sftp_target%:*}"
sftp_port=2022
if [[ "$sftp_target" == *:* ]]; then sftp_port="${sftp_target##*:}"; fi

# Runs the SFTP batch read on stdin and stops at its first failing command.
# BatchMode=no must come before -b, which would otherwise forbid password authentication.
run_sftp() {
  SSHPASS="$PTERODACTYL_SFTP_PASSWORD" sshpass -e sftp -q \
    -oBatchMode=no -oStrictHostKeyChecking=accept-new -oConnectTimeout=15 \
    -P "$sftp_port" -b - "$PTERODACTYL_SFTP_USER@$sftp_host"
}

# Names of the files in the mods folder.
list_mods() {
  local listing
  listing="$(printf 'ls -1 %s\n' "$mods_dir" | run_sftp)" || fail "cannot list $mods_dir over SFTP"
  printf '%s\n' "$listing" | sed -e '/^sftp>/d' -e 's|.*/||' -e '/^$/d'
}

# Prints the last LINES lines of the server console.
print_log() {
  local tmp
  tmp="$(mktemp)"
  if ! printf 'get /logs/latest.log "%s"\n' "$tmp" | run_sftp >/dev/null; then
    rm -f "$tmp"
    fail "cannot read logs/latest.log over SFTP"
  fi
  tail -n "$1" "$tmp"
  rm -f "$tmp"
}

require_api_key() {
  [[ -n "${PTERODACTYL_API_KEY:-}" ]] || fail "PTERODACTYL_API_KEY is not set, see .env.deploy.example"
}

# Calls the Pterodactyl client API: METHOD PATH [curl args], PATH relative to /api/client.
# The key goes through stdin so that it never shows in the process list.
client_api() {
  local method="$1" path="$2"
  shift 2
  : "${PTERODACTYL_URL:?missing, see .env.deploy.example}"
  command -v curl >/dev/null || fail "curl is required"
  printf 'Authorization: Bearer %s\n' "$PTERODACTYL_API_KEY" | curl -fsS -H @- \
    -H "Accept: application/json" -H "Content-Type: application/json" -X "$method" \
    "${PTERODACTYL_URL%/}/api/client$path" "$@"
}

# Same, on the test server: METHOD ENDPOINT [curl args].
server_api() {
  local method="$1" endpoint="$2"
  shift 2
  : "${PTERODACTYL_SERVER_ID:?missing, see .env.deploy.example}"
  client_api "$method" "/servers/$PTERODACTYL_SERVER_ID/$endpoint" "$@"
}

# offline, starting, running or stopping.
server_state() {
  local resources
  resources="$(server_api GET resources)" || fail "the client API refused the request"
  printf '%s' "$resources" | grep -o '"current_state":"[a-z]*"' | cut -d'"' -f4
}

# Sends a power signal and waits until the server reaches the state it leads to.
# Pterodactyl reports "running" once the server has finished starting.
power() {
  local signal="$1" target="running" state="" last="" left=true started=$SECONDS offline_since=""
  case "$signal" in
    start | restart) ;;
    stop | kill) target="offline" ;;
    *) fail "unknown power signal '$signal' (start, stop, restart or kill)" ;;
  esac
  # A restart first leaves "running"; anything else may already be in its target state.
  if [[ "$signal" == "restart" ]]; then left=false; fi
  server_api POST power -d "{\"signal\":\"$signal\"}" >/dev/null
  while ((SECONDS - started < 300)); do
    state="$(server_state)"
    if [[ "$state" != "$last" ]]; then
      echo "    $state"
      last="$state"
    fi
    if [[ "$state" != "running" ]]; then left=true; fi
    if $left && [[ "$state" == "$target" ]]; then return 0; fi
    # A server that should start but stays offline has crashed while starting.
    if [[ "$target" == "running" && "$state" == "offline" ]]; then
      if [[ -z "$offline_since" ]]; then offline_since=$SECONDS; fi
      if ((SECONDS - offline_since >= 45)); then
        fail "the server stays offline instead of starting; see scripts/deploy-test.sh --log"
      fi
    else
      offline_since=""
    fi
    sleep 2
  done
  fail "the server is still $state after $((SECONDS - started))s"
}

case "${1:-}" in
  --check)
    echo "==> SFTP: $mods_dir on $sftp_host:$sftp_port"
    list_mods | sed 's/^/    /'
    if [[ -z "${PTERODACTYL_API_KEY:-}" ]]; then
      echo "PTERODACTYL_API_KEY is not set: restarts will be manual."
      exit 0
    fi
    echo "==> API: state of the test server"
    echo "    $(server_state)"
    # The key should belong to an account that only reaches the test server.
    servers="$(client_api GET "?per_page=100" | grep -o '"identifier":"[^"]*"' | cut -d'"' -f4)" ||
      fail "the client API refused to list servers"
    count="$(printf '%s\n' "$servers" | grep -c . || true)"
    echo "==> API: the key reaches $count server(s)"
    # Root admins reach every server through the client API, listed or not.
    admin="$(client_api GET /account | grep -o '"admin":[a-z]*' | cut -d: -f2)" ||
      fail "the client API refused to read the account"
    if ((count > 1)) || [[ "$admin" != "false" ]]; then
      echo "warning: this key can act on other servers than the test one; use a key from a" >&2
      echo "         non-admin account that only has access to the test server." >&2
    fi
    exit 0
    ;;
  --power)
    require_api_key
    echo "==> Power: ${2:-}"
    power "${2:-}"
    exit 0
    ;;
  --log)
    lines="${2:-40}"
    [[ "$lines" =~ ^[0-9]+$ ]] || fail "--log takes a number of lines"
    print_log "$lines"
    exit 0
    ;;
  --*)
    fail "unknown option $1"
    ;;
esac

# --- Jars to deploy -----------------------------------------------------------------------

jars=()
if (($# > 0)); then
  jars=("$@")
else
  for libs in "$repo_root"/mods/*/build/libs; do
    latest=""
    for jar in "$libs"/satiscraft*.jar; do
      [[ -f "$jar" && "$jar" != *-sources.jar ]] || continue
      if [[ -z "$latest" || "$jar" -nt "$latest" ]]; then latest="$jar"; fi
    done
    if [[ -n "$latest" ]]; then jars+=("$latest"); fi
  done
  for jar in "$repo_root"/modpack/build/serverMods/*.jar; do
    if [[ -f "$jar" ]]; then jars+=("$jar"); fi
  done
fi
((${#jars[@]} > 0)) || fail "no jar found, run ./gradlew build first"

engine=""
version=""
names=" "
dep_prefixes=" "
for jar in "${jars[@]}"; do
  [[ -f "$jar" ]] || fail "$jar: no such file"
  [[ "$jar" != *[\"\\]* ]] || fail "$jar: path must not contain quotes or backslashes"
  name="$(basename "$jar")"
  [[ "$names" != *" $name "* ]] || fail "$name is given twice"
  names="$names$name "
  if [[ "$name" =~ ^(satiscraft(-[a-z]+)*)-([0-9][0-9A-Za-z.+-]*)\.jar$ ]]; then
    [[ -z "$version" || "$version" == "${BASH_REMATCH[3]}" ]] || fail "satiscraft jars have different versions"
    version="${BASH_REMATCH[3]}"
    if [[ "${BASH_REMATCH[1]}" == "satiscraft" ]]; then engine="$jar"; fi
  elif [[ "$name" =~ ^([A-Za-z0-9._+-]+)-[0-9][0-9A-Za-z.+-]*\.jar$ ]]; then
    # A dependency mod: replaces older versions of the same artifact.
    dep_prefixes="$dep_prefixes${BASH_REMATCH[1]}- "
  else
    fail "$name: cannot tell the artifact from the version"
  fi
done
[[ -n "$engine" ]] || fail "the engine jar (satiscraft-<version>.jar) is missing"

# The test server runs Linux x86_64: refuse an engine jar built for another platform.
if ! unzip -l "$engine" | grep -q 'natives/linux-x86_64/libfactory_jni.so'; then
  echo "error: $engine has no natives/linux-x86_64 library (built on another platform?)." >&2
  echo "       Use the CI artifact, or build on Linux x86_64." >&2
  exit 1
fi

# --- Upload -------------------------------------------------------------------------------

echo "==> Updating $mods_dir on $sftp_host"
listing="$(list_mods)"
present=" "
batch=""
while IFS= read -r entry; do
  [[ "$entry" == *.jar && "$entry" != *[[:space:]]* ]] || continue
  if [[ "$names" == *" $entry "* && "$entry" != satiscraft* ]]; then
    present="$present$entry "
    continue
  fi
  remove=false
  if [[ "$entry" == satiscraft*.jar ]]; then remove=true; fi
  for prefix in $dep_prefixes; do
    if [[ "$entry" == "$prefix"[0-9]* ]]; then remove=true; fi
  done
  if $remove; then
    echo "    removing $entry"
    batch+="rm $mods_dir/$entry"$'\n'
  fi
done <<<"$listing"
for jar in "${jars[@]}"; do
  name="$(basename "$jar")"
  if [[ "$present" == *" $name "* ]]; then
    echo "    keeping $name"
    continue
  fi
  echo "    uploading $name"
  batch+="put \"$jar\" $mods_dir/$name"$'\n'
done
printf '%s' "$batch" | run_sftp >/dev/null || fail "SFTP upload failed, check $mods_dir by hand"

# --- Restart ------------------------------------------------------------------------------

if [[ -z "${PTERODACTYL_API_KEY:-}" ]]; then
  echo "Uploaded $version. PTERODACTYL_API_KEY is not set: restart the test server from the panel."
  exit 0
fi

echo "==> Restarting the test server"
power restart
# The core logs its version a few seconds after the server reports it is running.
waited_since=$SECONDS
until print_log 3000 | grep -E "factory-core .* loaded"; do
  if ((SECONDS - waited_since >= 60)); then
    echo "error: factory-core did not load within 60s; see scripts/deploy-test.sh --log" >&2
    exit 1
  fi
  sleep 5
done
echo "Deployed $version to the test server."
