#!/bin/bash
# Verification script for bibliography files
# Returns JSON with bib entry counts

if [ -z "$1" ]; then
    echo '{"error": "Usage: verify-bibliography.sh <bib-file.bib>"}'
    exit 1
fi

BIB_FILE="$1"

# Count @ entries (BibTeX entries start with @)
ENTRIES=$(grep -c '^@' "$BIB_FILE" 2>/dev/null || echo 0)

# Count entry types by category
BOOKS=$(grep -c '^@book' "$BIB_FILE" 2>/dev/null || echo 0)
ARTICLES=$(grep -c '^@article' "$BIB_FILE" 2>/dev/null || echo 0)
INPROCEEDINGS=$(grep -c '^@inproceedings' "$BIB_FILE" 2>/dev/null || echo 0)
OTHERS=$((ENTRIES - BOOKS - ARTICLES - INPROCEEDINGS))

# Check for common BibTeX errors
MISSING_KEYS=$(grep -c '^@.*{$' "$BIB_FILE" 2>/dev/null || echo 0)
DUPLICATES=$(sort "$BIB_FILE" | uniq -d | wc -l | tr -d ' ')

cat << EOF
{
  "file": "$(basename "$BIB_FILE")",
  "total_entries": $ENTRIES,
  "books": $BOOKS,
  "articles": $ARTICLES,
  "inproceedings": $INPROCEEDINGS,
  "other": $OTHERS,
  "missing_keys": $MISSING_KEYS,
  "duplicate_potential": $DUPLICATES,
  "status": $([ "$ENTRIES" -ge 10 ] && echo "VALID" || echo "MINIMAL")
}
EOF
