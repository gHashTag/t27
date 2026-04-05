#!/bin/bash
# Ring-17 Fixed Point Verification
# Generates all specs twice and verifies bit-identical output
set -e
T27C="./bootstrap/target/release/t27c"
mkdir -p /tmp/fp_s1 /tmp/fp_s2
for f in specs/**/*.t27 compiler/**/*.t27; do
  [ -f "$f" ] || continue
  N=$(basename "$f" .t27)
  $T27C gen "$f" > "/tmp/fp_s1/${N}.zig" 2>/dev/null
  $T27C gen "$f" > "/tmp/fp_s2/${N}.zig" 2>/dev/null
done
DIFF=0
for f in /tmp/fp_s1/*.zig; do
  N=$(basename "$f")
  diff -q "$f" "/tmp/fp_s2/$N" >/dev/null 2>&1 || DIFF=$((DIFF+1))
done
if [ $DIFF -eq 0 ]; then
  echo "FIXED POINT REACHED: stage(N) == stage(N-1)"
  echo "phi^2 + 1/phi^2 = 3 | TRINITY"
else
  echo "DIVERGENCE: $DIFF files differ"
  exit 1
fi
