# NOW -- gen-c: arithmetic traps where t27 traps, and no C undefined behaviour (2026-10-04)

## gen-c arithmetic is defined, and agrees with Zig and t27b (Closes #5974)

- `t27c gen-c` wrote the bare C operators for `+ - * / % << >>`. In C a signed overflow, a shift amount outside `[0, width)`, a zero divisor and `MIN / -1` are undefined, so the t27b differential test (#5905) saw `inc(INT_MAX)` pass at -O0 and trap at -O2, and `shr(1024, 40)` do the opposite. The language rule is #1659: plain `+ - *` trap on overflow, `+% -% *%` wrap. Zig and t27b trap mode already did that.
- Inside a fn, test, bench or invariant body, every arithmetic operator now lowers to a `t27_*` helper from one guarded prelude block (`C_ARITH_PRELUDE`, emitted only when a module uses one). `+ - *` use `__builtin_*_overflow`, shifts check the amount, `/ %` check the zero divisor and `MIN / -1`, and the wrapping operators compute in unsigned arithmetic. A failure is `__builtin_trap()`, the same trap a failing `assert_eq` uses. `_Generic` picks the width, and a GNU statement expression evaluates each operand once.
- Unchanged on purpose: module-scope initializers (they must be C constant expressions), arithmetic whose operands are all literals (the C compiler folds and diagnoses it), and `+` on an array or string operand (concatenation, not arithmetic).
- A bare integer literal shifted by a runtime amount takes the declared width of the local it initializes, else its suffix, else u32/u64 by magnitude -- the rule the Zig backend already applies (`@as(u64, 1) << n`). Without it `(1 << n) - 1` with `n == 52` trapped in the 32-bit helper.
- `specs/igla/race/ternary_mac.t27`: the accumulator is `acc +% prod`. `tools/verify_igla_race.py` models it as a wrapping i32 and drives it across the INT32 edges, so checked `+` trapped there (as the Zig output already did). `tools/verify_igla_race.py` and `tools/check_duplicate_agreement.py` now carry the prelude into the functions they extract from the header.

## Measured, 2026-10-04

- Reproducer from the t27b README: `inc_max` traps, `shr_big` traps and `rem_neg` passes, identically at clang -O0 and -O2, and `-fsanitize=undefined` reports nothing. Before, -O0 and -O2 disagreed on two of the three and UBSan reported both.
- Corpus: 634 of 1186 specs change `gen-c` output. No spec that compiled cleanly stops compiling, and none starts. In the 229 specs that compile before and after, every test gives the same answer at -O0 and at -O2 except three. `gf_decode_param_fp64` `test_decode_neg_inf` now passes (UB had failed it). `ternary_memory` `test_ternary_word_write_read_trit` and `gf16_dot4` `test_intermediate_structure` now trap, and the Zig backend traps on both: an `@intCast` of -1 to u32, and a u16 multiply overflow.
- Verifiers: `verify_igla_race`, `check_duplicate_agreement`, `verify_multitarget` and `verify_trainer_c` pass.
- Seals: 899 seal files (536 specs) carry the new `gen_hash_c`; `check_seal_coverage` and `check_seal_currency` pass. 892 were resealed with `t27c seal --save`. Three seal files (`ternary_shift`, `gft_generalize_demo`) were sealed with `--force` because their Zig tests already fail on master (their `gen_hash_zig` is unchanged), and are ledgered as `tests-fail`. Four seal files (`clock_domain_tb`, `gf16_accel_tb`) had only `gen_hash_c` rewritten, because their own Zig test loops forever (`while !sync_ready`) and `seal --save` never returns.
- `cargo test --release -p t27c`: every test binary passes except `core_selfhost` (#6041, landed today). Its fixpoint requires the self-hosted core in `specs/compiler/core/t27core.t27` to write gen-c's bytes, and gen-c's bodies now call the `t27_*` helpers while the core still writes infix. The core has to learn the same lowering; this change does not edit it.
- `bootstrap/tests/c_checked_arith.rs`: 5 tests. Two check the emitted code, one checks that a module with no arithmetic gets no prelude, one runs 15 tests at -O0 and -O2 against the Zig/t27b answer, and one repeats that under UBSan. The runtime tests skip without a C compiler.

## Not established

- Types narrower than `int` (i8, u8, i16, u16) are promoted by C, so they are checked at `int` width and truncated on store. That is defined, but a u16 overflow that Zig traps only traps in C when it also overflows `int`.
- Unary minus is not checked, and pointer `+` is not supported: `+` with a pointer operand is now a compile error where it used to be a pointer offset.
