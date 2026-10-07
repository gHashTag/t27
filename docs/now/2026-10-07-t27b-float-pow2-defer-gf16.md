# NOW -- t27b: float x*2^k, if in a struct literal, defer, gf16::GF16 (2026-10-07)

## t27b coverage lane 2 (Closes #7239)

Four blockers fixed. A conformance spec came first for each; under `t27c test-report`, t27b now gives the same verdicts as the reference.

- **ExprBinary(f64 * 2^k).** t27c's strength reduction rewrites a float `x * 2^k` into a shift, which Zig refuses. t27b refused every such product. It now refuses one only where that rewrite reaches: the top-level value of an assignment, local or return in a module-level fn, through binary operators. Spec: `specs/tri/t27b/conformance/float_mul_pow2.t27`.
- **ExprIf(left operand).** An `if` as a struct literal field value prints as `.f = v,`, and Zig accepts it. Spec: `struct_lit_if.t27`.
- **StmtExpr statement.** gen-zig renders `defer <stmt>;` to nothing (T43): the deferred call neither runs nor is analyzed. t27b now drops it the same way. Spec: `scope_exit.t27`.
- **type gf16::GF16.** The type mapper (#6533) maps the scoped path to u16. `@as(gf16::GF16, x)` and `*gf16::GF16` are still refused, as the reference refuses them. Spec: `scoped_gf16.t27`.

Measured with the full corpus on the t27b Railway lab at `--jobs 2`:
- mismatch 0, jit/interp mismatch 0, reference disagree 0, crash 0;
- ratchet UNEXPECTED FAILURE 0.

Four specs move up to pass: `specs/compiler/zig_scoped_type.t27`, `ternary_mac_demo_top.t27`, `gen_gradient.t27` and `gen_gradient_descent.t27`. The four new conformance specs also pass.

The ledger is master's plus these 8 moves:
- pass goes from 488 to 496;
- not_pass and the cap go from 48 to 45.

Rust in `cli/t27b/src/lower.rs` is under the owner's `owner-approved-foreign` approval on #6063.
