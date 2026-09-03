# NOW -- The catalog table was 26 records stale, in every row (2026-09-04)

## A published seven-row table against what the gate actually runs

- `IGLA-FORMAL-RESULTS.md` P17 publishes what `t27c catalog-gate` checks. Every
  row has moved: mandatory-field **83 -> 109**, widths-partition 65 -> 91, the
  three gf checks 21 -> 30, source-agrees 10 -> 36, and `no-spurious-layout`
  **no longer exists** -- renamed to `fields-fit-concrete-width` in the W603
  correction printed directly above the table.
- Three checks the table has no row for run today: `emitted-agrees` (436),
  `getter-parity`, `gf-rule-unstated` (17).
- The findings column is the sharpest part: it publishes six zeros and one 5.
  The gate reports **3 findings and exits non-zero**. A reader auditing the
  catalog against this table would look for the wrong checks over the wrong
  populations and conclude it is clean.
- The repository contradicted itself in one build: `bootstrap/src/main.rs:405`
  says "whose **109** records", while the table and T397 both said 83.
- Corrected in the document's own convention -- a quoted block beside the
  original, anchored to a commit, with the W602 reading kept as the record it is.
- The test pins the one cell re-takable without a build: `mandatory-field` is
  bumped once per parsed record with no predicate, so its population is exactly
  `grep -c 'CATALOG:'`. It also asserts the help string and the document do not
  disagree about the same number.
- Three mutants, three kills. The gf rows moved for a REASON, not by drift:
  since #2792 the phi checks are gated on `rule=phi-ratio` rather than
  `cluster=GoldenFloat`, which the correction says rather than hiding.
