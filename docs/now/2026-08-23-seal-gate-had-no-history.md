# NOW -- the seal gate decided from history CI does not have (2026-08-23)

Refs #2325.

- `check_seal_coverage.py` classifies a broken seal as `dangling` (spec was
  committed then deleted -- "remove the seal with it, or restore both") or
  `phantom` (spec appears in NO commit -- "find the spec or drop the seal").
  It decides from `git log --all`.
- `seal-coverage.yml` used a bare `actions/checkout@v4`, i.e. depth 1. There
  is no history in a one-commit clone, so the dangling arm could never fire
  and every one printed as phantom -- wrong class, wrong prescribed repair, on
  the only output the gate prints. `l1-traceability.yml` and `now-sync-gate.yml`
  already set `fetch-depth: 0`, so the omission was load-bearing.
- Measured in a real `--depth=1` clone, three ways:

      full history           dangling 89, no-spec-path 5, stale 191
      shallow, before fix    PHANTOM  89, no-spec-path 5, stale 191
      shallow, after fix     dangling 89, no-spec-path 5, stale 191

- Two changes, because either alone is half: `fetch-depth: 0` in the workflow,
  and `_ever_existed` now detects a shallow checkout and returns the milder
  classification instead of answering a question it cannot see. A gate that
  reads its own verdict out of a truncated instrument is the broken-ruler
  error, and this function's docstring already records the author falling into
  it once -- an earlier pass overstated by fivefold.
