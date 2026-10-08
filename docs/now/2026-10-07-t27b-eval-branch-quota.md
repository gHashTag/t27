# NOW -- t27b: `@setEvalBranchQuota` is a compile-time no-op (2026-10-07)

## `@setEvalBranchQuota` (Closes #7410)

- `@setEvalBranchQuota(n);` raises the backward-branch budget of Zig's compile-time evaluation and does nothing at run time. t27b does not model that budget, so the statement lowers to nothing.
- Zig wants a compile-time `u32` operand. A run-time operand, a float, a value outside `u32`, or a call with other than one argument stays refused as `ExprCall(@setEvalBranchQuota)`.
- New conformance spec `specs/tri/t27b/conformance/eval_branch_quota.t27`: 4 tests, 4 pass in the reference and in t27b. Each of its 4 mutants fails the same test in both.
- Of 10 probes: 2 pass or fail the same in both; 4 are refused by t27b where the reference blocks; 2 are refused by t27b where the reference passes (operand from a typed local or an expression); 2 stop on an unrelated `StmtAssign(reference redeclares)` blocker.
- Lab corpus vs master 05e633d03: 823 -> 825 t27b passes of the reference's passes, mismatch 0, jit/interp mismatch 0. `specs/numeric/gfternary.t27` moves from blocked to pass (13/13), and the new conformance spec is added as pass.
- Code is in `cli/t27b/src/lower.rs` (+31) and `cli/t27b/tests/source.rs` (+39), listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063; master's `check_budget()` allows it (70 of 80).

Refs #6063. Closes #7410.
