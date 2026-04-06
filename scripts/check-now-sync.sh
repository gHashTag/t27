#!/usr/bin/env bash
# NOW sync gate — called by tri gen/compile, phi-loop CI, and locally.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NOW_FILE="$ROOT/docs/NOW.md"
TODAY=$(date +%Y-%m-%d)

if [ ! -f "$NOW_FILE" ]; then
  echo "tri/CI: docs/NOW.md not found at $NOW_FILE" >&2
  exit 1
fi

LINE=$(grep -m1 "Last updated:" "$NOW_FILE" 2>/dev/null || true)
LAST=""
if [ -n "$LINE" ]; then
  LAST=$(echo "$LINE" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' | head -1 || true)
fi

if [ "$LAST" != "$TODAY" ]; then
  cat << 'EOF'

╔═══════════════════════════════════════════════════════════════╗
║              ⛔  BUILD BLOCKED: SYNC REQUIRED                  ║
╠═══════════════════════════════════════════════════════════════╣
║  docs/NOW.md is STALE. All agents must be synchronized       ║
║  before any build can proceed.                               ║
╠═══════════════════════════════════════════════════════════════╣
║  STEPS TO UNBLOCK:                                            ║
║                                                               ║
║  1. Read coordination anchor:                                 ║
║     https://github.com/gHashTag/t27/issues/141               ║
║                                                               ║
║  2. Read agent sync state:                                    ║
║     cat .trinity/state/github-sync.json                      ║
║                                                               ║
║  3. Update docs/NOW.md:                                       ║
║     - Set "Last updated: TODAY"                              ║
║     - Update current sprint status                           ║
║     - Note what you are about to build and why               ║
║                                                               ║
║  4. Stage and commit NOW.md with your changes:               ║
║     git add docs/NOW.md && git commit --amend                ║
╚═══════════════════════════════════════════════════════════════╝
EOF
  echo "(Expected Last updated: $TODAY; found: ${LAST:-<none>})" >&2
  exit 1
fi

echo "✅ NOW.md synced ($TODAY) — build authorized"
