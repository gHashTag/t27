#!/usr/bin/env bash
# CI only: require root NOW.md in the PR or push diff (GitHub Actions).
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

event="${GITHUB_EVENT_NAME:?GITHUB_EVENT_NAME must be set}"

if [ "$event" = "pull_request" ]; then
  BASE="${PR_BASE_SHA:?}"
  HEAD="${PR_HEAD_SHA:?}"
  CHANGED=$(git diff --name-only "$BASE" "$HEAD" | grep -x 'NOW.md' || true)
elif [ "$event" = "push" ]; then
  BEFORE="${PUSH_BEFORE:?}"
  AFTER="${PUSH_AFTER:?}"
  if [ "$BEFORE" = "0000000000000000000000000000000000000000" ]; then
    CHANGED=$(git show --name-only --pretty=format: "$AFTER" | grep -x 'NOW.md' || true)
  else
    CHANGED=$(git diff --name-only "$BEFORE" "$AFTER" | grep -x 'NOW.md' || true)
  fi
else
  echo "::error::now-sync-gate-diff.sh: unsupported GITHUB_EVENT_NAME=$event"
  exit 1
fi

if [ -z "$CHANGED" ]; then
  echo "::error file=NOW.md::❌ SYNC REQUIRED: NOW.md was NOT updated in this PR/push."
  echo ""
  echo "Every PR/push to master must include an update to NOW.md."
  echo "See: https://github.com/gHashTag/t27/issues/141 (coordination anchor)"
  exit 1
fi

echo "✅ NOW.md is in the change set"
