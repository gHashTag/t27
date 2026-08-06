# Wave Loop 785 Plan

**Date:** 2026-07-24
**Issue:** #1498 (to create)
**Parent branch:** `wave-loop-784` HEAD
**Recommended variant:** A

## Goal

Close Wave Loop 785 by validating a module-scope `[389][2]^6 Pt` packed
array-of-struct variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

## Shape

- `[389][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `389 x 64 = 24,896`.
- Packed vector width: `24,896 x 32 = 796,672` bits (~0.760 MiBit).
- `MID_IDX = 194`; frame-condition element `[194][1][0][0][0][0][0]` is element
  `194*64 + 32 = 12,448`.

## Acceptance criteria

1. `specs/scratch/w785_bench_module_389x2p6_aos_var_call_write.t27` is generated and parses.
2. The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. The cocotb reference model matches the t27 semantics.
4. `t27c seal --save` succeeds and `FROZEN_HASH` remains unchanged.
5. All cargo suites remain green, including `cargo clippy -p t27c`.
6. Integration test `accepts_w785_bench_module_389x2p6_aos_var_call_write` is added.
7. Closeout report and W786 cooperation variants are written.
8. Learning is saved to `.trinity/experience.md`, memory, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.
9. PR reviewed and merged to `master` (or stacked after earlier waves land).

## Technical notes

- Generator script: `scripts/gen_w785.py` (copy from `scripts/gen_w784.py`, set
  `OUTER = 389` and `MID_IDX = 194`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / `FROZEN_HASH` changes expected for the witness.

## Cooperation variants for Wave Loop 786

### Variant A (recommended): `[391][2]^6 Pt` module-scope var from call

Continue the odd outer-dimension ladder. This is the default unless the ring
selects a different focus.

### Variant B: `[389][2]^6 Pt` bench/function-scope packed var from call

Keep the W785 width but move `dst` inside a `bench` or function scope to test
local packed-array lowering.

### Variant C: `[389][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W785 width and add conditional indexed signed field writes to test
control-flow emission on the packed reg.

---

φ² + 1/φ² = 3 | TRINITY
