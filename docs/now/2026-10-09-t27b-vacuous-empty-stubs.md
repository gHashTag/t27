# NOW -- 22 empty generated spec stubs leave the corpus (2026-10-09)

## t27b vacuous set shrinks by deletion, not by a test that asserts nothing (Closes #8052)

- Lab run 7a07828fc: 142 specs pass in the reference and run 0 asserts in t27b (`pass_vacuous`).
  22 of them are generated stubs whose only non-comment lines are `module X;`,
  `use base::types;` and `use math::constants;`: no const, fn, struct, enum, test, invariant or
  bench. They break L4 (TESTABILITY) and have nothing to test.
- Deleted: the 22 specs, their 50 seals in `.trinity/seals/` (matched by `spec_path`, so
  `check_seal_coverage.py` sees no dangling seal), and their 22 rows in
  `docs/reports/t27b_expectations.json` (master's ledger minus those rows, nothing else).
- Nothing imports them: no `use`, no generated file, no row in `suite_expectations.json`.
  `tools/oracle/baseline.tsv` lists some of them, but its ratchet compares only the pass count
  (189 floor; the 2026-10-08 nightly passed 1105).
- Check: `tri t27b ratchet` of the 7a07828fc run with these 22 results removed against this
  ledger prints the same findings as master's run against master's ledger.
- Kept for now (empty too, but an open issue names them): `specs/tri/math/math.t27`,
  `specs/tri/math/measurement.t27`, `specs/tri/utils/string.t27`.
- Expected on the next lab run: vacuous 142 -> 120, reference denominator 1213 -> 1191.
