# NOW -- a selftest that ran nowhere, and the report that counted it (2026-08-23)

Refs #2325.

- `tools/wp18_gate_selfconsistent_selftest.py` was invoked by **no workflow, no
  Makefile target, no script** -- a whole-tree grep found it only in its own
  docstring. It works: `SELFTEST RESULT: 13 PASS, 0 FAIL`, exit 0.
- It is not redundant with the selftest that DOES run. Measured with two
  mutants of Check B's three-kind dispatch -- `bitexact_selfconsistent` folded
  into `bitexact`, and an unrecognised kind ignored instead of reported:

      workflow selftest   exit 0   (green on both)
      live-corpus run     exit 0   (green on both)
      the orphan          exit 1   (catches both)

  The unknown-kind branch is reachable from any typo in a real `kind` field.
- Now a step in `conformance-integrity-gate.yml` and in both `paths:` blocks.
  Re-measured after wiring: the step exits 1 on each mutant and 0 on the clean
  tree.
- `docs/reports/WAVE_LOOP_70_REPORT.md:42` described the suite as "19 positive
  validations + 6 failure scenarios (25/25 pass)". It is 13, and the earlier
  figure reproduces from no run of it. Corrected in place with the old wording
  quoted, because that number had been offered as evidence of coverage.
