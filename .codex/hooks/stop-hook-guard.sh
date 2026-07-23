#!/bin/bash
# Stop Hook Guard - Runs before Claude Code session stops
# Purpose: Save current state and prevent data loss

set -euo pipefail

# Log stop event
echo "[$(date -Iseconds)] STOP: Session ending" >> ~/.claude/session-stop.log

# Check for uncommitted changes in git repos
if git rev-parse --git-dir > /dev/null 2>&1; then
    if ! git diff-index --quiet HEAD --; then
        echo "WARNING: Uncommitted changes detected" | tee -a ~/.claude/session-stop.log
        git status --short >> ~/.claude/session-stop.log
    fi
fi

# Save current PHI LOOP state if exists
if [ -f .trinity/state/phi-loop.json ]; then
    cp .trinity/state/phi-loop.json .trinity/state/phi-loop-last.json
fi

echo "Stop hook completed"
