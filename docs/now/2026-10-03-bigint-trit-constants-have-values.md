# NOW -- bigint Trit constants have values (2026-10-03)

## What was read

- `t27c parse-complete` on master 146321ca: `specs/ternary/bigint.t27` discarded 6 tokens on lines 37-39, the `.neg`, `.zero` and `.pos` initialisers of `TRIT_NEG`, `TRIT_ZERO` and `TRIT_POS`.
- `t27c gen` emitted `pub const TRIT_NEG: Trit;` (and the same for the other two): constants with no value. `zig ast-check` reported 4 errors, 3 of them `variables must be initialized` on those lines.

## What changed

- The constants are `Trit.neg`, `Trit.zero` and `Trit.pos`, the qualified form the rest of the corpus uses. `t27c gen` now emits the values, and `zig ast-check` reports 0 errors (4 on master).
- Both seals were regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH). The `parse-no-discard` entry left `docs/reports/suite_expectations.json`; `t27c suite --ratchet --corpus-only` is CLEAN.

## Not verified

- `t27c test-report` is still BLOCKED, now one step later: `unable to load 'Trit.zig'`, because `use tritype-base::Trit` resolves to no module. On master it stopped at the uninitialised constants. No test ran before or after.

Closes #5742
