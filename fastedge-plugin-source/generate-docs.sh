#!/usr/bin/env bash
set -euo pipefail

# Generate docs/ from source code using .generation-config.md
#
# Usage:
#   ./fastedge-plugin-source/generate-docs.sh                          # all files (parallel where possible)
#   ./fastedge-plugin-source/generate-docs.sh SDK_API.md               # specific file
#   ./fastedge-plugin-source/generate-docs.sh SDK_API.md HOST_SERVICES.md # multiple files
#
# Reference implementation: fastedge-test/fastedge-plugin-source/generate-docs.sh

# Model to use for generation (sonnet is recommended for cost efficiency)
MODEL="sonnet"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_FILE="$SCRIPT_DIR/.generation-config.md"
DOCS_DIR="$REPO_ROOT/docs"

# --- Cleanup on interrupt ---
# Two mechanisms work together to ensure clean shutdown:
#
# 1. kill_tree() recursively kills each background subshell AND its children
#    (the `claude` processes). Plain `kill $pid` only kills the subshell,
#    leaving `claude` orphaned.
#
# 2. INTERRUPT_FLAG file signals subshells to stop retrying. Bash variables
#    don't cross process boundaries, so a temp file is the reliable way to
#    communicate "stop" to background jobs before their next retry iteration.
ALL_PIDS=()
INTERRUPT_FLAG=$(mktemp /tmp/.generate-docs-interrupt.XXXXXX)
rm -f "$INTERRUPT_FLAG"  # absent = running; present = stop

kill_tree() {
  local pid=$1
  local children
  children=$(pgrep -P "$pid" 2>/dev/null || true)
  for child in $children; do
    kill_tree "$child"
  done
  kill "$pid" 2>/dev/null || true
}

cleanup() {
  echo ""
  echo "Interrupted — killing background processes..."
  touch "$INTERRUPT_FLAG"  # tell subshells to stop retrying
  trap - INT TERM          # prevent re-entry

  for pid in "${ALL_PIDS[@]}"; do
    kill_tree "$pid"
  done

  # Clean up temp files left by killed generate_file subshells
  # Must happen BEFORE kill -- -$$ which kills this script too
  rm -f "$DOCS_DIR"/.[A-Z]*.[a-zA-Z0-9]* 2>/dev/null || true
  rm -f "$INTERRUPT_FLAG"

  # Belt-and-suspenders: kill entire process group (including this script)
  kill -- -$$ 2>/dev/null || true
  exit 130
}

trap cleanup INT TERM

# =============================================================================
# === CUSTOMIZE: Define your doc files and their dependency tiers ===
#
# Tier 1: Independent files (generated in parallel — read from source code only)
# Tier 2: Files that reference tier 1 docs (e.g. quickstart, getting-started)
# Tier 3: Files that summarize all other docs (e.g. INDEX.md)
# =============================================================================

TIER1_FILES=("SDK_API.md" "HOST_SERVICES.md")
TIER2_FILES=("quickstart.md")
TIER3_FILES=("INDEX.md")

ALL_FILES=("${TIER1_FILES[@]}" "${TIER2_FILES[@]}" "${TIER3_FILES[@]}")

# =============================================================================
# === CUSTOMIZE: Map each doc file to its source files ===
#
# Keys must match the filenames in the tier arrays above.
# Values are space-separated paths relative to the repo root.
# The script reads each file and passes its content to the generation prompt.
# =============================================================================

declare -A SOURCE_FILES
SOURCE_FILES[SDK_API.md]="src/lib.rs src/http_client.rs derive/src/lib.rs Cargo.toml"
SOURCE_FILES[HOST_SERVICES.md]="src/lib.rs src/proxywasm/key_value.rs src/proxywasm/secret.rs src/proxywasm/dictionary.rs src/proxywasm/utils.rs"
SOURCE_FILES[quickstart.md]="Cargo.toml src/lib.rs examples/http/wasi/hello_world/src/lib.rs examples/http/basic/hello_world/src/lib.rs"
SOURCE_FILES[INDEX.md]="Cargo.toml"

# =============================================================================
# === CUSTOMIZE: Package name for the generation prompt ===
# =============================================================================

PACKAGE_NAME="fastedge"

# =============================================================================
# === END CUSTOMIZATION — everything below is the reusable engine ===
# =============================================================================

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: $CONFIG_FILE not found"
  exit 1
fi

# Determine which files to generate
if [ $# -eq 0 ]; then
  targets=("${ALL_FILES[@]}")
  run_all=true
else
  targets=("$@")
  run_all=false
  # Validate targets
  for target in "${targets[@]}"; do
    found=false
    for valid in "${ALL_FILES[@]}"; do
      if [ "$target" = "$valid" ]; then
        found=true
        break
      fi
    done
    if [ "$found" = false ]; then
      echo "Error: unknown doc file '$target'"
      echo "Valid files: ${ALL_FILES[*]}"
      exit 1
    fi
  done
fi

mkdir -p "$DOCS_DIR"

# Clean up stale temp files from previous interrupted runs
rm -f "$DOCS_DIR"/.[A-Z]*.[a-zA-Z0-9]* 2>/dev/null || true

generate_file() {
  local target="$1"

  # Validate that SOURCE_FILES has an entry for this target
  if [ -z "${SOURCE_FILES[$target]+set}" ] || [ -z "${SOURCE_FILES[$target]}" ]; then
    echo "  ERROR: no SOURCE_FILES entry for '$target' — add one to the CUSTOMIZE section"
    return 1
  fi

  local sources="${SOURCE_FILES[$target]}"

  # Build the source files content block
  local source_content=""
  local loaded=0
  for src in $sources; do
    local full_path="$REPO_ROOT/$src"
    if [ ! -f "$full_path" ]; then
      echo "  Warning: source file $src not found, skipping"
      continue
    fi
    source_content+="
--- FILE: $src ---
$(cat "$full_path")
--- END FILE ---
"
    loaded=$((loaded + 1))
  done

  if [ "$loaded" -eq 0 ]; then
    echo "  ERROR: all source files for '$target' are missing (expected: $sources)"
    return 1
  fi

  # Extract the section for this target from generation-config
  # Use awk variable to avoid regex delimiter issues with /
  local section
  local escaped_target="docs/$target"
  section=$(awk -v start="## $escaped_target" '
    $0 == start { found=1; next }
    found && /^## docs\// { exit }
    found { print }
  ' "$CONFIG_FILE")

  # Validate that the config section exists and has content
  if [ -z "$(echo "$section" | tr -d '[:space:]')" ]; then
    echo "  ERROR: no instructions found for '$target' — add a '## docs/$target' section to $CONFIG_FILE"
    return 1
  fi

  # Check for existing doc to enable incremental updates
  # When docs/<file> already exists, it is passed as context so the model
  # preserves accurate content and manual additions, only changing what is
  # incorrect, incomplete, or missing per the source code.
  local existing_doc=""
  local existing_path="$DOCS_DIR/$target"
  local mode="Generate"
  if [ -f "$existing_path" ]; then
    existing_doc=$(cat "$existing_path")
    mode="Update"
  fi

  if [ "$mode" = "Update" ]; then
    echo "Updating docs/$target ..."
  else
    echo "Generating docs/$target ..."
  fi

  local existing_section=""
  if [ -n "$existing_doc" ]; then
    existing_section="
# Existing Content for docs/$target
Use this as the baseline. Preserve all accurate content and manual additions. Only change what is incorrect, incomplete, or missing per the source code. Keep sections not covered by the instructions above. Apply table formatting rules to all tables.

<existing>
$existing_doc
</existing>
"
  fi

  # Build prompt with sandwich output constraint:
  # The OUTPUT CONSTRAINT appears at both the start and end of the prompt.
  # This is critical for large prompts where the model may lose track of
  # the instruction to output only raw markdown. Without it, the model
  # sometimes produces conversational preamble or asks for permission.
  local prompt
  prompt="$(cat <<PROMPT
OUTPUT CONSTRAINT: Your output is piped directly to a file. Output ONLY raw markdown. No conversational text. No preamble. No "here is" or "I'll generate". No questions. No explanation. No permission requests. Start your very first character with # (the level-1 heading). End with the last line of markdown.

Generate docs/$target for the $PACKAGE_NAME crate.

# Global Rules
$(awk '/^## Global Rules$/{found=1; next} found && /^## docs\//{exit} found{print}' "$CONFIG_FILE")

# Instructions for this file
$section

# Source Code to Reference
$source_content
$existing_section
REMINDER: Output raw markdown only. First character must be #. No conversational text.
PROMPT
)"

  # Write to a temp file first, only move to docs/ on success.
  # This protects the existing file from being clobbered by a failed attempt,
  # so re-running the script after a failure still has the original content
  # available for incremental updates.
  local tmpfile
  tmpfile=$(mktemp "$DOCS_DIR/.${target}.XXXXXX")
  trap "rm -f '$tmpfile'" RETURN

  # Retry loop: validate that the model produced raw markdown (starts with #)
  # On large prompts, the model occasionally produces conversational output
  # despite the sandwich constraint. Retrying usually succeeds.
  local max_attempts=3
  local attempt=1
  while [ $attempt -le $max_attempts ]; do
    # Stop retrying if parent signalled interrupt
    if [ -f "$INTERRUPT_FLAG" ]; then
      rm -f "$tmpfile"
      return 130
    fi

    claude -p --model "$MODEL" "$prompt" > "$tmpfile"

    # Validate: first non-empty line must start with #
    local first_line
    first_line=$(grep -m1 '.' "$tmpfile" || true)
    if [[ "$first_line" == \#* ]]; then
      mv "$tmpfile" "$DOCS_DIR/$target"
      echo "  Done: docs/$target"
      return 0
    fi

    echo "  Attempt $attempt/$max_attempts failed for $target (got conversational output), retrying..."
    attempt=$((attempt + 1))
  done

  rm -f "$tmpfile"
  echo "  FAILED after $max_attempts attempts: docs/$target"
  return 1
}

# Run a tier of files in parallel, wait for all to complete
run_tier() {
  local tier_name="$1"
  shift
  local files=("$@")
  local pids=()
  local failed=()

  # Skip empty tiers
  if [ ${#files[@]} -eq 0 ]; then
    return 0
  fi

  echo "--- $tier_name (${#files[@]} files in parallel) ---"

  for target in "${files[@]}"; do
    generate_file "$target" &
    pids+=($!)
    ALL_PIDS+=($!)
  done

  # Wait for all and collect failures
  for i in "${!pids[@]}"; do
    if ! wait "${pids[$i]}"; then
      failed+=("${files[$i]}")
    fi
  done

  if [ ${#failed[@]} -gt 0 ]; then
    echo "  FAILED in $tier_name: ${failed[*]}"
    return 1
  fi
  return 0
}

# --- Main execution ---

if [ "$run_all" = true ]; then
  # Parallel tiered execution
  tier_failed=false

  run_tier "Tier 1: Core reference" "${TIER1_FILES[@]}" || tier_failed=true

  if [ "$tier_failed" = false ]; then
    run_tier "Tier 2: Quickstart" "${TIER2_FILES[@]}" || tier_failed=true
  else
    echo "Skipping Tier 2 due to Tier 1 failures"
  fi

  if [ "$tier_failed" = false ]; then
    run_tier "Tier 3: Index" "${TIER3_FILES[@]}" || tier_failed=true
  else
    echo "Skipping Tier 3 due to earlier failures"
  fi

  echo ""
  echo "=== Generation Complete ==="
  echo "Generated: ${#ALL_FILES[@]} file(s) in docs/"
  if [ "$tier_failed" = true ]; then
    echo "Some files failed — check output above"
    exit 1
  fi

  # Regenerate llms.txt from docs/ contents (if the script is installed)
  if [ -x "$SCRIPT_DIR/generate-llms-txt.sh" ]; then
    "$SCRIPT_DIR/generate-llms-txt.sh"
  fi
else
  # Specific files: run sequentially (user chose explicit order)
  failed=()
  for target in "${targets[@]}"; do
    if ! generate_file "$target"; then
      failed+=("$target")
      echo "  FAILED: docs/$target"
    fi
  done

  echo ""
  echo "=== Generation Complete ==="
  echo "Generated: ${#targets[@]} file(s) in docs/"
  if [ ${#failed[@]} -gt 0 ]; then
    echo "Failed: ${failed[*]}"
    exit 1
  fi
fi
