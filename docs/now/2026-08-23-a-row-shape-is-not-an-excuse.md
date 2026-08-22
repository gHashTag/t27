# NOW -- 2110 rows left the checks by having the wrong shape (2026-08-23)

Refs #2325.

- The D/D2/E loop dropped every row that is not the f64 shape, with two silent
  `continue`s. Measured on the live corpus: **2110 of 5905 rows (35.7%)** leave
  there, across **seven packs the INDEX labels `bitexact`** --
  gf14/48/96/128/256/512/1024, gf256 alone contributing 2021.
- Consequence, reproduced: setting `abs_error` to `"1e300"` on all 2021 gf256
  rows and refreshing the pack digest returned **exit 0 CLEAN**. That field is
  validated by nothing -- the wide witness covering those packs re-derives from
  `bits` and compares `value`, and never reads `abs_error`.
- Every row is now classified into a new check `F`: wide rows must carry an
  exactly-zero stored error, and a shape nobody planned for is reported rather
  than skipped. In both controls the failing check list is exactly `["F"]` while
  C, B and D stay green -- so F is provably the thing catching it.
- **The demotion guard I added yesterday inherited the same blind spot.**
  `_count_value_rows` recognised only the f64 shape, so relabelling gf256
  `bitexact` -> `structural` passed with 2021 rows in the file. It now knows
  both shapes; the control fails with `["B2"]` and names the pack.
- The selftest fixture was a **schema monoculture**: every row was the f64
  shape, so the branch that drops other shapes was taken zero times by any
  planted mutant, in either selftest. It now carries a wide pack, and three new
  mutants (TF, TF2, TB2b) exercise the new path. 17/17 -> 20/20, and reverting
  either half of the fix drops exactly its own mutant.
- Simpler than planned in one place: no corpus data change was needed.
  `_is_exact_zero(None)` is True, so a wide row that stores no `abs_error` at
  all is already honest, and gf14's rows did not need a field added.
