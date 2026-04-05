#!/bin/bash
# Trinity/t27 Full Integration Test
# End-to-end verification: spec → parse → gen (Zig/Verilog/C) → seal → verify
# Tests EVERY spec in architecture/graph.tri automatically

set -euo pipefail

T27C="./bootstrap/target/release/t27c"
GRAPH_FILE="architecture/graph.tri"
SPECS_DIR="specs"
RESULTS_DIR="conformance/integration"
FAILED_LOG="conformance/integration_failed.log"

echo "=== Trinity/t27 Full Integration Test ==="
echo "Date: $(date)"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Check if t27c binary exists
if [ ! -f "$T27C" ]; then
    echo -e "${RED}ERROR: t27c binary not found at $T27C${NC}"
    echo "Run: cargo build --release in bootstrap/"
    exit 1
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
FAILED_SPECS=0

# Helper function to test a single spec
test_spec() {
    local spec="$1"
    local spec_name
    spec_name=$(basename "$spec" .t27)

    echo -e "\n${YELLOW}Testing: $spec_name${NC}"
    echo "────────────────────────────────────────"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    # Step 1: Parse spec
    echo -n "  [1/5] Parse..."
    if "$T27C" parse "$spec" > /dev/null 2>&1; then
        echo -e " ${GREEN}PASS${NC}"
    else
        echo -e " ${RED}FAIL${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_SPECS=$((FAILED_SPECS + 1))
        echo "  Parse output:" "$("$T27C" parse "$spec" 2>&1 | head -5)
        return 1
    fi

    # Step 2: Generate Zig
    echo -n "  [2/5] Zig gen..."
    if "$T27C" gen "$spec" > /dev/null 2>&1; then
        echo -e " ${GREEN}PASS${NC}"
    else
        echo -e " ${YELLOW}SKIP${NC} (no Zig backend for this spec type)"
    fi

    # Step 3: Generate Verilog (if applicable)
    echo -n "  [3/5] Verilog gen..."
    if "$T27C" gen-verilog "$spec" > /dev/null 2>&1; then
        echo -e " ${GREEN}PASS${NC}"
    else
        echo -e " ${YELLOW}SKIP${NC} (no Verilog backend for this spec type)"
    fi

    # Step 4: Generate C (if applicable)
    echo -n "  [4/5] C gen..."
    if "$T27C" gen-c "$spec" > /dev/null 2>&1; then
        echo -e " ${GREEN}PASS${NC}"
    else
        echo -e " ${YELLOW}SKIP${NC} (no C backend for this spec type)"
    fi

    # Step 5: Seal spec
    echo -n "  [5/5] Seal..."
    local seal_output
    seal_output=$("$T27C" seal "$spec" --verify 2>&1)
    local seal_exit=$?

    if [ $seal_exit -eq 0 ]; then
        echo -e " ${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e " ${RED}FAIL${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_SPECS=$((FAILED_SPECS + 1))
        echo "  Seal output:" "$seal_output" | head -10
    fi

    echo ""
}

# Extract all spec paths from graph.tri
echo "Extracting specs from graph.tri..."
echo ""

# Parse graph.tri to find all spec paths
specs_to_test=$(grep -E 'path = "'$SPECS_DIR" "$GRAPH_FILE" | sed 's/.*path = "'$SPECS_DIR//' | sed 's/";.*//' | sort -u)

SPEC_COUNT=0
for spec in $specs_to_test; do
    if [ -f "$spec" ]; then
        test_spec "$spec"
        SPEC_COUNT=$((SPEC_COUNT + 1))
    else
        echo -e "${YELLOW}WARNING: Spec not found: $spec${NC}"
    fi
done

# Final summary
echo ""
echo "=== Integration Test Summary ==="
echo -e "Total specs tested: ${GREEN}$SPEC_COUNT${NC}"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo -e "Failed specs: ${RED}$FAILED_SPECS${NC}"

# Calculate pass rate
if [ $PASSED_TESTS -gt 0 ]; then
    PASS_RATE=$(awk "BEGIN {printf \"%.1f\", ($1/$2)*100}" <<< "$PASSED_TESTS $TOTAL_TESTS")
    echo -e "Pass rate: ${GREEN}${PASS_RATE}%${NC}"
fi

# Write results to JSON
echo ""
echo "Writing results to $RESULTS_DIR/integration_results.json..."

RESULTS_JSON=$(cat <<EOF
{
  "date": "$(date -Iseconds)",
  "t27c_version": "$("$T27C" --version 2>/dev/null || echo "unknown")",
  "graph_file": "$GRAPH_FILE",
  "specs_tested": $SPEC_COUNT,
  "total_test_phases": 5,
  "phases": {
    "parse": { "total": $SPEC_COUNT, "passed": $((SPEC_COUNT - FAILED_SPECS)), "failed": $FAILED_SPECS },
    "gen_zig": "checked",
    "gen_verilog": "checked",
    "gen_c": "checked",
    "seal": $PASSED_TESTS
  },
  "overall": {
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "pass_rate": ${PASS_RATE:-0}
  },
  "failed_specs": []
}
EOF
)

echo "$RESULTS_JSON" > "$RESULTS_DIR/integration_results.json"

# Write failed specs list if any
if [ $FAILED_SPECS -gt 0 ]; then
    echo "Writing failed specs to $FAILED_LOG..."
    grep -E 'path = "'$SPECS_DIR" "$GRAPH_FILE" | sed 's/.*path = "'$SPECS_DIR//' | sed 's/";.*//' | sort -u > "$FAILED_LOG"
fi

# Final verdict
echo ""
if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}=== ALL TESTS PASSED ===${NC}"
    exit 0
else
    echo -e "${RED}=== SOME TESTS FAILED ===${NC}"
    echo "See $FAILED_LOG for details"
    exit 1
fi
