---
name: research
description: Academic research workflow for GoldenFloat paper — NeurIPS 2026
version: 1.0.0
---

# Research Skill: Academic Writing Workflow

Execute academic research, literature review, and paper writing for NeurIPS submission.

## Target Files

- Paper: `docs/WHITEPAPER/latex/neurips_main.tex`
- Bibliography: `docs/WHITEPAPER/latex/references_expanded.bib`
- Figures: `docs/WHITEPAPER/latex/figure*.pdf`
- Deadline: May 4, 2026 (abstract), May 6, 2026 (full paper)

## Literature Review

```bash
# Query NotebookLM for related papers
./bootstrap/target/release/t27c bridge memory query \
  --notebook "t27-QUEEN-BRAIN" \
  --query "ternary floating point ML training efficiency"

# Add new source to bibliography (via spec-generated code)
./bootstrap/target/release/t27c bridge research bib-add \
  --key "gustafson2017posit" \
  --type article \
  --title "Posit: A New Kind of Floating-Point" \
  --author "Gustafson, John L." \
  --year 2017 \
  --category posit
```

## Paper Checklist

Coverage check before submission:

- [ ] 25+ BibTeX entries (7 categories covered)
- [ ] NeurIPS checklist filled (Section *NeurIPS Paper Checklist*)
- [ ] Limitations section present (Section 7)
- [ ] Figure 1 (eye-catcher) included
- [ ] Anonymous GitHub repo linked
- [ ] 9 pages exactly (`pdflatex` check)
- [ ] Double-blind: no author names anywhere
- [ ] No "our previous work" references

## Paper Review with NotebookLM

```bash
# 1. Export LaTeX to Markdown for review
pandoc docs/WHITEPAPER/latex/neurips_main.tex \
  -o /tmp/paper_for_review.md \
  -f latex -t markdown

# 2. Upload to t27-QUEEN-BRAIN via tri wrapup
tri wrapup \
  --summary "NeurIPS 2026 draft for review" \
  --decisions "GoldenFloat format family, phi-optimization proofs" \
  --files "neurips_main.tex,figure1.pdf" \
  --steps "Review claims, related work, limitations"

# 3. Query for specific feedback
tri notebook query \
  --query "Does this paper clearly state 1-3 concrete claims in the abstract?"
```

## Review Criteria

Ask NotebookLM to review:

1. **Claims**: Are 1-3 concrete claims clearly stated in abstract/intro?
2. **Related Work**: Is the literature complete for floating-point formats?
3. **Limitations**: What limitations are not addressed?
4. **Reproducibility**: Does the paper meet NeurIPS reproducibility requirements?
5. **Theory**: Are all assumptions stated? Are proofs complete?

## Wrap-up Fields (research sessions)

| Field | Description |
|-------|-------------|
| `sources_found` | Papers retrieved from NotebookLM |
| `bib_entries_added` | New BibTeX entries added |
| `sections_edited` | Paper sections modified |
| `review_feedback` | Key feedback from NotebookLM review |
| `next_research_steps` | Outstanding literature gaps |

## LaTeX Compilation

```bash
cd docs/WHITEPAPER/latex

# Full compilation cycle (pdflatex -> bibtex -> pdflatex x2)
pdflatex neurips_main.tex
bibtex neurips_main
pdflatex neurips_main.tex
pdflatex neurips_main.tex

# Check page count
pdfinfo neurips_main.pdf | grep Pages

# Verify no warnings
grep -i "warning\|error" neurips_main.log
```

## Figure Generation

```bash
# Regenerate Figure 1 (if needed)
python3 figure1_gen.py

# Output: figure1.pdf (vector format)
```

## Double-Blind Verification

Before final submission, verify:

```bash
# Check for author names
grep -i "author\|affiliation\|university" neurips_main.tex

# Check for self-references
grep -i "our\|previous work\|we propose" neurips_main.tex

# Should find only anonymous references
# e.g., "In the previous work of Anonymous et al. [X]..."
```

## Submission Package

```bash
# Create anonymous submission package
mkdir -p /tmp/neurips2026_submission
cp docs/WHITEPAPER/latex/neurips_main.pdf /tmp/neurips2026_submission/paper.pdf

# Create code package (max 100MB)
# - Reference implementation in Zig
# - Benchmarking scripts
# - README (anonymous)

# Verify file sizes
ls -lh /tmp/neurips2026_submission/
# paper.pdf must be ≤ 50MB
# code.zip must be ≤ 100MB
```

## Related Specs

- `specs/research/notebooklm.t27` — NotebookLM integration
- `specs/research/literature.t27` — Bibliography management
- `specs/format/FORMAT-SPEC-001.json` — GF16 format definition
