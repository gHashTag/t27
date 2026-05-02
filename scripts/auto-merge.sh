#!/bin/bash
# Auto-merge PRs in order with automatic NOW.md conflict resolution

set -e

REPO="gHashTag/t27"
PRS_TO_MERGE=(
  "198"
  "200"
  "202"
  "206"
  "208"
  "210"
  "212"
)

echo "=== T27 Auto-Merge Script ==="
echo ""

# Function to auto-resolve NOW.md conflicts
resolve_now_conflict() {
  local branch=$1

  echo "  Resolving NOW.md conflict for $branch..."

  # Use master's version (accumulated)
  git checkout --theirs docs/NOW.md

  # Add all and continue
  git add docs/NOW.md
  git commit -m "auto-resolve: NOW.md conflict (auto-merged from master)"
}

# Process each PR in order
for pr_num in "${PRS_TO_MERGE[@]}"; do
  echo "Processing PR #$pr_num..."

  # Get PR info
  pr_data=$(gh pr view $pr_num --json headRefName,state,mergeable --jq '.')
  head_ref=$(echo "$pr_data" | jq -r '.headRefName')
  state=$(echo "$pr_data" | jq -r '.state')
  mergeable=$(echo "$pr_data" | jq -r '.mergeable')

  echo "  Branch: $head_ref"
  echo "  State: $state"
  echo "  Mergeable: $mergeable"

  # Skip if not open
  if [ "$state" != "OPEN" ]; then
    echo "  SKIPPED: not open"
    continue
  fi

  # Skip if already merged
  if git merge-base --is-ancestor "origin/$head_ref" HEAD 2>/dev/null; then
    echo "  SKIPPED: already merged"
    continue
  fi

  # Checkout and merge
  echo "  Merging $head_ref..."

  if git merge "origin/$head_ref" --no-edit; then
    echo "  SUCCESS: clean merge"
  else
    echo "  CONFLICT: resolving..."

    # Auto-resolve NOW.md conflicts
    if git status | grep -q "docs/NOW.md"; then
      resolve_now_conflict "$head_ref"
    else
      echo "  ERROR: unresolvable conflict (not NOW.md)"
      git merge --abort
      exit 1
    fi
  fi
done

echo ""
echo "=== All merges complete ==="
echo ""
echo "Pushing to master..."
git push origin master

echo ""
echo "=== Done ==="
