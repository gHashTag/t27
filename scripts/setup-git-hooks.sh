#!/usr/bin/env bash
<<<<<<< Updated upstream
# Point this repo at .githooks/ (NOW.md pre-commit gate, NotebookLM pre-push gate, and future hooks).
# phi^2 + 1/phi^2 = 3 | TRINITY
=======
# Point this repo at .githooks/ (NOW.md pre-commit gate and NotebookLM pre-push gate).
>>>>>>> Stashed changes
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit 2>/dev/null || true
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
echo "core.hooksPath=.githooks — pre-commit enforces NOW.md (today's date)."
=======
=======
>>>>>>> Stashed changes
chmod +x .githooks/pre-push 2>/dev/null || true
echo "core.hooksPath=.githooks"
echo "  - pre-commit: enforces docs/NOW.md (today's date)"
echo "  - pre-push: enforces NotebookLM notebook ID"
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
chmod +x .githooks/pre-push 2>/dev/null || true
echo "core.hooksPath=.githooks — pre-commit enforces docs/NOW.md, pre-push enforces NotebookLM notebook."
>>>>>>> Stashed changes
