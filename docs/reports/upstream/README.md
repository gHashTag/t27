# Proposed upstream, `gHashTag/trinity-fpga`, branch `tnf-publication-readiness`

Three changes, each measured before and after. **None has been sent.** Reporting
a defect to another project is a publication, and that decision belongs to the
repository's owner.

## 1. `01-section-labels.patch` — a paper defect, twelve references

Twelve `Section~\ref{sec:X}` resolved to FIGURE numbers, so a reader following
"see Section N" landed on a figure. Cause, identical in all seven labels: the
`\label{sec:X}` sat 7-8 source lines below its `\section{}` heading with a
`figure` environment in between, so LaTeX bound it to the figure counter the
float had just stepped.

Fix: each label moved to immediately follow its heading. Nothing else changed.

    check_ref_kinds   19 failures -> 7
    build             136 pages, 0 errors, 0 undefined references, unchanged

The remaining 7 are NOT defects -- see below.

## 2. `check_ref_kinds` — a false positive, seven captions

All seven surviving "sits inside the very section it points at" are
`\caption{Canon plate. ... Section~\ref{X} ...}`. A plate caption naming the
section it illustrates is correct and useful: the float lands pages away from the
text it belongs to. The gate reads SOURCE position.

Suggested: exclude `\caption{...}` from the self-reference rule.

## 3. `check_self_consistency.WIDENED.py` — a false positive, eight marks

The gate reported "claims 20 retractions but the body marks 12". Measured over
the same file, the paper's withdrawal vocabulary is wider than the gate's four
forms: `we withdraw` (4) and `\paragraph{...Retract...}` (1) are unmatched.
Widening to those two unambiguous first-person classes takes 12 -> 17.

`does not survive` (8), `was wrong` (7) and `narrowed` (4) are deliberately NOT
counted: they qualify a claim without withdrawing it.

**The residual gap is not a gap.** Section 27 items 3 and 4 -- a claim and ITS
REPLACEMENT -- are closed by one sentence at `:2444`, and the paper withdraws
claims no longer in it (`previous revision` x6, `earlier version` x7). Enumerated
retractions and body marks are not in bijection, so requiring equality is wrong
in principle rather than tuned wrong.
