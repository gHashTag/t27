# NOW — every mutation pushed the same way (2026-08-24)

`tri gates mutate` turned failures into passes. Every one of its mutants asked the same question — *can this gate still fail?* — so a control made entirely of cases demanding RED satisfied all of them, and the opposite defect went unmeasured for six days.

- **`--loud` is the missing operator.** `return 0` → `return 1`: does anything notice a gate that fails on a **clean** tree? First run, seven of thirteen gates have survivors.

- **Verified end to end on the worst one, which is mine.** `check_gate_preconditions.py` with its success return forced to 1:

  ```
  OK: 9 precondition(s) across 6 gates fail loudly; 1 known-uncovered
  exit 1
  control: exit 0 -- nothing noticed
  ```

  The gate prints a clean bill of health and reports failure. This is the exact mirror of the campaign's first finding — `check_catalog_integrity` printing `OK` while exiting 0 — and it survived every iteration since, because every mutation until today pushed the same way.

- **The survivors are one coherent class, not eight accidents.** Four are the success return of `--update-baseline`, right after *"baseline written: N entries"*: `check_json_parses`, `check_seal_coverage`, `check_specs_generate`, `check_withdrawn_live`. Nothing exercises the ledger-writing path's exit code. `check_vector_data` is the exception at 2/2 — it has a case that runs `--update-baseline` and asserts its refusal, added three days ago for an unrelated reason.

- **Only a bare `return 0` counts.** A ternary can yield 0 on one arm and a verdict on the other, so forcing the whole line to 1 is the silent operator seen backwards and would be scored against the wrong control. The two directions are asserted never to claim the same line.

- **Not fixed here.** Eight sites across seven gates is the next iteration's work, and landing the operator with a finding is the finished state of this one. The alternative — fixing eight things in the same change that introduced the tool that found them — would leave nobody able to check whether the tool or the fixes were doing the work.

Refs #2472
