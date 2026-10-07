# NOW -- t27b: a fn result typed as an anonymous struct (2026-10-07)

## `-> struct { a: T, b: U }` (Closes #7256)

- New conformance spec `specs/tri/t27b/conformance/anonymous_struct.t27`. It gives 4 pass with 0 vacuous under `t27c test-report`, and the same 4 under t27b, with 20 runtime asserts.
- A fn declared `-> struct { a: T, b: U }` now gets its own struct layout, keyed by the spelling and the fn, and `.{ .a = .., .b = .. }` fills it like a named struct. The caller reads the fields, compares them and passes them on.
- The reference prints the result type verbatim, so t27b takes only field types that are valid Zig there. A `str` field, a t27 array spelling such as `[u8; 2]`, a field named for a Zig keyword, and a parameter typed as an anonymous struct stay refused. The reference blocks every one of them too.
- 8 mutants each fail the same tests in t27b and in the reference. Of 14 probes, none disagree: 6 pass in both, 1 fails in both, and 7 are blocked by the reference and refused by t27b.
- Code is in `cli/t27b/src/lower.rs`, which is listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- Ledger `docs/reports/t27b_expectations.json`: `specs/port/scripts/test-agent-bridge.t27` (8 pass, 9 asserts) and the new spec move to pass. The agent-server `routes/memory.t27` now stops at `type [N]T`, and `gen_work_stealing.t27` at `ExprReturn` (`return undefined` in a void fn).
Refs #6063. Closes #7256.
