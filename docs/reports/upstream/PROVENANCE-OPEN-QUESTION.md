# Open question for the author: which record backs which table

Not a patch. A question that only you can answer.

## What was measured

`tnf_paper.tex` names none of its data:

    `.json`          0 occurrences        `recompute_`     0
    `measurements/`  0 occurrences        `gen_figures`    0

All twelve occurrences of "measurements" are the English word. `measurements/README.md`
describes each record in prose -- "backs the rung-threshold table" -- naming no `\label`.

So of 59 tables carrying fractional numbers, **0 state which file backs them**, and
4 have a regenerator a reader could run (121 of 2,094 fractional cells, 5.8%).

## Why this is not offered as a patch

Matching the README's prose to all 60 captions by keyword resolves exactly one:

| record | candidate captions | verdict |
|---|---|---|
| `gpt2_window_2026-08-13e` | `tab:landing` (3/3) vs `tab:blockpct` (1/3) | **unambiguous** |
| `per_rung_2026-08-13g` | `tab:rungthr`, `tab:landing` | tie 2/2 |
| `workloads_strict_2026-08-13g` | `tab:ladderacc`, `tab:workloads` | tie 2/2 |
| `strict_range_2026-08-13g` | `tab:invariant`, `tab:rungthr` | tie 3/3 |
| `crossover2_2026-08-13e` | `tab:crosscorrected`, `tab:window` | tie 1/1 |
| `centering_2026-08-13f` | -- | no candidate |
| `tnf_downstream_bayesian_si_2026-08-13` | -- | no candidate |

A provenance table assembled from ties would be wrong in five rows of eight and
would read downstream as authoritative. That is the defect class Section 27
documents. So: the finding and the one clean mapping are offered; the rest is left
open.

## The cheap fix, if you want it

One line per caption -- `\emph{Data:} \texttt{measurements/per_rung_2026-08-13g.json}` --
turns 59 unverifiable tables into 59 checkable ones. It is the least expensive
change in this package and the largest gain in what a reader can confirm.
