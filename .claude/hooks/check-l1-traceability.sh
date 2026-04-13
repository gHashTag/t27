#!/bin/bash
# L1 Traceability Check - Ensures commits reference issues
# L1: "No code merged without `Closes #N`"

set -euo pipefail

# Get last commit message
COMMIT_MSG=$(git log -1 --pretty=%B HEAD)

# Check for issue reference pattern
if ! echo "$COMMIT_MSG" | grep -qE "Closes #|Fixes #|Resolves #|Reference #"; then
    echo "L1 VIOLATION: Commit missing issue reference"
    echo "Commit message: $COMMIT_MSG"
    echo "Required pattern: Closes #N, Fixes #N, etc."
    exit 1
fi

# Check for issue number after pattern
ISSUE_NUM=$(echo "$COMMIT_MSG" | grep -oE "#[0-9]+" | head -1)
if [ -z "$ISSUE_NUM" ]; then
    echo "L1 VIOLATION: No issue number found"
    exit 1
fi

echo "L1 PASSED: Issue #$ISSUE_NUM referenced"
exit 0
