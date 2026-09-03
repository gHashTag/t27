# NOW -- `known` read a file that has never existed (2026-09-05)

## Three empty signals, and a sign-off telling you to record the negative

- `t27c known` read table captions from `tnf_paper.tex`. **0 on disk, 0 in the
  index, 0 ever added on any branch** -- the same glob does find
  `tnf_paper.FIXED.tex`, so the zero is a real absence. `find` returned None,
  `unwrap_or_else` handed `read_to_string` a path just proven absent, `if let
  Ok` skipped the loop, and `(none)` printed -- identical to "no caption
  mentions this".
- The `--dir` was never checked. The project's own record invokes
  `--dir research/arxiv_tnf`, which does not exist; every section fell through
  and the command concluded **"Nothing speaks to this. Measure -- and record
  the negative, it is a result."** It had read nothing.
- The rule was already written twenty lines above, for the gates directory --
  *a silent "(none)" from looking in the wrong directory is exactly the false
  all-clear this command exists to prevent* -- and paid for with a
  `gates read from` line. It was omitted for captions.
- The count reaches published numbers. `oracle-method.md` makes this step 1 of
  the mandated order and weighs captions; `IGLA-FORMAL-RESULTS.md` publishes
  "fourteen gates, ninety baselines or **sixty captions**". The caption
  population is structurally zero, and 16 gates are on disk against fourteen
  published.
- Now: an absent `--dir` refuses with exit 2, a directory with no paper prints
  `NO PAPER FOUND` and both paths tried, and the summary says `NOT READ` rather
  than `caption 0`.
- The signal was never broken, only unfed: with a paper in place the same loop
  counts **19** captions in the real `.tex` and **1** in a two-table fixture.
- Three mutants, three kills, and the third is the control: refusing every
  directory fails two assertions, including the one that proves the loop still
  counts.
