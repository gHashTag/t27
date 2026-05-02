#!/bin/bash
# Check-conflicts.sh - Detect which PRs need merge

set -e

echo "=== T27 Conflict Checker ==="
echo ""

# Branches to check in merge order
BRANCHES=(
  "feat/ring-051-verdict-schema"
  "feat/ring-052-lotus-automation"
  "feat/ring-053-property-test"
  "feat/ring-055-experience-schema"
  "feat/ring-056-schema-validation-ci"
)

# Stale PRs (older PRs that may need attention first)
STALE_PRS=(
  "feat/ring-039-clara-prep-plan"
  "feat/ring-035-technology-tree"
  "feat/ring-041-gf16-arxiv-draft"
  "ring/037-soul-parser-enforcement"
)

echo "Checking Phase 4 PRs (merge order):"
echo ""

MERGED=0
PENDING=0

for branch in "${BRANCHES[@]}"; do
  if git merge-base --is-ancestor "origin/$branch" master 2>/dev/null; then
    echo "  [MERGED] $branch"
    ((MERGED++))
  else
    echo "  [PENDING] $branch"
    ((PENDING++))
  fi
done

echo ""
echo "Summary: $MERGED merged, $PENDING pending"
echo ""

echo "Checking stale PRs:"
echo ""

for branch in "${STALE_PRS[@]}"; do
  if git merge-base --is-ancestor "origin/$branch" master 2>/dev/null; then
    echo "  [MERGED] $branch"
  else
    echo "  [PENDING] $branch (STALE - review needed)"
  fi
done

echo ""
echo "=== End of check ==="
