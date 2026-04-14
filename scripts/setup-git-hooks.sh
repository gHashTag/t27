#!/usr/bin/env bash
# Point this repo at .githooks/ (NOW.md pre-commit gate and future hooks).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit 2>/dev/null || true
echo "core.hooksPath=.githooks — pre-commit enforces docs/NOW.md (today's date)."
