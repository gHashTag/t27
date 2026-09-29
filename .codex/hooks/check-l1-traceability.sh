#!/usr/bin/env bash
# L1 TRACEABILITY gate — pointer, not a copy.
#
# Canonical implementation: .claude/hooks/check-l1-traceability.sh
# (-> `tri hooks l1-check`, logic in cli/tri/src/hooks.rs). A second copy of
# the rule here WILL drift — this file exists only so absolute-path harness
# wiring keeps resolving. Do not add logic here.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec bash "$REPO_ROOT/.claude/hooks/check-l1-traceability.sh" "$@"
