#!/usr/bin/env bash
# Validates copilot-instructions.md stays in sync with the codebase:
#   1. All doc files in manifest.json are referenced in the mapping table
#   2. All doc files in the mapping table actually exist on disk
#
# This script is part of the fastedge-plugin pipeline contract.
# Canonical template: fastedge-plugin/scripts/sync/templates/check-copilot-sync-template.sh
# Each source repo gets a copy at: fastedge-plugin-source/check-copilot-sync.sh
#
# Exits 0 if in sync, 1 if drift detected.

set -euo pipefail

MANIFEST="fastedge-plugin-source/manifest.json"
COPILOT=".github/copilot-instructions.md"
errors=0

if [ ! -f "$COPILOT" ]; then
  echo "FAIL: $COPILOT does not exist"
  exit 1
fi

# --- Check 1: manifest doc files appear in the mapping table ---

# Extract doc/schema paths that appear in mapping table rows (lines starting with '|')
# Use POSIX-compatible awk instead of grep -P so this works on macOS/BSD grep too
mapping_table_docs=$(awk '
  /^\|/ {
    line = $0
    while (match(line, /`(docs|schemas)\/[^`]+`/)) {
      print substr(line, RSTART + 1, RLENGTH - 2)
      line = substr(line, RSTART + RLENGTH)
    }
  }
' "$COPILOT" | sort -u)

if [ -z "$mapping_table_docs" ]; then
  echo "FAIL: No doc/schema paths found in $COPILOT mapping table — expected backticked docs/ or schemas/ paths in table rows"
  exit 1
fi

if [ -f "$MANIFEST" ]; then
  if ! command -v jq &>/dev/null; then
    echo "Error: jq is required but not installed"
    exit 1
  fi

  doc_files=$(jq -r '.sources[].files[]' "$MANIFEST" | grep -E '^(docs|schemas)/' | sort -u)

  missing=()
  for doc in $doc_files; do
    if ! echo "$mapping_table_docs" | grep -qF "$doc"; then
      missing+=("$doc")
    fi
  done

  if [ ${#missing[@]} -gt 0 ]; then
    echo "FAIL: Doc files from $MANIFEST missing from $COPILOT:"
    for f in "${missing[@]}"; do
      echo "  - $f"
    done
    errors=1
  else
    echo "OK: All manifest doc files are referenced in copilot-instructions.md"
  fi
else
  echo "SKIP: No manifest found at $MANIFEST"
fi

# --- Check 2: doc files referenced in mapping table exist on disk ---

# Reuse mapping_table_docs extracted above for check 2
stale=()
while IFS= read -r doc_path; do
  [ -z "$doc_path" ] && continue
  if [ ! -f "$doc_path" ]; then
    stale+=("$doc_path")
  fi
done <<< "$mapping_table_docs"

if [ ${#stale[@]} -gt 0 ]; then
  echo "FAIL: Doc files referenced in $COPILOT mapping table do not exist:"
  for f in "${stale[@]}"; do
    echo "  - $f"
  done
  errors=1
else
  echo "OK: All doc files in copilot-instructions mapping table exist on disk"
fi

# --- Result ---

if [ $errors -ne 0 ]; then
  echo ""
  echo "Fix the issues above and re-run this check."
  exit 1
fi
