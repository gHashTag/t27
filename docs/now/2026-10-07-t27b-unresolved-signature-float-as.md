# NOW -- t27b: unresolved signatures and float `as` (2026-10-07)

## a fn no test reaches may name a struct t27b cannot lay out (Closes #7175)

- New conformance spec `specs/tri/t27b/conformance/unresolved_signature.t27`. It gives 2 pass under `t27c test-report`, and t27b gives the same 2 pass plus its invariant.
- Zig resolves a fn's signature only when analysis reaches the fn. In `specs/compiler/optimizer.t27` only fns that no test reaches take `Node`, which holds `[N]Node` by value, so the reference runs all 14 tests.
- t27b computes every signature up front. Now, when a fn outside the analyzed set fails only on struct layouts, t27b withdraws those errors and lowers no body for that fn. A call to it is refused as `ExprCall(unresolved fn)`.
- An undeclared name in such a signature is still refused, because Zig's AstGen resolves names in every fn.
- Negative controls, run on the lab:
  - A test that reaches the fn: the reference gives BLOCKED, and t27b refuses StructDecl.
  - An undeclared type: the reference gives BLOCKED, and t27b refuses it.

## `x as T` with a float operand (Closes #7179)

- New conformance spec `specs/tri/t27b/conformance/float_as.t27`. It gives 5 pass under `t27c test-report`, and the same 5 under t27b.
- gen-zig picks the cast builtin from how the operand is spelled (`is_float_expr`). It treats these as a float:
  - a literal with a decimal point;
  - a parameter, local or struct field declared f16/f32/f64;
  - a binary expression with such a side.
- Such an operand is lowered:
  - to an integer, as `@intFromFloat`, toward zero, with its trap;
  - to a float, as `@floatCast`, nearest.
- A float operand gen-zig does not spell as one, such as a call result, is printed with `@intCast`, so the reference cannot compile it. t27b still refuses it as `ExprCast(f64)`. That is the negative control, and `differential.rs` now uses that shape for its refusal cases.
- Code is in `cli/t27b/src/lower.rs`, `lower/unanalyzed.rs` and the new `lower/floatas.rs`; tests are in `cli/t27b/tests/differential.rs`. All four files are listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- Not in this PR: `ExprCall(@log)` (`gen_cross_entropy.t27`). Zig 0.16 compiler_rt `log` for f64 is the table-based routine (128 `invc`/`logc` entries plus a second 128-entry table). It reads the float's bits, and t27b IR has no f64-to-u64 bitcast yet.
- Ledger `docs/reports/t27b_expectations.json`: `optimizer.t27`, `queen-needs-you.t27` and the two new specs move to pass. The not-pass count goes from 48 to 46, and pass from 488 to 492. On the lab corpus (`--jobs 2`, with the reference) the work tree gives jit/interp mismatch 0, reference disagree 0 and UNEXPECTED FAILURE 0.
