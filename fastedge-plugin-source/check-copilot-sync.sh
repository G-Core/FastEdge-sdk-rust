#!/usr/bin/env bash
# Validates copilot-instructions.md stays in sync with the codebase:
#   1. All doc files in manifest.json are referenced in the mapping table
#   2. All doc files in the mapping table actually exist on disk
#   3. All example directories on disk are tracked in manifest.json
#
# This script is part of the fastedge-plugin pipeline contract.
# Canonical template: fastedge-plugin/scripts/sync/templates/check-copilot-sync-template.sh
# Each source repo gets a copy at: fastedge-plugin-source/check-copilot-sync.sh
#
# Exits 0 if in sync (warnings don't affect exit code), 1 if drift detected.

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
# Uses awk to parse backticked paths — works on macOS/BSD and Linux
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

  doc_files=$(jq -r '.sources[].files[] | select(startswith("docs/") or startswith("schemas/"))' "$MANIFEST" | sort -u)

  missing=()
  for doc in $doc_files; do
    if ! printf '%s\n' "$mapping_table_docs" | grep -qxF "$doc"; then
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

# --- Check 3: example directories on disk are tracked in manifest ---
#
# Detects example projects not listed in manifest.json so the fastedge-plugin
# pipeline doesn't silently miss new examples. Uses ::warning:: annotations
# for visibility in GitHub PR checks.
#
# This check is advisory (does not affect exit code) because repos may have
# known gaps during rollout.

if [ -d "examples" ] && [ -f "$MANIFEST" ]; then
  # Get all examples/ file paths from the manifest (filter in jq to avoid
  # grep exit-1 on no matches, which would kill the script under pipefail)
  manifest_examples=$(jq -r '.sources[].files[] | select(startswith("examples/"))' "$MANIFEST" | sort -u)

  # Find example project directories (contain package.json, Cargo.toml, or asconfig.json)
  # Handles flat (examples/<name>/) and nested (examples/cdn/<name>/) structures
  untracked=()
  while IFS= read -r marker_file; do
    [ -z "$marker_file" ] && continue
    project_dir=$(dirname "$marker_file")
    # Check if any manifest file starts with this project directory
    # Uses awk index() for literal prefix match (grep would treat . [ + as regex)
    if [ -z "$manifest_examples" ] || ! printf '%s\n' "$manifest_examples" | awk -v prefix="${project_dir}/" 'index($0, prefix) == 1 {found=1; exit} END {exit !found}'; then
      untracked+=("$project_dir")
    fi
  done < <(find examples/ -maxdepth 4 \( -name "package.json" -o -name "Cargo.toml" -o -name "asconfig.json" \) -not -path "*/node_modules/*" 2>/dev/null | sort)

  if [ ${#untracked[@]} -gt 0 ]; then
    echo "WARN: Example directories not tracked in $MANIFEST:"
    for dir in "${untracked[@]}"; do
      echo "  - $dir"
      echo "::warning::Example directory '$dir' is not tracked in manifest.json. Add it to fastedge-plugin-source/manifest.json so the fastedge-plugin pipeline can access it."
    done
    echo ""
    echo "  To fix: add source entries for the above directories to $MANIFEST"
  else
    echo "OK: All example directories are tracked in manifest.json"
  fi
elif [ ! -d "examples" ]; then
  echo "SKIP: No examples/ directory"
else
  echo "SKIP: No manifest found at $MANIFEST (cannot check examples coverage)"
fi

# --- Result ---

if [ $errors -ne 0 ]; then
  echo ""
  echo "Fix the issues above and re-run this check."
  exit 1
fi
