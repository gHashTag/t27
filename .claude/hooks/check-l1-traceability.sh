#!/usr/bin/env bash
# L1 TRACEABILITY gate — thin forwarder to the Rust implementation.
#
# Real logic lives in `cli/tri` (`tri hooks l1-check`). This file exists
# only so that pre-existing harness wiring that exec's the .sh path keeps
# working. Do not add logic here — edit `cli/tri/src/hooks.rs` instead.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

for p in \
  "$REPO_ROOT/target/release/tri" \
  "$REPO_ROOT/target/debug/tri" \
  ; do
  if [[ -x "$p" ]]; then
    exec "$p" hooks l1-check
  fi
done

# Fallback if the Rust binary is not yet built (e.g. fresh clone).
COMMIT_MSG=$(git log -1 --pretty=%B HEAD)
if ! echo "$COMMIT_MSG" | grep -qE "(Closes|Fixes|Resolves|Reference) #[0-9]+"; then
    echo "L1 VIOLATION: Commit missing issue reference"
    echo "Commit message: $COMMIT_MSG"
    echo "Required pattern: Closes #N | Fixes #N | Resolves #N | Reference #N"
    exit 1
fi
ISSUE_NUM=$(echo "$COMMIT_MSG" | grep -oE "#[0-9]+" | head -1)
echo "L1 PASSED: Issue $ISSUE_NUM referenced"
