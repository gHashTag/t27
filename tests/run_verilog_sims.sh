#!/usr/bin/env bash
# Icarus Verilog simulation runner for FPGA testbenches
# Ring 46 | phi^2 + phi^-2 = 3 | TRINITY
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

BUILD_DIR="build/sim"
mkdir -p "$BUILD_DIR"

FAIL=0
TOTAL=0
SIM_TIMEOUT=60

echo "================================================================"
echo "  Icarus Verilog Testbench Simulation"
echo "================================================================"

# --- MAC Testbench ---
TOTAL=$((TOTAL + 1))
echo ""
echo "[SIM 1] MAC Testbench"
if iverilog -o "$BUILD_DIR/mac_tb" \
    gen/verilog/fpga/testbench/mac_tb.v \
    gen/verilog/fpga/mac.v 2>&1; then
    if timeout "$SIM_TIMEOUT" vvp "$BUILD_DIR/mac_tb" 2>&1; then
        echo "  [COMPILED + SIMULATED]"
    else
        echo "  [SIM TIMEOUT/ERROR]"
        FAIL=$((FAIL + 1))
    fi
else
    echo "  [COMPILE ERROR]"
    FAIL=$((FAIL + 1))
fi

# --- Top-Level Testbench ---
TOTAL=$((TOTAL + 1))
echo ""
echo "[SIM 2] Top-Level Testbench"
if iverilog -o "$BUILD_DIR/top_tb" \
    gen/verilog/fpga/testbench/top_tb.v 2>&1; then
    if timeout "$SIM_TIMEOUT" vvp "$BUILD_DIR/top_tb" 2>&1; then
        echo "  [COMPILED + SIMULATED]"
    else
        echo "  [SIM TIMEOUT/ERROR]"
        FAIL=$((FAIL + 1))
    fi
else
    echo "  [COMPILE ERROR]"
    FAIL=$((FAIL + 1))
fi

# --- UART Testbench ---
TOTAL=$((TOTAL + 1))
echo ""
echo "[SIM 3] UART Testbench"
if iverilog -o "$BUILD_DIR/uart_tb" \
    gen/verilog/fpga/testbench/uart_tb.v \
    gen/verilog/fpga/uart.v 2>&1; then
    if timeout "$SIM_TIMEOUT" vvp "$BUILD_DIR/uart_tb" 2>&1; then
        echo "  [COMPILED + SIMULATED]"
    else
        echo "  [SIM TIMEOUT/ERROR]"
        FAIL=$((FAIL + 1))
    fi
else
    echo "  [COMPILE ERROR]"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "================================================================"
echo "  Simulation Results: $((TOTAL - FAIL))/$TOTAL compiled and ran"
echo "================================================================"

# Clean up VCD files
rm -f *.vcd

if [ "$FAIL" -gt 0 ]; then
    echo "  STATUS: SOME SIMULATIONS FAILED"
    exit 1
else
    echo "  STATUS: ALL SIMULATIONS COMPLETED"
    exit 0
fi
