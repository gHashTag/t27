#!/bin/bash
# Verification script for LaTeX chapters
# Returns JSON with chapter metrics

if [ -z "$1" ]; then
    echo '{"error": "Usage: verify-chapter.sh <chapter-file.tex>"}'
    exit 1
fi

CHAPTER_FILE="$1"
CHAPTER_NAME=$(basename "$CHAPTER_FILE" .tex)

# Count lines
LINES=$(wc -l < "$CHAPTER_FILE" 2>/dev/null | tr -d ' ')

# Count theorems, lemmas, definitions, proofs
THEOREMS=$(grep -c '\\begin{theorem}' "$CHAPTER_FILE" 2>/dev/null || echo 0)
LEMMAS=$(grep -c '\\begin{lemma}' "$CHAPTER_FILE" 2>/dev/null || echo 0)
DEFINITIONS=$(grep -c '\\begin{definition}' "$CHAPTER_FILE" 2>/dev/null || echo 0)
PROOFS=$(grep -c '\\begin{proof}' "$CHAPTER_FILE" 2>/dev/null || echo 0)

# Count equations
EQUATIONS=$(grep -c '\\begin{equation}' "$CHAPTER_FILE" 2>/dev/null || echo 0)

# Determine status based on line count and theorems
if [ "$LINES" -ge 1500 ] && [ "$THEOREMS" -ge 5 ]; then
    STATUS="COMPLETE"
elif [ "$LINES" -ge 800 ]; then
    STATUS="PARTIAL"
else
    STATUS="TODO"
fi

cat << EOF
{
  "chapter": "$CHAPTER_NAME",
  "lines": $LINES,
  "theorems": $THEOREMS,
  "lemmas": $LEMMAS,
  "definitions": $DEFINITIONS,
  "equations": $EQUATIONS,
  "proofs": $PROOFS,
  "status": "$STATUS"
}
EOF
