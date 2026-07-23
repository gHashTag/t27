#!/bin/bash
# toxic-verdict.sh - Run toxic regression check
#
# Usage: ./toxic-verdict.sh [--sacred|--format <format>]

set -euo pipefail

MODE="${1:---default}"
FORMAT=""

case "$MODE" in
    --sacred)
        echo "Running sacred physics conformance check..."
        tri test --sacred
        tri verdict --sacred
        ;;
    --format)
        FORMAT="$2"
        if [[ -z "$FORMAT" ]]; then
            echo "Error: --format requires argument" >&2
            exit 1
        fi
        echo "Running toxic check for format: $FORMAT"
        tri gen --format "$FORMAT"
        tri test --format "$FORMAT"
        tri verdict --toxic
        ;;
    --module)
        MODULE="$2"
        if [[ -z "$MODULE" ]]; then
            echo "Error: --module requires argument" >&2
            exit 1
        fi
        echo "Running toxic check for module: $MODULE"
        tri gen --module "$MODULE"
        tri test --module "$MODULE"
        tri verdict --toxic
        ;;
    --all)
        echo "Running full toxic regression check..."
        tri gen
        tri test
        tri verdict --toxic --full
        ;;
    --default)
        echo "Running standard toxic check..."
        tri verdict --toxic
        ;;
    *)
        echo "Usage: $0 [--sacred|--format <fmt|--module <mod|--all]" >&2
        exit 1
        ;;
esac

# Capture exit code
VERDICT_CODE=$?

if [[ $VERDICT_CODE -eq 0 ]]; then
    echo ""
    echo "✓ Verdict: CLEAN"
    echo "  No toxic regressions detected"
    exit 0
else
    echo ""
    echo "✗ Verdict: TOXIC"
    echo "  Toxic regressions detected"
    echo "  Rollback required"
    echo ""
    echo "Use 'tri experience record --mistake' to record failure"
    echo "Use 'git restore <spec>' to rollback spec (NOT generated code)"
    exit 1
fi
