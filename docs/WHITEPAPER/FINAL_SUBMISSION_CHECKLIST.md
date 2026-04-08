# NeurIPS 2026 Final Submission Checklist

> **Paper:** GoldenFloat: A Formally Verified, $\varphi$-Optimal Floating-Point Family
> **Deadline:** May 6, 2026 (AOE) — Full paper + supplementary

## Pre-Submission Checklist

### PDF Generation

- [ ] Run `pdflatex neurips_main.tex` → Clean compile, no errors
- [ ] Run `bibtex neurips_main` → No warnings
- [ ] Run `pdflatex neurips_main.tex` ×2 → For cross-references
- [ ] Verify `neurips_main.pdf` exists
- [ ] Check page count: `pdfinfo neurips_main.pdf | grep Pages` → **9 pages or less**
- [ ] Check file size: `ls -lh neurips_main.pdf` → **≤ 50 MB**
- [ ] Open PDF and visually inspect for issues

### Content Verification

- [ ] **Abstract** ≤ 2000 characters (plain text)
- [ ] **Title** is clear and descriptive
- [ ] **Authors**: Anonymous (double-blind)
- [ ] **Affiliations**: None (double-blind)
- [ ] **Acknowledgments**: None (double-blind)
- [ ] **References**: 25+ entries
- [ ] **Bibliography**: All cited works included
- [ ] **Figures**: Figure 1 included (vector PDF)
- [ ] **Tables**: All tables formatted correctly
- [ ] **Equations**: All equations numbered correctly

### Double-Blind Compliance

- [ ] **No author names** in PDF (search for author names)
- [ ] **No affiliations** (search for universities, labs)
- [ ] **No self-references** (search for "our", "we", "my")
- [ ] **Anonymous citations** (use "Anonymous et al. [X]")
- [ ] **GitHub link**: Anonymous repo only (no personal links)

### NeurIPS Paper Checklist

The following must be in the PDF as a section titled "NeurIPS Paper Checklist":

- [ ] **Claims**: Section completed with Yes/No + Justification
- [ ] **Limitations**: Section completed with Yes/No + Justification
- [ ] **Theory Assumptions and Proofs**: Section completed
- [ ] **Experimental Result Reproducibility**: Section completed
- [ ] **Open Access to Data and Code**: Section completed
- [ ] **Broader Impact**: Section completed

### Code Package (Separate ZIP, ≤ 100 MB)

- [ ] Anonymous GitHub repository created
- [ ] README.md is anonymous (no names, affiliations)
- [ ] Code compiles and runs
- [ ] Tests pass
- [ ] ZIP file created: `code.zip`
- [ ] ZIP file size ≤ 100 MB
- [ ] README includes reproduction instructions

### LaTeX Source Verification

```bash
# Check for common issues
grep -n "TODO\|FIXME\|XXX" neuraps_main.tex  # Should be empty
grep -n "author\|affiliation" neuraps_main.tex  # Should be "Anonymous" only
grep -n "our\|we propose\|my work" neuraps_main.tex  # Should be minimal/absent
```

### BibTeX Verification

```bash
# Check bibliography
cd docs/WHITEPAPER/latex
cat references_expanded.bib | grep "@" | wc -l  # Should be ≥ 25

# Check categories
# ieee754, posit, low_precision_ml, ternary_nn, golden_ratio, hardware, theory
```

## Final Compilation Commands

```bash
cd docs/WHITEPAPER/latex

# Clean build
rm -f neurips_main.aux neurips_main.bbl neurips_main.blg neuraps_main.log

# Full compilation
pdflatex neurips_main.tex
bibtex neurips_main
pdflatex neurips_main.tex
pdflatex neurips_main.tex

# Verify
pdfinfo neuraps_main.pdf | grep "Pages:"
ls -lh neuraps_main.pdf
```

## File Checksums (for verification)

Generate checksums for submission files:

```bash
shasum -a 256 neurips_main.pdf > neurips_main.pdf.sha256
shasum -a 256 code.zip > code.zip.sha256

cat neuraps_main.pdf.sha256
# Example: abc123...  neurips_main.pdf
```

## Submission Day Checklist (May 6, 2026)

- [ ] Log into OpenReview (https://openreview.net)
- [ ] Navigate to NeurIPS 2026 submission page
- [ ] Select **Main Track** submission
- [ ] Upload `neurips_main.pdf`
- [ ] Upload `code.zip` (optional but recommended)
- [ ] Enter title: "GoldenFloat: A Formally Verified, $\varphi$-Optimal Floating-Point Family"
- [ ] Enter abstract (copy from LaTeX)
- [ ] Select keywords: "Machine Learning", "Numerical Methods", "Hardware"
- [ ] Select contribution type: **Theory** (or **Use-Inspired**)
- [ ] Confirm authors are all added to OpenReview profile
- [ ] Declare any conflicts of interest
- [ ] Review submission preview
- [ ] **SUBMIT**

## Post-Submission

- [ ] Note submission ID from OpenReview
- [ ] Record submission timestamp
- [ ] Backup submission files to external storage
- [ ] Update personal task tracker
- [ ] **Wait for reviews** (notification: September 24, 2026)

## Rebuttal Period (if reviews are unfavorable)

- [ ] Download reviews from OpenReview
- [ ] Draft rebuttal addressing reviewer concerns
- [ ] Keep rebuttal professional and factual
- [ ] Submit rebuttal by deadline

---

**Current Status:**

- [ ] Sprint 9 (Paper Draft): ✅ Complete
- [ ] Sprint 10 (Infrastructure): ✅ Complete
- [ ] Sprint 11 (Submission Prep): 🔄 In Progress

**Days until deadline:** 26 days (as of April 8, 2026)
