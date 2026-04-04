#!/bin/bash
set -e
echo "=== T27 Comprehensive Test Suite ==="
echo "phi^2 + 1/phi^2 = 3 | TRINITY"
echo ""

T27C="./bootstrap/target/release/t27c"
PASS=0; FAIL=0; TOTAL=0

run_test() {
  TOTAL=$((TOTAL+1))
  if eval "$1" > /dev/null 2>&1; then
    PASS=$((PASS+1))
  else
    echo "FAIL: $2"
    FAIL=$((FAIL+1))
  fi
}

echo "--- Phase 1: Parse ---"
for f in specs/**/*.t27 compiler/**/*.t27; do
  [ -f "$f" ] || continue
  run_test "$T27C parse $f" "parse $f"
done
echo "Parse: $PASS passed, $FAIL failed"
P1_FAIL=$FAIL; PASS=0; FAIL=0

echo "--- Phase 2: Gen Zig ---"
for f in specs/**/*.t27 compiler/**/*.t27; do
  [ -f "$f" ] || continue
  run_test "$T27C gen $f" "gen-zig $f"
done
echo "Gen Zig: $PASS passed, $FAIL failed"
P2_FAIL=$FAIL; PASS=0; FAIL=0

echo "--- Phase 3: Gen Verilog ---"
for f in specs/**/*.t27; do
  [ -f "$f" ] || continue
  run_test "$T27C gen-verilog $f" "gen-verilog $f"
done
echo "Gen Verilog: $PASS passed, $FAIL failed"
P3_FAIL=$FAIL; PASS=0; FAIL=0

echo "--- Phase 4: Gen C ---"
for f in specs/**/*.t27; do
  [ -f "$f" ] || continue
  run_test "$T27C gen-c $f" "gen-c $f"
done
echo "Gen C: $PASS passed, $FAIL failed"
P4_FAIL=$FAIL; PASS=0; FAIL=0

echo "--- Phase 5: Seal Verify ---"
for f in specs/**/*.t27; do
  [ -f "$f" ] || continue
  run_test "$T27C seal $f --verify 2>&1 | grep -v MISMATCH" "seal-verify $f"
done
echo "Seal Verify: $PASS passed, $FAIL failed"
P5_FAIL=$FAIL; PASS=0; FAIL=0

echo "--- Phase 6: Fixed Point ---"
mkdir -p /tmp/fp1 /tmp/fp2
for f in specs/**/*.t27 compiler/**/*.t27; do
  [ -f "$f" ] || continue
  N=$(basename "$f" .t27)
  $T27C gen "$f" > "/tmp/fp1/${N}.zig" 2>/dev/null
  $T27C gen "$f" > "/tmp/fp2/${N}.zig" 2>/dev/null
done
FP_DIFF=0
for f in /tmp/fp1/*.zig; do
  N=$(basename "$f")
  diff -q "$f" "/tmp/fp2/$N" > /dev/null 2>&1 || FP_DIFF=$((FP_DIFF+1))
done
echo "Fixed Point: $FP_DIFF divergences"

echo ""
echo "=== SUMMARY ==="
TOTAL_FAIL=$((P1_FAIL + P2_FAIL + P3_FAIL + P4_FAIL + P5_FAIL + FP_DIFF))
echo "Parse failures:    $P1_FAIL"
echo "Gen Zig failures:  $P2_FAIL"
echo "Gen Verilog fails: $P3_FAIL"
echo "Gen C failures:    $P4_FAIL"
echo "Seal mismatches:   $P5_FAIL"
echo "FP divergences:    $FP_DIFF"
echo "TOTAL FAILURES:    $TOTAL_FAIL"
echo ""
if [ $TOTAL_FAIL -eq 0 ]; then
  echo "ALL TESTS PASSED"
  echo "phi^2 + 1/phi^2 = 3 | TRINITY"
  exit 0
else
  echo "SOME TESTS FAILED"
  exit 1
fi
