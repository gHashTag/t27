#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# t27/contrib/hil/smoke_test.sh
# Hardware-in-the-Loop (HIL) smoke test for QMTECH XC7A100T
# Prerequisites: openFPGALoader, bitstream built, FPGA connected via JTAG
# Usage: ./contrib/hil/smoke_test.sh [--cable CABLE] [--board BOARD]
# phi^2 + 1/phi^2 = 3 | TRINITY
set -euo pipefail

BOARD="${BOARD:-qmtech-a100t}"
CABLE="${CABLE:-ft2232}"
BITSTREAM="${BITSTREAM:-build/fpga/zerodsp_top.bit}"
UART_PORT="${UART_PORT:-/dev/cu.usbserial-1140}"
BAUD="${BAUD:-115200}"
TIMEOUT_SEC="${TIMEOUT_SEC:-30}"
PASS=0
FAIL=0

info()  { echo "[INFO]  $*"; }
pass()  { echo "[PASS]  $*"; PASS=$((PASS+1)); }
fail()  { echo "[FAIL]  $*"; FAIL=$((FAIL+1)); }

echo "=== T27 FPGA HIL Smoke Test ==="
echo "Board:      $BOARD"
echo "Bitstream:  $BITSTREAM"
echo "UART:       $UART_PORT @ $BAUD baud"
echo ""

if [ ! -f "$BITSTREAM" ]; then
    echo "[ERROR] Bitstream not found: $BITSTREAM"
    echo "Run: ./bootstrap/target/release/t27c fpga-build"
    exit 1
fi

info "Step 1: Detecting FPGA..."
if openFPGALoader --cable "$CABLE" --detect 2>&1 | grep -q "JTAG"; then
    pass "FPGA detected via JTAG ($CABLE)"
else
    fail "FPGA not detected (cable=$CABLE)"
    info "Try: --cable ft232RL / digilent_hs2 / xvc-client"
fi

info "Step 2: Loading bitstream to SRAM..."
if openFPGALoader --cable "$CABLE" --bitstream "$BITSTREAM" --freq 6000000 2>&1; then
    pass "Bitstream loaded successfully"
else
    fail "Bitstream loading failed"
    exit 1
fi

info "Step 3: Waiting for FPGA init (1s)..."
sleep 1

info "Step 4: Checking UART port..."
if [ -e "$UART_PORT" ]; then
    pass "UART port exists: $UART_PORT"
else
    fail "UART port not found: $UART_PORT"
fi

info "Step 5: Configuring UART..."
if stty -f "$UART_PORT" "$BAUD" raw -echo 2>/dev/null; then
    pass "UART configured at $BAUD baud"
else
    fail "UART configuration failed"
fi

info "Step 6: Sending test byte (0x55 = 'U')..."
if [ -e "$UART_PORT" ]; then
    printf '\x55' > "$UART_PORT" 2>/dev/null && pass "TX byte sent" || fail "TX failed"
fi

info "Step 7: Listening for heartbeat/echo (5s timeout)..."
if [ -e "$UART_PORT" ]; then
    RESPONSE=$(timeout 5 cat "$UART_PORT" 2>/dev/null | head -c 64 || true)
    if [ -n "$RESPONSE" ]; then
        pass "Received UART response: $(echo "$RESPONSE" | xxd | head -2)"
    else
        info "No UART response (expected if no echo firmware loaded)"
    fi
fi

echo ""
echo "=== HIL Smoke Test Results ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo ""
if [ "$FAIL" -eq 0 ]; then
    echo "ALL CHECKS PASSED"
    echo "LED heartbeat should be visible on LED[0]"
else
    echo "SOME CHECKS FAILED"
fi
echo ""
echo "phi^2 + 1/phi^2 = 3 | TRINITY"
