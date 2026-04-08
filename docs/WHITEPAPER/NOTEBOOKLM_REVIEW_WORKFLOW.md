# Paper Proofreading with NotebookLM — NeurIPS 2026

> **Purpose:** Use NotebookLM to review GoldenFloat paper against NeurIPS criteria.
> **Target:** `docs/WHITEPAPER/latex/neurips_main.tex`

## Workflow Overview

```
LaTeX → Markdown → Upload to NotebookLM → Query for review → Feedback
```

## Step 1: Export LaTeX to Markdown

```bash
cd docs/WHITEPAPER/latex

# Convert LaTeX to Markdown for NotebookLM
pandoc neurips_main.tex -o /tmp/neurips_review.md -f latex -t markdown

# Optional: Add section headers for clarity
echo -e "\n---\n\n# NeurIPS Review Criteria\n\n" >> /tmp/neurips_review.md
```

## Step 2: Upload to NotebookLM via `tri wrapup`

```bash
cd /Users/playra/t27

# Create a wrapup entry with the paper content
./scripts/tri wrapup \
  --summary "GoldenFloat NeurIPS 2026 draft for review" \
  --decisions "GoldenFloat format family, phi-optimization proofs, ternary computing" \
  --files "docs/WHITEPAPER/latex/neurips_main.tex,docs/WHITEPAPER/latex/figure1.pdf" \
  --steps "Review claims, related work, limitations, and reproducibility" \
  --dry-run
```

## Step 3: Query for Specific Feedback

Use `tri notebook query` to ask targeted questions:

### 3.1 Claims Review

```bash
./scripts/tri notebook query \
  --notebook "t27-QUEEN-BRAIN" \
  --query "Does the abstract clearly state 1-3 concrete claims? What are they?"
```

**Expected feedback:**
- Claim 1: φ is unique self-similar proportion for bit allocation
- Claim 2: Rounding rule matches all 7 GF formats exactly
- Claim 3: φ-guided mixed-precision as O(L) baseline

### 3.2 Related Work Review

```bash
./scripts/tri notebook query \
  --notebook "t27-QUEEN-BRAIN" \
  --query "Is the related work section complete for floating-point formats? What's missing?"
```

**Check categories:**
- IEEE 754 standards ✓
- Posit/Unum formats ✓
- Low-precision ML (FP8, FP16, bfloat16) ✓
- Ternary neural networks ✓
- Golden ratio applications ✓
- Hardware implementations ✓
- Floating point theory ✓

### 3.3 Limitations Review

```bash
./scripts/tri notebook query \
  --notebook "t27-QUEEN-BRAIN" \
  --query "What limitations of the work are NOT addressed in the Limitations section?"
```

**Current limitations (from paper):**
1. No ternary hardware implementation (software simulation only)
2. φ-allocation validation preliminary (small models only)
3. Posit benchmark data missing
4. Qutrit bridge incomplete

**Ask NotebookLM:** Are there other limitations?

### 3.4 Reproducibility Review

```bash
./scripts/tri notebook query \
  --notebook "t27-QUEEN-BRAIN" \
  --query "Does the paper meet NeurIPS reproducibility requirements? What's missing?"
```

**NeurIPS reproducibility checklist:**
- [ ] Code availability (will be in anon GitHub)
- [ ] Data availability (not applicable - synthetic benchmarks)
- [ ] Clear instructions for running experiments
- [ ] Hardware/software specifications
- [ ] Hyperparameters listed

### 3.5 Theory Review

```bash
./scripts/tri notebook query \
  --notebook "t27-QUEEN-BRAIN" \
  --query "Are all assumptions stated in the theoretical derivations? Are proofs complete?"
```

**Verify:**
- Proposition 1 proof complete ✓
- Proposition 2 proof complete ✓
- All assumptions stated (self-similarity, rounding rule) ✓

## Step 4: Incorporate Feedback

Create a feedback tracking document:

```markdown
# NotebookLM Review Feedback

## Claims
- [ ] Feedback: ...
- [ ] Action: ...

## Related Work
- [ ] Feedback: ...
- [ ] Action: ...

## Limitations
- [ ] Feedback: ...
- [ ] Action: ...

## Reproducibility
- [ ] Feedback: ...
- [ ] Action: ...

## Theory
- [ ] Feedback: ...
- [ ] Action: ...
```

## Step 5: Re-compile and Re-review

```bash
cd docs/WHITEPAPER/latex

# After incorporating feedback
pdflatex neurips_main.tex
bibtex neurips_main
pdflatex neurips_main.tex
pdflatex neurips_main.tex

# Re-upload for second review
# (same process as Step 1-4)
```

## Alternative: Direct NotebookLM Web UI

If `tri notebook` commands are not available:

1. Go to https://notebooklm.google.com
2. Select or create "t27-QUEEN-BRAIN" notebook
3. Upload `/tmp/neurips_review.md` as a source
4. Use the AI chat to ask review questions

## Review Query Templates

### Template 1: Abstract Review
```
Review the abstract for NeurIPS submission. Does it:
1. State 1-3 concrete claims?
2. Explain the significance?
3. Outline contributions?
4. Stay under 2000 characters?
```

### Template 2: Introduction Review
```
Review the introduction. Does it:
1. Clearly state the problem?
2. Explain why it's important?
3. Summarize related work?
4. State contributions clearly?
```

### Template 3: Double-Blind Check
```
Check this paper for double-blind compliance:
1. Are there any author names?
2. Are there any self-references ("our", "we")?
3. Are there any affiliations mentioned?
4. Does it pass NeurIPS anonymity requirements?
```

## Post-Review Actions

- [ ] Incorporate high-priority feedback
- [ ] Update bibliography if needed
- [ ] Add missing citations
- [ ] Clarify ambiguous statements
- [ ] Update limitations section
- [ ] Verify page count (≤ 9 pages)
- [ ] Re-run LaTeX compilation
- [ ] Final review before submission
