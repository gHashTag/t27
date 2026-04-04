#!/usr/bin/env bash
# Ring-15 Integration Test: Full test suite — all specs gen + seal
set -euo pipefail

T27C="${T27C:-./bootstrap/target/release/t27c}"

echo "=== Ring-15: Full test suite — all specs gen + seal ==="

# 1. Gen all specs
echo "--- Generating all specs ---"
FAIL=0
TOTAL=0
for f in specs/**/*.t27; do
    TOTAL=$((TOTAL+1))
    if ! timeout 10 "$T27C" gen "$f" > /dev/null 2>&1; then
        echo "FAIL gen: $f"
        FAIL=$((FAIL+1))
    fi
done
echo "Specs gen: $FAIL/$TOTAL failed"
if [ "$FAIL" -ne 0 ]; then
    echo "FAIL: $FAIL spec files failed gen"
    exit 1
fi
echo "PASS: all $TOTAL specs gen OK"

# 2. Gen all compiler specs
echo "--- Generating all compiler specs ---"
CFAIL=0
CTOTAL=0
for f in compiler/**/*.t27; do
    CTOTAL=$((CTOTAL+1))
    if ! timeout 10 "$T27C" gen "$f" > /dev/null 2>&1; then
        echo "FAIL gen: $f"
        CFAIL=$((CFAIL+1))
    fi
done
echo "Compiler specs gen: $CFAIL/$CTOTAL failed"
if [ "$CFAIL" -ne 0 ]; then
    echo "FAIL: $CFAIL compiler spec files failed gen"
    exit 1
fi
echo "PASS: all $CTOTAL compiler specs gen OK"

# 3. Seal all specs
echo "--- Sealing all specs ---"
SFAIL=0
STOTAL=0
for f in specs/**/*.t27; do
    STOTAL=$((STOTAL+1))
    if ! "$T27C" seal "$f" --save > /dev/null 2>&1; then
        echo "FAIL seal: $f"
        SFAIL=$((SFAIL+1))
    fi
done
echo "Specs seal: $SFAIL/$STOTAL failed"
if [ "$SFAIL" -ne 0 ]; then
    echo "FAIL: $SFAIL spec files failed seal"
    exit 1
fi
echo "PASS: all $STOTAL specs sealed OK"

# 4. Seal all compiler specs
echo "--- Sealing all compiler specs ---"
CSFAIL=0
CSTOTAL=0
for f in compiler/**/*.t27; do
    CSTOTAL=$((CSTOTAL+1))
    if ! "$T27C" seal "$f" --save > /dev/null 2>&1; then
        echo "FAIL seal: $f"
        CSFAIL=$((CSFAIL+1))
    fi
done
echo "Compiler specs seal: $CSFAIL/$CSTOTAL failed"
if [ "$CSFAIL" -ne 0 ]; then
    echo "FAIL: $CSFAIL compiler spec files failed seal"
    exit 1
fi
echo "PASS: all $CSTOTAL compiler specs sealed OK"

# Summary
GRAND_TOTAL=$((TOTAL+CTOTAL))
echo ""
echo "=== Ring-15 COMPLETE ==="
echo "Total: $GRAND_TOTAL files gen + seal OK"
echo "BRANCH LAYER COMPLETE"
