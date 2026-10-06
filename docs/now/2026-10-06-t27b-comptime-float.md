# NOW -- t27b: a comptime_float is a binary128 value (2026-10-06)

## comptime_float folding (Closes #7007)

- New conformance spec `specs/tri/t27b/conformance/comptime_float.t27`. Under `t27c test-report` it gives 6 pass, 13 runtime asserts and 0 vacuous. Under t27b it gives 6 pass with 14 runtime asserts. Six mutants each fail exactly the mutated test, in t27b and in the reference alike.
- In Zig, an untyped float literal is the nearest binary128 value. Each compile-time `+ - * /` rounds to binary128, comparisons are made in binary128, and the value is rounded once where an f64 or f32 is needed. t27b held the nearest f64 plus an exact flag and refused every inexact fold as `ExprBinary(*)`. `Val::Cf` now holds `float::Q`, which is computed exactly with integers and rounded to nearest even. The new code is in `cli/t27b/src/lower_float.rs`, and its call sites are in `cli/t27b/src/lower.rs`.
- Unit oracle: 113 >= 2 * 53 + 2, so rounding to binary128 and then to f64 equals the direct f64 rounding. 2000 random f64 pairs, one fifth of them subnormal or tiny, must give the hardware result bit for bit for all four operations, for comparison and for the f32 cast. The exact decimal expansion of 300 random f64s must also parse back to the same value.
- Still refused, which is conservative: a folded value outside the f64 range (`1e308 * 10.0` typed f64). The reference accepts it as infinity. Also refused: a division by a constant zero, `@sqrt` of a comptime value, and an untyped `var` holding a float.
- Lab numbers on branch 747c62f48, with mismatch 0, jit_interp_mismatch 0, reference_disagree 0 and fuzz T27B_BUG 0, compared with master 38a6e30ca run on the same lab at the same time:
  - The family gains `pysr_trinity_blind_test_v2`, `verify_smoking_guns` and the new spec.
  - The branch run timed out on `kernel_fib`, `kernel_matmul` and `d_g22_test`. None of them contains a float, and all of them pass when re-run alone on both binaries. The timeouts came from two labs running at once.
  - Raw counts are 525 for master and 525 for the branch. Counted by file, that is 525 -> 528.
- Ledger: the three files become pass, and the cap goes from 53 to 52.
