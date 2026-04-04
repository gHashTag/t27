#!/bin/bash
set -e
T27C="./bootstrap/target/release/t27c"

echo "=== Ring-9 Zig codegen validation ==="

# Test 1: trivial.t27
$T27C gen tests/ring0_trivial.t27 > /tmp/ring9_trivial.zig
echo "trivial.t27 → $(wc -l < /tmp/ring9_trivial.zig) lines of Zig"

# Verify no bad patterns in trivial output
if grep -q '@compileAssert' /tmp/ring9_trivial.zig; then
  echo "FAIL: trivial.zig contains @compileAssert"
  exit 1
fi

# Test 2: types.t27
$T27C gen specs/base/types.t27 > /tmp/ring9_types.zig
echo "types.t27 → $(wc -l < /tmp/ring9_types.zig) lines of Zig"

# Verify no known bad patterns
if grep -q '@compileAssert' /tmp/ring9_types.zig; then
  echo "FAIL: types.zig contains @compileAssert (not valid Zig)"
  exit 1
fi

if grep -qE 'switch \(.*\) \{' /tmp/ring9_types.zig && grep -qP '\.\d+ =>' /tmp/ring9_types.zig; then
  echo "FAIL: types.zig contains .N => pattern (should be N => for integers)"
  exit 1
fi

if grep -q 'for (0\.\.,' /tmp/ring9_types.zig; then
  echo "FAIL: types.zig contains broken for(0.., X) pattern"
  exit 1
fi

# Verify expected patterns exist
if ! grep -q 'const std = @import("std")' /tmp/ring9_types.zig; then
  echo "FAIL: types.zig missing std import"
  exit 1
fi

if ! grep -q 'pub const Trit = enum(i8)' /tmp/ring9_types.zig; then
  echo "FAIL: types.zig missing Trit enum"
  exit 1
fi

if ! grep -q 'pub fn trit_add' /tmp/ring9_types.zig; then
  echo "FAIL: types.zig missing trit_add function"
  exit 1
fi

echo "Ring-9 gen: PASS"
