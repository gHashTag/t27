#!/usr/bin/env bash
# PHI LOOP Stacked Branches Creator
# Creates all 9 phases of PHI LOOP as GitButler stacked branches

set -euo pipefail

# ANSI colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
RING_NUMBER="${1:-}"
ISSUE_NUMBER="${2:-}"

# GitButler CLI path
GITBUTLER_CLI="/private/var/folders/cm/2n1qdh892xldd1rc2ly1jv8r0000gn/T/AppTranslocation/9D544204-8A1B-4235-9673-46E56EA72367/d/GitButler.app/Contents/MacOS/gitbutler-tauri"

# Usage
usage() {
    echo -e "${BLUE}PHI LOOP Stacked Branches Creator${NC}"
    echo ""
    echo "Usage: $0 <ring-number> <issue-number>"
    echo ""
    echo "Example:"
    echo "  $0 32 42"
    echo ""
    echo "Creates 9 stacked branches for Ring 32, referencing Issue #42"
    echo ""
    echo "PHI LOOP Phases:"
    echo "  1. ring-NNN-issue    - Define the problem"
    echo "  2. ring-NNN-spec     - Write .t27 specifications"
    echo "  3. ring-NNN-tdd      - Write tests"
    echo "  4. ring-NNN-code     - Implement feature"
    echo "  5. ring-NNN-gen      - Generate code from specs"
    echo "  6. ring-NNN-seal     - Create verification seals"
    echo "  7. ring-NNN-verify   - Verify conformance"
    echo "  8. ring-NNN-land     - Land to main branch"
    echo "  9. ring-NNN-learn    - Document learnings"
    echo ""
    exit 1
}

# Validate inputs
validate_inputs() {
    if [ -z "$RING_NUMBER" ] || [ -z "$ISSUE_NUMBER" ]; then
        echo -e "${RED}Error: Ring number and issue number are required${NC}"
        usage
    fi

    if ! [[ "$RING_NUMBER" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}Error: Ring number must be numeric${NC}"
        exit 1
    fi

    if ! [[ "$ISSUE_NUMBER" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}Error: Issue number must be numeric${NC}"
        exit 1
    fi

    # Check GitButler CLI exists
    if [ ! -x "$GITBUTLER_CLI" ]; then
        echo -e "${RED}Error: GitButler CLI not found${NC}"
        echo "Expected path: $GITBUTLER_CLI"
        exit 1
    fi
}

# Create branch with parent
create_branch() {
    local name=$1
    local parent=$2
    local description=$3

    echo -e "${BLUE}Creating branch:${NC} $name (from: $parent)"
    echo "  $description"

    if ! $GITBUTLER_CLI branch create "$name" --from "$parent" 2>/dev/null; then
        echo -e "${YELLOW}  Warning: Branch may already exist${NC}"
    fi

    echo -e "${GREEN}  ✓ Created${NC}"
    echo ""
}

# Main execution
main() {
    validate_inputs

    # Pad ring number to 3 digits
    PADDED_RING=$(printf "%03d" "$RING_NUMBER")

    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}PHI LOOP Stacked Branches Creator${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
    echo "Ring Number: $RING_NUMBER (padded: $PADDED_RING)"
    echo "Issue Reference: #$ISSUE_NUMBER"
    echo "Issue Reference for commits: Closes #${ISSUE_NUMBER}"
    echo ""

    # Phase 1: Issue
    create_branch \
        "ring-${PADDED_RING}-issue" \
        "dev" \
        "Phase 1: Define the problem and create GitHub issue"

    # Phase 2: Spec
    create_branch \
        "ring-${PADDED_RING}-spec" \
        "ring-${PADDED_RING}-issue" \
        "Phase 2: Write .t27 specifications (L2 GENERATION)"

    # Phase 3: TDD
    create_branch \
        "ring-${PADDED_RING}-tdd" \
        "ring-${PADDED_RING}-spec" \
        "Phase 3: Write tests before implementation (L4 TESTABILITY)"

    # Phase 4: Code
    create_branch \
        "ring-${PADDED_RING}-code" \
        "ring-${PADDED_RING}-tdd" \
        "Phase 4: Implement the feature in .t27 specs"

    # Phase 5: Gen
    create_branch \
        "ring-${PADDED_RING}-gen" \
        "ring-${PADDED_RING}-code" \
        "Phase 5: Generate code from specs (L2 GENERATION)"

    # Phase 6: Seal
    create_branch \
        "ring-${PADDED_RING}-seal" \
        "ring-${PADDED_RING}-gen" \
        "Phase 6: Create verification seals (L6 CEILING)"

    # Phase 7: Verify
    create_branch \
        "ring-${PADDED_RING}-verify" \
        "ring-${PADDED_RING}-seal" \
        "Phase 7: Verify conformance"

    # Phase 8: Land
    create_branch \
        "ring-${PADDED_RING}-land" \
        "ring-${PADDED_RING}-verify" \
        "Phase 8: Land to main branch"

    # Phase 9: Learn
    create_branch \
        "ring-${PADDED_RING}-learn" \
        "ring-${PADDED_RING}-land" \
        "Phase 9: Document learnings"

    # Summary
    echo -e "${GREEN}======================================${NC}"
    echo -e "${GREEN}PHI LOOP Stacked Branches Created!${NC}"
    echo -e "${GREEN}======================================${NC}"
    echo ""
    echo "Stacked Branches:"
    echo "  1. ring-${PADDED_RING}-issue   ← Starting point"
    echo "  2. ring-${PADDED_RING}-spec    ← depends on issue"
    echo "  3. ring-${PADDED_RING}-tdd     ← depends on spec"
    echo "  4. ring-${PADDED_RING}-code    ← depends on tdd"
    echo "  5. ring-${PADDED_RING}-gen     ← depends on code"
    echo "  6. ring-${PADDED_RING}-seal    ← depends on gen"
    echo "  7. ring-${PADDED_RING}-verify  ← depends on seal"
    echo "  8. ring-${PADDED_RING}-land    ← depends on verify"
    echo "  9. ring-${PADDED_RING}-learn   ← depends on land"
    echo ""
    echo "Next Steps:"
    echo "  1. Apply first phase: ${GITBUTLER_CLI} apply ring-${PADDED_RING}-issue"
    echo "  2. Create GitHub issue: #$ISSUE_NUMBER"
    echo "  3. Work through each phase sequentially"
    echo "  4. Use '${GITBUTLER_CLI} rub <source> <target>' to move changes between phases"
    echo ""
    echo "Documentation: docs/phi-loop-stacked-branches.md"
    echo ""
    echo -e "${BLUE}φ² + φ⁻² = 3 | TRINITY${NC}"
}

# Run main
main "$@"
