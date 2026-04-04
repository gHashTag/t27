#!/bin/bash
# Validates that all gen/ files have proper headers
set -e
echo "=== Gen Header Validation ==="

PASS=0; FAIL=0

for f in gen/zig/**/*.zig gen/c/**/*.c gen/c/**/*.h gen/verilog/**/*.v; do
    [ -f "$f" ] || continue
    if head -3 "$f" | grep -q "Auto-generated\|DO NOT EDIT\|TRINITY"; then
        PASS=$((PASS+1))
    else
        echo "FAIL: $f missing required header"
        FAIL=$((FAIL+1))
    fi
done

echo "Gen files: $((PASS+FAIL)) total, $PASS valid headers, $FAIL missing"
if [ $FAIL -eq 0 ]; then
    echo "ALL GEN HEADERS VALID"
    exit 0
else
    echo "HEADER FAILURES DETECTED"
    exit 1
fi
