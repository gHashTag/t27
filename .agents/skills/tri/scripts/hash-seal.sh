#!/bin/bash
# hash-seal.sh - Compute hash seal for t27 PHI LOOP
#
# Usage: ./hash-seal.sh <spec-file>
#
# Outputs JSON with spec_hash_before/after, gen_hash_after, test_vector_hash

set -euo pipefail

SPEC_FILE="$1"

if [[ -z "$SPEC_FILE" ]]; then
    echo "Usage: $0 <spec-file>" >&2
    exit 1
fi

if [[ ! -f "$SPEC_FILE" ]]; then
    echo "Error: Spec file not found: $SPEC_FILE" >&2
    exit 1
fi

# Compute spec hash (current)
SPEC_HASH=$(sha256sum "$SPEC_FILE" | awk '{print $1}')

# Determine output path based on spec location
if [[ "$SPEC_FILE" == specs/base/* ]]; then
    GEN_PATH="backend/zig/base/$(basename "$SPEC_FILE" .t27).zig"
elif [[ "$SPEC_FILE" == specs/math/* ]]; then
    GEN_PATH="backend/zig/math/$(basename "$SPEC_FILE" .t27).zig"
elif [[ "$SPEC_FILE" == specs/numeric/* ]]; then
    GEN_PATH="backend/zig/numeric/$(basename "$SPEC_FILE" .t27).zig"
elif [[ "$SPEC_FILE" == specs/vsa/* ]]; then
    GEN_PATH="backend/zig/vsa/$(basename "$SPEC_FILE" .t27).zig"
elif [[ "$SPEC_FILE" == specs/orch/* ]]; then
    GEN_PATH="backend/zig/orch/$(basename "$SPEC_FILE" .t27).zig"
elif [[ "$SPEC_FILE" == specs/cli/* ]]; then
    GEN_PATH="backend/zig/cli/$(basename "$SPEC_FILE" .t27).zig"
else
    GEN_PATH="backend/zig/$(basename "$SPEC_FILE" .t27).zig"
fi

# Compute gen hash if exists
if [[ -f "$GEN_PATH" ]]; then
    GEN_HASH=$(sha256sum "$GEN_PATH" | awk '{print $1}')
else
    GEN_HASH="null"
fi

# Determine test vector path
SPEC_NAME=$(basename "$SPEC_FILE" .t27)
TEST_VECTOR_PATH="conformance/${SPEC_NAME}-conformance.json"

if [[ -f "$TEST_VECTOR_PATH" ]]; then
    TEST_VECTOR_HASH=$(sha256sum "$TEST_VECTOR_PATH" | awk '{print $1}')
else
    TEST_VECTOR_HASH="null"
fi

# Output JSON
cat <<EOF
{
  "spec_hash": "$SPEC_HASH",
  "gen_hash": $GEN_HASH,
  "test_vector_hash": $TEST_VECTOR_HASH,
  "spec_path": "$SPEC_FILE",
  "gen_path": "$GEN_PATH",
  "test_vector_path": "$TEST_VECTOR_PATH",
  "computed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
