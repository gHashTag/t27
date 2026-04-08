#!/bin/bash
set -e
cd "$(dirname "$0")"
pandoc gf_paper_v3_imrad_draft.md \
  -o paper_from_md.pdf \
  --pdf-engine=xelatex \
  --citeproc \
  --bibliography=latex/references.bib \
  --metadata title="GoldenFloat: A Formally Verified, phi-Optimal Floating-Point Family" \
  --metadata author="t27 Project Team" \
  --metadata date="April 2026"
