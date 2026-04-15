#!/usr/bin/env bash
# Trinity Cognitive Probe Runner — Launcher Script
# Source: specs/benchmarks/trinity_cognitive_probe_runner.t27
#
# Prerequisites:
#   pip install anthropic openai
#   export ANTHROPIC_API_KEY=...
#   export OPENAI_API_KEY=...
#   export TOGETHER_API_KEY=...  (optional, for Llama)
#
# Usage:
#   ./scripts/benchmark/run_kaggle_probes.sh [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DATA_DIR="$REPO_ROOT/external/kaggle/data"
RUNNER="$REPO_ROOT/gen/benchmarks/trinity_probe_runner.py"

echo "============================================"
echo "Trinity Cognitive Probe Runner"
echo "phi^2 + 1/phi^2 = 3 = TRINITY"
echo "============================================"
echo ""
echo "Data dir: $DATA_DIR"
echo "Runner:   $RUNNER"
echo ""

# Prepare data directory with correct filenames
WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

cp "$DATA_DIR/thlp_mc_new.csv" "$WORK_DIR/"
cp "$DATA_DIR/ttm_mc_new.csv" "$WORK_DIR/ttm_mc_v5.csv"
cp "$DATA_DIR/tagp_mc.csv" "$WORK_DIR/"
cp "$DATA_DIR/tefb_mc_new.csv" "$WORK_DIR/"
cp "$DATA_DIR/tscp_mc_new.csv" "$WORK_DIR/tscp_mc_v5.csv"

echo "Prepared data in $WORK_DIR"
ls -la "$WORK_DIR"
echo ""

# Run benchmark
python3 "$RUNNER" \
    --data-dir "$WORK_DIR" \
    --sample-size 100 \
    --output-dir "$REPO_ROOT/outputs/benchmark" \
    "$@"

echo ""
echo "Done. Results in $REPO_ROOT/outputs/benchmark/"
