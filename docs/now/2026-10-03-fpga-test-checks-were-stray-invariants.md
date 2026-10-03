# NOW -- fpga test checks were stray invariants (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `specs/fpga/power_analysis.t27` discarded 3 tokens (line 372) and `specs/fpga/vcd_conformance_compare.t27` 38 (8 lines, 228-253).
- Every dropped line has the form `invariant <expr>;` at the end of a braceless test. `invariant` is a top-level keyword, so the parser closed the test there, opened a new invariant named after the first identifier (`r`, `total`) and discarded the rest. `t27c gen` emitted `// invariant: r NOT CHECKED -- body was not lowered (T43)` eight times; the six tests ran their setup and asserted nothing.

## What changed

- Each `invariant <expr>;` became `then <expr>` (the first in a test) or `and <expr>` (the next ones). The 9 expressions are unchanged.
- `t27c gen` now emits the assertions inside the tests (e.g. `record_pass_increments` checks `r.total_checks == 1` and `r.passed == 1`), and no `NOT CHECKED` stub is left.
- The four seals of the two specs were regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH).
- Both `parse-no-discard` entries left `docs/reports/suite_expectations.json`; `t27c suite --ratchet --corpus-only` is CLEAN (113 entries).

## Not verified

- `t27c test-report` is BLOCKED on both specs on master and here alike, for pre-existing reasons outside the changed lines (`expected type '[*]T'` in power_analysis, `expected type 'u32', found 'u64'` in vcd_conformance_compare). `zig ast-check` reports 0 errors before and after. The new assertions are in the Zig output, but none of them ran.

Closes #5730
