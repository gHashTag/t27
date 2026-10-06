# NOW -- t27b: array literals with no result type of their own (2026-10-06)

## constant slice returns and literal locals passed by address (Closes #6866)

- New conformance spec `specs/tri/t27b/conformance/array_literal_value.t27`. It gives 6 pass under `t27c test-report` (none vacuous), and the same 6 under t27b.
- `return [a, b]` in a fn that returns `[]const E` is a constant slice in the reference (`@constCast(&[_]E{ a, b })`). t27b now places the elements in read-only data and returns a slice of them. Integer elements take the return element type.
- Two cases stay refused, each with its own name: a literal of run-time values (`ExprArrayLiteral(run-time slice return)`), and a constant returned as a mutable `[]E` from a fn that a test reaches (`ExprArrayLiteral(constant to mutable slice)`). The reference faults on a write through that slice.
- A local `const x = [N]T{...}` is a tuple in the reference: N and T are dropped. When it is only used as `&x` in arguments of one element type, t27b binds it as a slice local, as it already did for `&[_]T{...}`.
- Code is in `cli/t27b/src/lower/arraylit.rs`, with two small hooks in `lower.rs`. Tests are in `cli/t27b/tests/arraylit.rs`. Rust is edited under the owner's `owner-approved-foreign` approval on #6063; the three files are listed in `tools/policy/foreign-exceptions.txt`.
- Lab, same tree and same reference before and after: `specs/ml/layers/conv2d_layer.t27` and `specs/port/trios/apps/trios-macos/rings/RUST-13/trios-mesh/src/bin/smoke_m1.t27` move from blocked to pass, the new spec passes, and per-test mismatch stays 0.
- The ledger `docs/reports/t27b_expectations.json` is blessed in this PR (Q53). Reference verdicts come from the lab's master run bdf66f4f7. t27b verdicts come from this branch's t27b on master a17dc9d70, and crashed or timed-out dirs were rerun with 4 jobs and a 300 s timeout. 0 entries move down. The cap goes from 67 to 66, with no `--accept-new`.
- `specs/tri/t27b/fuzz.t27` (new on master) is `codegen` under t27b before and after this change: 80000 bytes of local aggregates in one frame, and the limit is 16384. It enters the ledger as unimplemented.
