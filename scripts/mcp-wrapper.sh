#!/usr/bin/env bash
# Trinity MCP server wrapper - routes to traceability server
set -euo pipefail

# Resolve repo root portably (no hardcoded user path).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

exec node scripts/mcp-traceability-server.js "$@"
