# Wave Loop 783 — Issue #1495

**Branch:** `wave-loop-783`
**Parent branch:** `wave-loop-782` HEAD
**Date:** 2026-07-24
**Issue:** #1495
**PR:** (to open after closeout)
**Cooperation variant:** A (recommended)
**Status:** closeout complete

## Goal

Close Wave Loop 783 by validating a module-scope `[385][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block. In addition, fix the
one actionable weak point discovered in the 2026-07-24 audit so that
`cargo test -p t27c --test verilog_const_array` is green again.

Earlier wave PRs remain open awaiting review, so W783 will be branched from
`wave-loop-782` HEAD to avoid blocking the sequence.

## Acceptance criteria

1. [x] `specs/scratch/w783_bench_module_385x2p6_aos_var_call_write.t27` is generated and parses.
2. [ ] The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. [ ] The cocotb reference model matches the t27 semantics.
4. [ ] `t27c seal --save` succeeds and FROZEN_HASH remains unchanged.
5. [ ] All cargo suites remain green, including `cargo clippy -p t27c`.
6. [ ] Integration test `accepts_w783_bench_module_385x2p6_aos_var_call_write` is added.
7. [ ] Weak-point fix applied:
   - `bootstrap/tests/verilog_const_array.rs:166` accepts the current richer TODO marker.
8. [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W783_2026-07-24.md` is written.
9. [ ] Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
10. [ ] `.claude/plans/wave-loop-784.md` with three cooperation variants is created.
11. [x] PR reviewed and merged to `master` (or stacked after earlier waves land).

## Technical notes

- Shape: `[385][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `385 x 64 = 24,640`.
- Packed vector width: `24,640 x 32 = 788,480` bits (~0.752 MiBit).
- `MID_IDX = 192`; frame-condition element `[192][1][0][0][0][0][0]` is element
  `192*64 + 32 = 12,320`.
- Generator script: `scripts/gen_w783.py` (copy from `scripts/gen_w782.py`, set
  `OUTER = 385` and `MID_IDX = 192`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not
  emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[387][2]^6 Pt`.
- **Variant B:** keep width at ~0.752 MiBit but move the packed var to bench/function scope.
- **Variant C:** keep width and add `if`-guarded indexed signed field writes.
