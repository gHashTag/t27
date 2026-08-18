# Proposed upstream, `gHashTag/trinity-fpga`, branch `tnf-publication-readiness`

Seven items: three paper defects, two gate corrections, one superseded-script
marker, one open question. Every claim below was measured on
`origin/tnf-publication-readiness` at `9ce0d1129`, before and after, in the same
environment.

The paper is in good shape. Two of the three defects share one cause, and the
third is a LaTeX binding accident. Nothing here challenges a result.

---

## 1. `01-section-labels.patch` — twelve references pointed at figures

Twelve `Section~\ref{sec:X}` resolved to FIGURE numbers, so a reader following
"see Section N" landed on a figure. Cause, identical in all seven labels: the
`\label{sec:X}` sat 7–8 source lines below its `\section{}` heading with a
`figure` environment in between, so LaTeX bound it to the figure counter the
float had just stepped.

Fix: each label moved to immediately follow its heading. Nothing else changed.

    check_ref_kinds   19 failures -> 7
    pdflatex x3       136 pages, rc=0, 0 LaTeX errors, 0 undefined references

## 2. `02-tnf8-law-row.patch` — the TNF8 row of `tab:law`

    -TNF8 & $4$ & $3.12\mathrm{e}{-2}$ & $1.11\mathrm{e}{-2}$ & $0.36$ & $1.082$
    +TNF8 & $4$ & $3.12\mathrm{e}{-2}$ & $1.54\mathrm{e}{-2}$ & $0.49$ & $1.883$

Confirmed by the table's own regenerator, character for character:

    $ python3 recompute_law_table.py
    TNF8  M=4  u=3.12e-02  measured=1.54e-02  ratio=0.49  flatness=1.883

## 3. `03-tnf8-ladder-cell.patch` — the TNF8 middle band of `tab:ladderacc`

    -TNF8 & $8$ & $1.07\mathrm{e}{-2}$ & $1.16\mathrm{e}{-2}$\,(82/187) & beyond range
    +TNF8 & $8$ & $1.07\mathrm{e}{-2}$ & $2.02\mathrm{e}{-2}$\,(85/187) & beyond range

Confirmed by `recompute_ladder_exact.py`, which owns this table:

    TNF8  M=4  dec=8  [('1.07e-02', 0), ('2.02e-02', 102), ('out', 285)]

The tuple is `(mean, out_of_range_count)` (line 64). The band holds 187 values —
`TNF4` shows `('out', 187)` for the same band — so in-range is `187 - 102 = 85`.

**Items 2 and 3 share one cause.** A reconciliation updated the TNF16 and TNF32
rungs and did not reach the 8-bit one. Both tables carry the pre-reconciliation
8-bit numbers; every other rung already agrees with its regenerator.

---

## 4. `tools/check_withdrawn_live.py` — three false positives, and a baseline that must be regenerated with it

Three narrow corrections, each with the case that motivated it in a comment:

| change | effect |
|---|---|
| `NUM` captures the LaTeX suffix | `2.44\%` (withdrawn) no longer matches `2.44\mathrm{e}{-4}` (a live precision-table entry) |
| replacement qualifiers may follow the number, with a strictly smaller key set | `0.180 at a median of five placement seeds` is recognised as the replacement, without making `0.189 against 0.177` invisible |
| context window stops at `\\` | a key no longer reaches into the next table row, where an unrelated index shifts it |

Measured against the same paper, baseline removed so the counts describe the
document rather than the diff:

    before   22 numbers in withdrawal passages, 19 flagged live
    after    21 numbers in withdrawal passages, 17 flagged live

Strictly fewer. It drops `0.1736` and `2.44`, and refines `0.1651` to
`0.1651\%`. Nothing new is introduced.

**`tools/withdrawn_live_baseline.txt` must be regenerated in the same commit.**
The baseline is keyed on the value *and its surrounding words*, so changing the
context window invalidates every key: without regeneration the gate reports 17
spurious new failures on first run. Regenerated here, 15 → 25 entries, after
which `check_withdrawn_live` exits 0.

Negative test, which this file's own header demands: injecting the withdrawn
`0.189` outside any withdrawal zone is caught, `rc=1`, and the injection
provably changed the file (a no-op replace would make the test meaningless).

## 5. `tools/check_self_consistency.py` — eight unmatched marks

The gate reported "claims 20 retractions but the body marks 12". The paper's
withdrawal vocabulary is wider than the gate's four forms: `we withdraw` (4) and
`\paragraph{...Retract...}` (1) are unmatched. Widening to those two unambiguous
first-person classes takes **12 → 17**.

`does not survive` (8), `was wrong` (7) and `narrowed` (4) are deliberately NOT
counted: they qualify a claim without withdrawing it.

**The residual 20 vs 17 is not a gap, and we suggest the equality itself is the
wrong test.** Section 27 items 3 and 4 — a claim and ITS REPLACEMENT — are closed
by one sentence at `:2444`, and the paper withdraws claims that are no longer in
it (`previous revision` ×6, `earlier version` ×7). Enumerated retractions and
body marks are not in bijection, so requiring equality is wrong in principle
rather than tuned wrong. The gate still exits 1 after this change; that is
honest, and the remedy is a different rule, not a different constant.

## 6. `check_ref_kinds` — a false positive, seven captions

All seven surviving "sits inside the very section it points at" are
`\caption{Canon plate. ... Section~\ref{X} ...}`. A plate caption naming the
section it illustrates is correct and useful: the float lands pages away from the
text it belongs to. The gate reads SOURCE position.

Suggested: exclude `\caption{...}` from the self-reference rule. Not patched
here — it is your call whether the rule or the captions should move.

## 7. `recompute_ladder_table.SUPERSEDED.py` — a marker, not a change

`recompute_ladder_table.py` regenerates a table the paper no longer contains: it
emits decades 8, 26, 80, 242 while the ladder table prints 2, 8, 24, 73, and
26/80/242 occur ZERO times anywhere in `tnf_paper.tex`. The difference is the
source — the script reads the SPECIFICATION's field widths, the paper now takes
the ORACLE's throughout, which the table's own caption records.

Running it against the current paper reports a table's worth of false
differences. That is what two separate audit passes saw and could not account
for. `recompute_ladder_exact.py` owns `tab:ladderacc`. The proposed header says
so, in the file, so the next reader loses minutes instead of hours.

## 8. Provenance — the paper does not state it, but the labels encode it

`tnf_paper.tex` names none of its data: `.json` 0 occurrences, `measurements/` 0,
`recompute_` 0, `gen_figures` 0. All twelve occurrences of "measurements" are the
English word. So of 59 tables carrying fractional numbers, **0 state which record
backs them**, and 4 have a regenerator a reader could run — 121 of 2,094
fractional cells, **5.8 %**.

**Correction to an earlier draft of this note.** It said the mapping was not
recoverable from outside and offered `gpt2_window -> tab:landing` as the one safe
row. Both claims were wrong, and the cause was that the search read caption TEXT
and never grepped `\label{tab:`. The labels are named after the records:

    gpt2_window_2026-08-13e.json        ->  tab:gpt2window
    centering_2026-08-13f.json          ->  tab:centring
    tnf_downstream_bayesian_si_...json  ->  tab:downstream
    workloads_strict_2026-08-13g.json   ->  tab:workloads

`centring` also explains one "no candidate": the label uses the British spelling
(13 occurrences) and the prose the American (148).

### What numeric overlap adds, and where it stops

Each record's numbers were matched against the cells of all 60 tables, with the
median across all tables as the noise floor. Values 0, 1, 2, 100 and anything
below 1e-3 excluded as non-distinctive:

| record | best table | match | runner-up | noise floor |
|---|---|---|---|---|
| `workloads_strict` | `tab:workloads` | 93.3 % (89 cells) | 46.7 % | 22.9 % |
| `centering` | `tab:centring` | 92.6 % (27 cells) | 73.3 % | 27.3 % |
| `inside_window` | `tab:landing` | 78.8 % (33 cells) | 33.3 % | 8.2 % |
| `crossover2` | `tab:window` | 96.6 % (29 cells) | 66.7 % | 24.4 % |
| `tnf_downstream` | `tab:downstream` | 14.7 % (34 cells) | 13.3 % | 0.0 % |
| `per_rung` | `tab:paretobudget` | 95.2 % | 82.8 % | **53.2 %** |
| `strict_range` | `tab:invariant` | 100 % | 93.1 % | **56.5 %** |

**The method fails on large records.** `per_rung` holds 408 distinct numbers and
`strict_range` 563; at that size a record matches most of the paper and the noise
floor rises above half. Those two rows are not evidence of anything.

**And it is not stable under its own filter.** `gpt2_window` ranks
`tab:gpt2window` at 44 % against `tab:landing` at 5.4 % unfiltered, but with the
non-distinctive values excluded `tab:gpt2window` leaves the top two entirely. We
report both rather than the one we prefer.

### What we would actually assert

Three mappings carry **two independent agreeing signals** — the label is named
after the record, and the record is that table's numeric top-1:

    centering_2026-08-13f.json         -> tab:centring
    workloads_strict_2026-08-13g.json  -> tab:workloads
    tnf_downstream_bayesian_si_....json -> tab:downstream

One carries strong numeric separation without name support:

    inside_window_2026-08-13f.json     -> tab:landing   (78.8 % vs 33.3 %, floor 8.2 %)

The rest we leave to you. `per_rung` and `strict_range` in particular cannot be
settled by this method at all.

### The cheap fix, if you want it

One line per caption — `\emph{Data:} \texttt{measurements/per_rung_2026-08-13g.json}` —
turns 59 tables a PDF reader cannot check into 59 they can. The information
already exists in your label names; it just does not survive into the rendered
document. It is the least expensive change in this package and the largest gain
in what a reader can confirm.

## What was verified, and what was not

    patched paper builds            136 pages, rc=0, 0 errors, 0 undefined refs
    tab:law TNF8                    matches recompute_law_table.py exactly
    tab:ladderacc TNF8              matches recompute_ladder_exact.py exactly
    check_withdrawn_live            strictly fewer findings; negative test passes
    check_self_consistency          marks 12 -> 17
    the other 55 numeric tables     NOT verified -- no regenerator, no named record

`tnf_paper.pdf` is NOT included. The tree ships a built PDF; ours rebuilds at
136 pages but differs in byte size (31,730,993 vs 32,303,608) because the
toolchain is not yours. Rebuild it on your side rather than taking ours.
