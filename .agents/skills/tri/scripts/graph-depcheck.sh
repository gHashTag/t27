#!/bin/bash
# graph-depcheck.sh - Validate t27 canonical dependency graph
#
# Usage: ./graph-depcheck.sh [--check-cycles|--check-tiers|--check-all]
#
# This used to print a verdict it had not computed. `check_tiers` set
# `local violations=0`, never changed it, and printed "No forward tier
# dependencies detected"; `GRAPH_FILE` was assigned and never read; running the
# script from an empty directory produced byte-identical output. Meanwhile the
# graph carries 1 cycle and 5 tier-backward edges.
#
# It now delegates to tools/check_graph_law8.py, which reads
# architecture/graph_v2.json and refuses (exit 2) when it cannot. A local
# re-implementation answers the question once and then drifts away from it.

set -euo pipefail

MODE="${1:---check-all}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECKER="$ROOT/tools/check_graph_law8.py"

case "$MODE" in
  --check-cycles|--check-tiers|--check-all) ;;
  --check-sacred)
    echo "graph-depcheck: sacred-core tolerances are not checked here." >&2
    echo "graph-depcheck: nothing in this repository checks them; exit 2." >&2
    exit 2
    ;;
  *)
    echo "Usage: $0 [--check-cycles|--check-tiers|--check-sacred|--check-all]" >&2
    exit 1
    ;;
esac

if [[ ! -f "$CHECKER" ]]; then
  echo "graph-depcheck: $CHECKER is missing, so nothing was checked." >&2
  echo "graph-depcheck: exit 2 = could not run, not a clean graph." >&2
  exit 2
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "graph-depcheck: python3 is not on PATH, so nothing was checked." >&2
  exit 2
fi

# Both modes read the same graph and the checker prints both readings, so
# neither flag hides the other's answer.
exec python3 "$CHECKER"
