#!/usr/bin/env bash
# Ring-12 Integration Test: seal --save / --verify
set -euo pipefail

T27C="${T27C:-./bootstrap/target/debug/t27c}"
SPEC="specs/base/types.t27"
SEAL_FILE=".trinity/seals/tritype-base.json"

echo "=== Ring-12: seal --save / --verify test ==="

# Clean previous seal
rm -f "$SEAL_FILE"

# 1. Default seal (print only)
echo "--- t27c seal (print) ---"
OUTPUT=$("$T27C" seal "$SPEC")
echo "$OUTPUT"
echo "$OUTPUT" | grep -q "spec_hash=sha256:" || { echo "FAIL: missing spec_hash"; exit 1; }
echo "$OUTPUT" | grep -q "gen_hash_zig=sha256:" || { echo "FAIL: missing gen_hash_zig"; exit 1; }
echo "$OUTPUT" | grep -q "gen_hash_verilog=sha256:" || { echo "FAIL: missing gen_hash_verilog"; exit 1; }
echo "$OUTPUT" | grep -q "gen_hash_c=sha256:" || { echo "FAIL: missing gen_hash_c"; exit 1; }
echo "PASS: default seal prints all hashes"

# 2. Verify fails without saved seal
echo "--- t27c seal --verify (no saved seal) ---"
if "$T27C" seal "$SPEC" --verify 2>&1; then
    echo "FAIL: --verify should fail without saved seal"
    exit 1
fi
echo "PASS: --verify fails without saved seal"

# 3. Save seal
echo "--- t27c seal --save ---"
"$T27C" seal "$SPEC" --save
test -f "$SEAL_FILE" || { echo "FAIL: seal file not created"; exit 1; }
echo "PASS: seal saved to $SEAL_FILE"

# 4. Verify matches
echo "--- t27c seal --verify ---"
OUTPUT=$("$T27C" seal "$SPEC" --verify)
echo "$OUTPUT"
echo "$OUTPUT" | grep -q "all hashes MATCH" || { echo "FAIL: verification should match"; exit 1; }
echo "PASS: all hashes MATCH"

# 5. Check JSON format
echo "--- Checking seal JSON format ---"
python3 -c "
import json, sys
with open('$SEAL_FILE') as f:
    d = json.load(f)
required = ['module', 'spec_path', 'spec_hash', 'gen_hash_zig', 'gen_hash_verilog', 'gen_hash_c', 'sealed_at', 'ring']
for k in required:
    assert k in d, f'Missing key: {k}'
assert d['ring'] == 12, f'Wrong ring: {d[\"ring\"]}'
assert d['module'] == 'tritype-base', f'Wrong module: {d[\"module\"]}'
print('JSON format valid')
"
echo "PASS: seal JSON has all required fields"

echo ""
echo "=== ALL RING-12 TESTS PASSED ==="
