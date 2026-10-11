#!/usr/bin/env bash
# Deploys the mod jar to the TEST server only, through the Pterodactyl client API:
# replaces satiscraft-*.jar in the mods folder, then restarts the server.
#
# Usage: scripts/deploy-test.sh [path/to/satiscraft-x.y.z.jar]
# Config: .env.deploy at the repo root (see .env.deploy.example) or environment variables.
# Production is updated by hand, never by this script.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -f "$repo_root/.env.deploy" ]]; then
  set -a
  # shellcheck source=/dev/null
  source "$repo_root/.env.deploy"
  set +a
fi

: "${PTERODACTYL_URL:?missing, see .env.deploy.example}"
: "${PTERODACTYL_API_KEY:?missing, see .env.deploy.example}"
: "${PTERODACTYL_SERVER_ID:?missing, see .env.deploy.example}"
mods_dir="${PTERODACTYL_MODS_DIR:-/mods}"

for tool in curl jq unzip; do
  command -v "$tool" >/dev/null || { echo "error: $tool is required" >&2; exit 1; }
done

jar="${1:-}"
if [[ -z "$jar" ]]; then
  jar="$(ls -t "$repo_root"/neoforge/build/libs/satiscraft-*.jar 2>/dev/null | grep -v -- '-sources' | head -n 1 || true)"
fi
[[ -f "$jar" ]] || { echo "error: no jar found, run ./gradlew build first" >&2; exit 1; }

# The V1 server runs Linux x86_64: refuse a jar built for another platform.
if ! unzip -l "$jar" | grep -q 'natives/linux-x86_64/libfactory_jni.so'; then
  echo "error: $jar has no natives/linux-x86_64 library (built on another platform?)." >&2
  echo "       Use the CI artifact, or build on Linux x86_64." >&2
  exit 1
fi

api="${PTERODACTYL_URL%/}/api/client/servers/$PTERODACTYL_SERVER_ID"
auth=(-H "Authorization: Bearer $PTERODACTYL_API_KEY" -H "Accept: application/json")

echo "==> Removing previous satiscraft jars from $mods_dir"
old_jars="$(curl -fsS "${auth[@]}" --get --data-urlencode "directory=$mods_dir" "$api/files/list" \
  | jq -c '[.data[].attributes.name | select(startswith("satiscraft-") and endswith(".jar"))]')"
if [[ "$old_jars" != "[]" ]]; then
  curl -fsS "${auth[@]}" -H "Content-Type: application/json" -X POST "$api/files/delete" \
    -d "$(jq -n --arg root "$mods_dir" --argjson files "$old_jars" '{root: $root, files: $files}')" >/dev/null
fi

echo "==> Uploading $(basename "$jar")"
upload_url="$(curl -fsS "${auth[@]}" "$api/files/upload" | jq -r '.attributes.url')"
curl -fsS -X POST "$upload_url&directory=$(jq -rn --arg d "$mods_dir" '$d|@uri')" -F "files=@$jar" >/dev/null

echo "==> Restarting the test server"
curl -fsS "${auth[@]}" -H "Content-Type: application/json" -X POST "$api/power" -d '{"signal":"restart"}' >/dev/null

echo "Deployed $(basename "$jar") to the test server."
