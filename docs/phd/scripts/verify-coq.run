#!/bin/bash
# Verification script for Coq proof files
# Returns JSON with Coq compilation status

COQ_DIR="/Users/playra/trios/docs/phd/coq"

if [ ! -d "$COQ_DIR" ]; then
    cat << EOF
{
  "coq_directory": "$COQ_DIR",
  "status": "NOT_FOUND",
  "message": "Coq directory does not exist"
}
EOF
    exit 0
fi

# Count .v files
V_FILES=$(find "$COQ_DIR" -name "*.v" 2>/dev/null | wc -l | tr -d ' ')
VO_FILES=$(find "$COQ_DIR" -name "*.vo" 2>/dev/null | wc -l | tr -d ' ')

# Count theorems across all .v files
THEOREMS=$(find "$COQ_DIR" -name "*.v" -exec grep -h '\\Theorem' {} \; | wc -l | tr -d ' ')
LEMMAS=$(find "$COQ_DIR" -name "*.v" -exec grep -h '\\Lemma' {} \; | wc -l | tr -d ' ')

# Try to compile if coqc is available
if command -v coqc &> /dev/null; then
    cd "$COQ_DIR"
    COMPILE_ERRORS=0
    for f in *.v; do
        if [ -f "$f" ]; then
            OUTPUT=$(coqc -R . "$f" 2>&1)
            if echo "$OUTPUT" | grep -q "Error"; then
                COMPILE_ERRORS=$((COMPILE_ERRORS + 1))
            fi
        fi
    done

    cat << EOF
{
  "v_files": $V_FILES,
  "vo_files": $VO_FILES,
  "theorems": $THEOREMS,
  "lemmas": $LEMMAS,
  "compile_errors": $COMPILE_ERRORS,
  "status": $([ "$V_FILES" -eq "$VO_FILES" ] && [ "$COMPILE_ERRORS" -eq 0 ] && echo "COMPILED" || echo "NEEDS_WORK")
}
EOF
else
    cat << EOF
{
  "v_files": $V_FILES,
  "vo_files": $VO_FILES,
  "theorems": $THEOREMS,
  "lemmas": $LEMMAS,
  "coqc_available": false,
  "status": "COQC_NOT_INSTALLED"
}
EOF
fi
