# Wave Loop 782 — Issue #1493

**Branch:** `wave-loop-782`
**Parent branch:** `wave-loop-781` HEAD
**Date:** 2026-07-24
**Issue:** #1493
**PR:** (to open after closeout)
**Cooperation variant:** A (recommended)
**Status:** closeout complete

## Goal

Close Wave Loop 782 by validating a module-scope `[383][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block. In addition, fix the
one actionable weak point discovered in the 2026-07-24 audit so that
`cargo clippy --all-targets` is green again.

Earlier wave PRs remain open awaiting review, so W782 will be branched from
`wave-loop-781` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [ ] `specs/scratch/w782_bench_module_383x2p6_aos_var_call_write.t27` is generated and parses.
2. [ ] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [ ] The cocotb reference model matches the t27 semantics.
4. [ ] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [ ] All cargo suites remain green, including `cargo clippy --all-targets`.
6. [ ] Integration test `accepts_w782_bench_module_383x2p6_aos_var_call_write` is added.
7. [ ] Weak-point fix applied:
   - `bootstrap/src/host/telemetry.rs:242` uses `std::f64::consts::PI` instead of literal `3.14`.
8. [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W782_2026-07-24.md` is written.
9. [ ] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
10. [ ] `.claude/plans/wave-loop-783.md` with three cooperation variants is created.
11. [ ] PR reviewed and merged to `master` (or stacked after earlier waves land).

## Technical notes

- Shape: `[383][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `383 x 64 = 24,512`.
- Packed vector width: `24,512 x 32 = 784,384` bits (~0.748 MiBit).
- `MID_IDX = 191`; frame-condition element `[191][1][0][0][0][0][0]` is element
  `191*64 + 32 = 12,256`.
- Generator script: `scripts/gen_w782.py` (copy from `scripts/gen_w781.py`, set
  `OUTER = 383` and `MID_IDX = 191`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[385][2]^6 Pt`.
- **Variant B:** keep width at ~0.748 MiBit but move the packed var to bench/function scope.
- **Variant C:** keep width and add `if`-guarded indexed signed field writes.
