#!/bin/bash
# Master verification script for all dissertation deliverables

cd "/Users/playra/trios/docs/phd"

# Get metrics
CHAPTER_COUNT=$(find chapters -name "*.tex" 2>/dev/null | wc -l | tr -d ' ')
CHAPTER_LINES=$(find chapters -name "*.tex" -exec wc -l {} + 2>/dev/null | awk '{s+=$1} END {print s}')
BIB_ENTRIES=$(grep -c '^@' bibliography.bib 2>/dev/null || echo 0)
COQ_V=$(find coq -name "*.v" 2>/dev/null | wc -l | tr -d ' ' || echo 0)
COQ_VO=$(find coq -name "*.vo" 2>/dev/null | wc -l | tr -d ' ' || echo 0)

# Get monograph info
if [ -f "monograph.pdf" ]; then
    PDF_EXISTS="true"
    PDF_PAGES=$(pdfinfo monograph.pdf 2>/dev/null | grep "Pages:" | awk '{print $2}')
    PDF_SIZE=$(ls -l monograph.pdf | awk '{print $5}')
else
    PDF_EXISTS="false"
    PDF_PAGES=0
    PDF_SIZE=0
fi

# Calculate target progress (rough estimate)
# 300 pages * 2000 lines/page = 600,000 target lines
# 300-800 bib entries, use 550 as mid
# 84 Coq theorems
TARGET_LINES=600000
CURRENT_PROGRESS=$(awk "BEGIN {p=$CHAPTER_LINES; b=$BIB_ENTRIES; c=$COQ_VO; t=$TARGET_LINES; printf \"%.2f\", (p*10 + b*10 + c*20) / t * 100}")

cat << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "monograph": {
    "exists": $PDF_EXISTS,
    "pages": $PDF_PAGES,
    "size_bytes": $PDF_SIZE
  },
  "chapters": {
    "file_count": $CHAPTER_COUNT,
    "total_lines": $CHAPTER_LINES
  },
  "bibliography": {
    "entries": $BIB_ENTRIES,
    "target": "300-800"
  },
  "coq": {
    "v_files": $COQ_V,
    "vo_files": $COQ_VO,
    "target_theorems": 84
  },
  "figures": {
    "directory_exists": $( [ -d figures ] && echo true || echo false),
    "file_count": $(find figures -type f 2>/dev/null | wc -l | tr -d ' ' || echo 0)
  },
  "targets": {
    "pages": 300,
    "chapters_lines": 600000,
    "bibliography_entries_min": 300,
    "bibliography_entries_max": 800,
    "coq_theorems": 84,
    "figures_min": 40
  },
  "progress": {
    "estimated_percent": $CURRENT_PROGRESS,
    "status": "IN_PROGRESS"
  }
}
EOF
