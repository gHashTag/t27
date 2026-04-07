#!/bin/bash
# Close individual PRs after meta-PR is merged

set -e

# Meta PR that contains all the work
META_PR="215"

# Old PRs to close (they're superseded by meta PR)
OLD_PRS=(
  "212"
  "210"
  "208"
  "206"
  "202"
  "200"
  "198"
)

echo "=== T27 Auto-Close Old PRs ==="
echo "Waiting for meta PR #$META_PR to merge..."
echo ""

# Wait for meta PR to merge
while true; do
  meta_state=$(gh pr view $META_PR --json state --jq '.state')
  if [ "$meta_state" = "MERGED" ]; then
    echo "Meta PR #$META_PR merged! Closing old PRs..."
    break
  elif [ "$meta_state" = "CLOSED" ]; then
    echo "Meta PR #$META_PR closed. Checking merge status..."
    if gh pr view $META_PR --json merged --jq '.' | grep -q "true"; then
      echo "Meta PR #$META_PR was merged. Closing old PRs..."
      break
    else
      echo "Meta PR #$META_PR was closed without merge. Aborting."
      exit 1
    fi
  fi
  echo "Waiting... (state: $meta_state)"
  sleep 30
done

# Close old PRs
for old_pr in "${OLD_PRS[@]}"; do
  old_state=$(gh pr view $old_pr --json state --jq '.state')
  if [ "$old_state" = "OPEN" ]; then
    echo "Closing PR #$old_pr..."
    gh pr close $old_pr --comment "Superseded by meta PR #$META_PR which contains this work and has been merged."
  else
    echo "PR #$old_pr already $old_state"
  fi
done

echo ""
echo "=== Done ==="
echo "Meta PR #$META_PR merged and old PRs closed."
