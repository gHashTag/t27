# Wave Loop 781 — Issue #1492

**Branch:** `wave-loop-781`
**Parent branch:** `wave-loop-780` HEAD
**Date:** 2026-07-24
**Issue:** #1492
**PR:** (to open after closeout)
**Cooperation variant:** A (recommended)
**Status:** implementation pending, plan ready

## Goal

Close Wave Loop 781 by validating a module-scope `[381][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block. In addition, fix the
three actionable weak points discovered in the 2026-07-24 audit so that
`cargo test --workspace`, `cargo clippy -p t27c`, and the bitnet pipeline gate
are green again.

Earlier wave PRs remain open awaiting review, so W781 will be branched from
`wave-loop-780` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [ ] `specs/scratch/w781_bench_module_381x2p6_aos_var_call_write.t27` is generated and parses.
2. [ ] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [ ] The cocotb reference model matches the t27 semantics.
4. [ ] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [ ] All cargo suites remain green, including `cargo test --workspace` and `cargo clippy -p t27c`.
6. [ ] Integration test `accepts_w781_bench_module_381x2p6_aos_var_call_write` is added.
7. [ ] Weak-point fixes applied:
   - `cli/flash-spi/src/main.rs` passes new `FlashOpts` fields (`bitswap`, `no_jprogram`).
   - `bootstrap/src/sensitivity.rs:126` uses `std::f64::consts::PI`.
   - `bootstrap/tests/bitnet_pipeline.rs:143` expects `IDLE: begin done<=0; if(start)`.
8. [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W781_2026-07-24.md` is written.
9. [ ] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
10. [ ] `.claude/plans/wave-loop-782.md` with three cooperation variants is created.
11. [ ] PR reviewed and merged to `master` (or stacked after earlier waves land).

## Technical notes

- Shape: `[381][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `381 x 64 = 24,384`.
- Packed vector width: `24,384 x 32 = 780,288` bits (~0.745 MiBit).
- `MID_IDX = 190`; frame-condition element `[190][1][0][0][0][0][0]` is element
  `190*64 + 32 = 12,192`.
- Generator script: `scripts/gen_w781.py` (copy from `scripts/gen_w780.py`, set
  `OUTER = 381` and `MID_IDX = 190`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[383][2]^6 Pt`.
- **Variant B:** keep width at ~0.745 MiBit but move the packed var to bench/function scope.
- **Variant C:** keep width and add `if`-guarded indexed signed field writes.
