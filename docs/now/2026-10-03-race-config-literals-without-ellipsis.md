# NOW -- race_config literals without the ellipsis (2026-10-03)

## What was read

- `t27c parse-complete` on master 146321ca: `specs/ml/optimizer/race_config.t27` discarded 4 tokens on lines 56 and 57 -- the trailing `...` of `0.5559321608...` and `0.6174052591...`.
- `t27c gen` kept the digits, so the emitted constants were already `0.5559321608` and `0.6174052591`.
- The comments call them `0.9 / phi` and `0.999 / phi`, which are 0.5562305899 and 0.6174159548: a mismatch in the fourth decimal. `specs/ml/optimizer/adamw.t27` writes the same arm as `0.9 / PHI`.

## What changed

- The `...` is gone; the digits are unchanged, so `t27c gen` output is byte-identical to master's.
- A comment above the two constants records the mismatch with 0.9/phi and 0.999/phi and leaves the choice of value to the owner.
- The seal `optimizer_RaceConfig.json` was regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH). The `parse-no-discard` entry left `docs/reports/suite_expectations.json`; `t27c suite --ratchet --corpus-only` is CLEAN.

## Not verified

- Which value the phi-damped arm should carry. Not decided here.
- `t27c test-report` is BLOCKED on master and here alike (undeclared `elapsed_time_ns`, 1 `ast-check` error before and after).

Closes #5740
