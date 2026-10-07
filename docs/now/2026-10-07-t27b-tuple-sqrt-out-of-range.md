# NOW -- t27b: tuple locals, @sqrt, out-of-range compares, unreached &repeat (2026-10-07)

## t27b coverage lane 2 (Closes #7244)

Four blockers fixed. A conformance spec came first for each; under `t27c test-report`, t27b now gives the same verdicts as the reference.

- **ExprTuple.** gen-zig prints `const t = .{ a, b };` as a Zig tuple of the run-time values, read at constant indices. t27b now lays such a local out as a tuple struct in a stack slot, its fields evaluated once and in order. Only run-time scalar fields are taken; a literal field is still refused. Spec: `specs/tri/t27b/conformance/tuple_local.t27`.
- **@sqrt of a run-time float.** Zig's `@sqrt` is the correctly rounded IEEE-754 square root. t27b emits AArch64 `FSQRT` (with its encoder case) and the interpreter uses the host's `sqrt`. Spec: `float_sqrt.t27`.
- **ExprBinary (`&[_]T{...} ** n`).** Zig refuses this once it analyzes it, but it never analyzes a fn that only a bench names. t27b now lowers it to the stub trap there, as for other code no test can reach. Spec: `repeat_addr_unreached.t27`.
- **literal out of range.** Zig settles a run-time uN compared with a comptime_int outside uN's range at compile time, and still evaluates the run-time side. t27b now does the same. Arithmetic with such a literal (`x + (1 << 32)` on a u32) is still refused, as the reference refuses it. Spec: `cmp_out_of_range.t27`.

Measured with the full corpus (1362 of 1362 files) on the t27b Railway lab at `--jobs 2`, with the reference:
- mismatch 0, jit/interp mismatch 0, reference disagree 0 (812 compared), crash 0;
- ratchet UNEXPECTED FAILURE 0.

Three specs move up to pass: `gen_ray.t27` (ExprTuple and `@sqrt`), `d_f19_test.t27` (ExprBinary) and `gft_dup2_jtag.t27` (literal out of range). The four new conformance specs also pass. `specs/ml/layers/layernorm_layer.t27` gets past `@sqrt` and stays blocked, now on `StmtAssign(undeclared)`.

The ledger is master's plus these 7 moves to pass and the one new blocker:
- pass goes from 490 to 497;
- not_pass and the cap go from 47 to 44.

The wide integer types (`u128` and up) are parked as #7245.

Rust in `cli/t27b/src` and `cli/t27b/tests` is under the owner's `owner-approved-foreign` approval on #6063 and #7244.
