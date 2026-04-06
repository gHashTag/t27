#!/usr/bin/env bash
# validate-conformance-v2.sh — Validate conformance vectors against SCHEMA_V2
# φ² + 1/φ² = 3 | TRINITY

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SCHEMA="$REPO_ROOT/conformance/SCHEMA_V2.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq not found. Please install jq to run validation.${NC}"
    echo "  macOS: brew install jq"
    echo "  ubuntu/debian: sudo apt-get install jq"
    exit 1
fi

# Check if ajv or similar JSON schema validator is available
HAS_VALIDATOR=false
VALIDATOR_CMD=""

if command -v ajv &> /dev/null; then
    HAS_VALIDATOR=true
    VALIDATOR_CMD="ajv validate -s $SCHEMA -d"
elif command -v check-jsonschema &> /dev/null; then
    HAS_VALIDATOR=true
    VALIDATOR_CMD="check-jsonschema $SCHEMA"
fi

# Fallback: basic structural validation with jq
validate_with_jq() {
    local file="$1"
    local errors=0

    # Check required fields
    for field in schema_version format_family vector_name verdict seal test_vectors; do
        if ! jq -e ".\"$field\"" "$file" > /dev/null 2>&1; then
            echo -e "${RED}✗ Missing required field: $field${NC}" >&2
            ((errors++))
        fi
    done

    # Check schema_version == 2
    local sv=$(jq -r '.schema_version // "null"' "$file")
    if [ "$sv" != "2" ]; then
        echo -e "${RED}✗ schema_version must be 2, got: $sv${NC}" >&2
        ((errors++))
    fi

    # Check verdict is valid
    local verdict=$(jq -r '.verdict // "null"' "$file")
    if [[ ! "$verdict" =~ ^(CLEAN|FAIL|PARTIAL|SKIP)$ ]]; then
        echo -e "${RED}✗ Invalid verdict: $verdict${NC}" >&2
        ((errors++))
    fi

    # Check seal format
    local seal=$(jq -r '.seal // "null"' "$file")
    if [[ ! "$seal" =~ ^sha256:[0-9a-f]{64}$ ]]; then
        echo -e "${RED}✗ Invalid seal format${NC}" >&2
        ((errors++))
    fi

    # Check test_vectors array
    local tv_count=$(jq '.test_vectors | length' "$file")
    if [ "$tv_count" -eq 0 ]; then
        echo -e "${RED}✗ test_vectors array is empty${NC}" >&2
        ((errors++))
    fi

    # Check per-case verdicts
    local bad_verdicts=$(jq -r '.test_vectors[] | .verdict // "NULL"' "$file" 2>/dev/null | grep -v -E '^(CLEAN|FAIL|PARTIAL|SKIP)$' || true)
    if [ -n "$bad_verdicts" ]; then
        echo -e "${RED}✗ Invalid per-case verdict(s):$bad_verdicts${NC}" >&2
        ((errors++))
    fi

    return $errors
}

main() {
    local total_errors=0
    local total_files=0

    echo "=== T27 Conformance Validation (SCHEMA_V2) ==="
    echo "Schema: $SCHEMA"
    echo ""

    # Validate all JSON files in conformance/ except SCHEMA_V2.json itself
    while IFS= read -r -d '' file; do
        filename=$(basename "$file")

        # Skip the schema file itself
        if [ "$filename" = "SCHEMA_V2.json" ]; then
            continue
        fi

        # Skip if file doesn't have schema_version field (v1 files)
        if ! jq -e '.schema_version' "$file" > /dev/null 2>&1; then
            echo -e "${YELLOW}⊘ $filename (v1, not validated against v2)${NC}"
            continue
        fi

        ((total_files++))

        echo -n "Validating $filename ... "

        if [ "$HAS_VALIDATOR" = true ]; then
            if $VALIDATOR_CMD "$file" > /dev/null 2>&1; then
                echo -e "${GREEN}✓ PASS${NC}"
            else
                echo -e "${RED}✗ FAIL${NC}"
                ((total_errors++))
            fi
        else
            # validate_with_jq outputs errors to stderr, returns error count
            if validate_with_jq "$file" > /dev/null; then
                echo -e "${GREEN}✓ PASS${NC}"
            else
                local err_count=$?
                echo -e "${RED}✗ FAIL${NC}"
                ((total_errors++))
            fi
        fi
    done < <(find "$REPO_ROOT/conformance" -name "*.json" -print0)

    echo ""
    echo "=== Summary ==="
    echo "Files validated (v2): $total_files"
    echo "Total errors: $total_errors"

    if [ "$total_errors" -eq 0 ]; then
        echo -e "${GREEN}All v2 vectors are valid!${NC}"
        return 0
    else
        echo -e "${RED}Validation failed with $total_errors error(s)${NC}"
        return 1
    fi
}

main "$@"
