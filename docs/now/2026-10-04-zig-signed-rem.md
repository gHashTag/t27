# NOW -- gen (Zig): signed and float % lowers to @rem (2026-10-04)

## A signed remainder compiles, and truncates (Closes #5973)

- `t27c gen` wrote `%` as Zig's infix `%` whatever its operands. Zig 0.16 accepts infix `%` only when both operands are unsigned and refuses everything else: "signed integers and floats must use @rem or @mod". So `fn rem(a: i32, b: i32) -> i32 { return a % b; }` made the whole generated file fail to compile. Found by the t27b differential test (#5905).
- The fix is in `Codegen::gen_expr` (`ExprBinary`), next to the W593 `@divTrunc` arm for `/`, and uses the same test: when either operand is a known signed integer (`is_signed_int_expr`) or a float (`is_float_expr`), `%` becomes `@rem(a, b)`. `@rem` is the truncated remainder, so `rem(-7, 2) == -1`, the same answer C, Rust and t27b give. `@mod` would floor and give 1. When both operands are unsigned, `%` stays `%`.
- `/` needed nothing. W593 already writes `@divTrunc` for a signed `/`.

## Measured, 2026-10-04

- Reproducer from the t27b README under `zig test`: before, the file did not compile. After, `rem_neg` passes. `inc_max` panics with "integer overflow" and `shr_big` with "integer does not fit in destination type". Those two are t27 trap semantics, and the Zig backend already had them.
- Corpus: 8 of 1184 specs change `gen` output, and no spec changes `gen-c` output. In each of the 8, the number of `zig` compile errors went down or stayed the same: `compiler/optimizer.t27` 1 -> 0 (its 14 tests now pass), `igla/race/backend.t27` 49 -> 48, `isa/tri27_machine.t27` 5 -> 4, `vsa/trinity_compat.t27` 6 -> 5. The others are unchanged, and no spec has a new error.
- 12 seals resealed with `t27c seal <spec> --save`. `tools/check_seal_coverage.py`: OK, 1448 seals.
- `bootstrap/tests/zig_signed_rem.rs`: 4 tests. Three check the emitted code: signed `%` becomes `@rem` and never `@mod`, unsigned `%` stays infix, and float `%` becomes `@rem`. The fourth runs `zig test` and skips when zig is not on PATH. `cargo test --release -p t27c`: 2812 passed, 0 failed.

## Not established

- What `@rem` does with a zero divisor or `MIN % -1` in the generated code. Neither case is tested here.
