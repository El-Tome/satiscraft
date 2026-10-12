#!/usr/bin/env bash
# Fails when a source file passes the line limit, so that code is split by concept instead
# of piling up in one file (see "Code size" in CLAUDE.md).
#
# Usage: scripts/check-file-size.sh
# Checks tracked and new (not ignored) .rs, .kt, .kts, .java, .sh and .py files, tests
# included. Generated sources and data files (TOML, JSON) are exempt.
# MAX_LINES overrides the limit (600). Runs on the bash 3.2 that ships with macOS.
set -euo pipefail

MAX_LINES="${MAX_LINES:-600}"

# Prints "<lines> <path>" for each source file of the repo above the limit.
oversized_files() {
  local file lines
  git ls-files -z --cached --others --exclude-standard -- \
    '*.rs' '*.kt' '*.kts' '*.java' '*.sh' '*.py' |
    while IFS= read -r -d '' file; do
      case "$file" in */generated/*) continue ;; esac
      [ -f "$file" ] || continue
      lines=$(($(wc -l <"$file")))
      if [ "$lines" -gt "$MAX_LINES" ]; then
        printf '%s %s\n' "$lines" "$file"
      fi
    done
}

cd "$(dirname "$0")/.."
found=0
while read -r lines file; do
  found=1
  echo "$file: $lines lines, limit $MAX_LINES: split it by concept (CLAUDE.md, Code size)" >&2
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    echo "::error file=$file::$lines lines, limit $MAX_LINES: split it by concept"
  fi
done < <(oversized_files)

if [ "$found" -eq 1 ]; then
  exit 1
fi
echo "Every source file is under $MAX_LINES lines."
