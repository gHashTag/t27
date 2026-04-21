#!/bin/bash
# Verification script for figures
# Returns JSON with figure counts

FIGURES_DIR="/Users/playra/trios/docs/phd/figures"

if [ ! -d "$FIGURES_DIR" ]; then
    cat << EOF
{
  "figures_directory": "$FIGURES_DIR",
  "status": "NOT_FOUND",
  "message": "Figures directory does not exist"
}
EOF
    exit 0
fi

# Count figure files by type
TIKZ_FILES=$(find "$FIGURES_DIR" -name "*.tikz" 2>/dev/null | wc -l | tr -d ' ')
PDF_FILES=$(find "$FIGURES_DIR" -name "*.pdf" 2>/dev/null | wc -l | tr -d ' ')
PNG_FILES=$(find "$FIGURES_DIR" -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
SVG_FILES=$(find "$FIGURES_DIR" -name "*.svg" 2>/dev/null | wc -l | tr -d ' ')

TOTAL=$((TIKZ_FILES + PDF_FILES + PNG_FILES + SVG_FILES))

cat << EOF
{
  "tikz": $TIKZ_FILES,
  "pdf": $PDF_FILES,
  "png": $PNG_FILES,
  "svg": $SVG_FILES,
  "total": $TOTAL,
  "status": $([ "$TOTAL" -ge 5 ] && echo "ADEQUATE" || echo "INSUFFICIENT")
}
EOF
