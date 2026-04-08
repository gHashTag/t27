# GoldenFloat Paper - LaTeX Source

This directory contains the LaTeX source for the NeurIPS 2026 OPT Workshop submission.

## Files

- `main.tex` - Main LaTeX document (NeurIPS-style formatting)
- `references.bib` - Bibliography in BibTeX format
- `main.pdf` - Compiled PDF (generated)
- `README.md` - This file

## Building Locally

### Using pdflatex (recommended)

```bash
cd docs/WHITEPAPER/latex
pdflatex -interaction=nonstopmode main.tex
bibtex main
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
```

### Using pandoc from Markdown

```bash
cd docs/WHITEPAPER
pandoc gf_paper_v3_imrad_draft.md \
  -o paper_from_md.pdf \
  --pdf-engine=xelatex \
  --bibliography=latex/references.bib \
  --citeproc
```

## Overleaf Integration

1. Create a new project on Overleaf
2. Upload `main.tex` and `references.bib`
3. Set compiler to XeLaTeX (recommended for math fonts)
4. Click "Recompile" to generate PDF

## References

Citations are extracted from the Markdown draft and formatted in BibTeX:

- t27 GitHub repository
- Knuth (1974) - The Art of Computer Programming
- Gustafson (2017) - The Posit
- Etiemble (2019) - Ternary Circuits
- Huawei (2025) - Ternary logic gate patent
- Bennett & Brassard (1984) - Quantum cryptography
- Mixed-Precision Quantization Survey (2024)

## NeurIPS 2026 Deadlines

| Date | Milestone |
|------|-----------|
| April 4  | Abstract deadline |
| May 6     | Full paper deadline |
| May 20     | Notification |
| July 7-13  | Workshop |

## Notes

- The draft is in IMRaD format (Introduction, Methods, Results, Discussion)
- Total word count: ~10,000
- Main format: GF16 (16-bit GoldenFloat) - primary contribution
