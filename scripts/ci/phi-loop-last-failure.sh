#!/usr/bin/env bash
# Print failed-step logs for the latest GitHub Actions run of phi-loop-ci.yml.
# Requires: gh auth login, network. Run from any directory inside the repo.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

JSON="$(gh run list --workflow=phi-loop-ci.yml --limit 1 --json databaseId,conclusion,headBranch,displayTitle)"
ID="$(echo "$JSON" | jq -r '.[0].databaseId')"
CONC="$(echo "$JSON" | jq -r '.[0].conclusion')"
BR="$(echo "$JSON" | jq -r '.[0].headBranch')"
TITLE="$(echo "$JSON" | jq -r '.[0].displayTitle')"

echo "Latest phi-loop-ci: run=$ID branch=$BR conclusion=$CONC"
echo "  $TITLE"
echo ""

if [[ "$CONC" == "success" ]]; then
  echo "Latest run succeeded — no failed logs."
  exit 0
fi

gh run view "$ID" --log-failed
