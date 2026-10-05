#!/usr/bin/env bash
# Point this repo at .githooks/ (commit-time gates via `tri hooks pre-commit`).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit 2>/dev/null || true
echo "core.hooksPath=.githooks -- pre-commit runs \`tri hooks pre-commit\`."
