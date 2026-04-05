#!/usr/bin/env bash
# Verilator lint validation for all gen/verilog files
# Ring 45 | phi^2 + phi^-2 = 3 | TRINITY
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

FAIL=0
PASS=0
TOTAL=0

echo "================================================================"
echo "  Verilator Lint Validation"
echo "================================================================"

# Lint non-testbench files individually
for f in $(find gen/verilog -name '*.v' ! -path '*/testbench/*' ! -name 'top_level.v' | sort); do
    TOTAL=$((TOTAL + 1))
    if verilator --lint-only -Wall "$f" >/dev/null 2>&1; then
        PASS=$((PASS + 1))
    else
        echo "  [FAIL] $f"
        verilator --lint-only -Wall "$f" 2>&1 | head -5
        FAIL=$((FAIL + 1))
    fi
done

# Lint top_level.v with submodule dependencies
TOTAL=$((TOTAL + 1))
if verilator --lint-only -Wall --top-module trinity_fpga_top \
    gen/verilog/fpga/top_level.v \
    gen/verilog/fpga/uart.v \
    gen/verilog/fpga/spi.v \
    gen/verilog/fpga/mac.v >/dev/null 2>&1; then
    PASS=$((PASS + 1))
else
    echo "  [FAIL] gen/verilog/fpga/top_level.v"
    FAIL=$((FAIL + 1))
fi

# Lint testbench files with --timing and submodule deps
for tb_info in \
    "gen/verilog/fpga/testbench/mac_tb.v:gen/verilog/fpga/mac.v" \
    "gen/verilog/fpga/testbench/top_tb.v:" \
    "gen/verilog/fpga/testbench/uart_tb.v:gen/verilog/fpga/uart.v"; do
    tb="${tb_info%%:*}"
    deps="${tb_info#*:}"
    TOTAL=$((TOTAL + 1))
    if verilator --lint-only -Wall --timing $tb $deps >/dev/null 2>&1; then
        PASS=$((PASS + 1))
    else
        echo "  [FAIL] $tb"
        verilator --lint-only -Wall --timing $tb $deps 2>&1 | head -5
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "================================================================"
echo "  Results: $PASS/$TOTAL passed, $FAIL failed"
echo "================================================================"

if [ "$FAIL" -gt 0 ]; then
    echo "  STATUS: LINT FAILURES DETECTED"
    exit 1
else
    echo "  STATUS: ALL LINT CHECKS PASSED"
    exit 0
fi
