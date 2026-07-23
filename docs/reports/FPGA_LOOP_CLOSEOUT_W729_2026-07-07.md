# FPGA / Simulation Wave Loop 729 Closeout

**Issue:** #1700  
**Branch:** `wave-loop-729`  
**Date:** 2026-07-07  
**Previous:** Wave Loop 728 (#1699, branch `wave-loop-728`)

## Chosen variant

**Variant A — module-scope `[277][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable initialized from a function call, with indexed signed field writes.**

Witness: `specs/scratch/w729_bench_module_277x2p6_aos_var_call_write.t27`.

```t27
pub struct Pt { x : i16, y : i16 }
pub fn make_grid(offset : u16) -> [277][2][2][2][2][2][2] Pt { ... }
pub const expected : [277][2][2][2][2][2][2] Pt = make_grid(0);
pub var dst : [277][2][2][2][2][2][2] Pt = make_grid(0);

test module_var_277x2p6_call_write { ... }
bench module_bench_277x2p6_call_write { ... }
```

This continues the module-scope packed AoS odd outer-dimension ladder from 275
(W728) to 277. Total vector width: **277 × 64 × 32 = 598,912 bits**
(17,792 elements, ≈0.571 MiBit), still well under the ~4-MiBit Icarus/Yosys
comfort threshold.

## What we validated

- A module-level `pub var dst : [277][2]^6 Pt` can be initialized from a function
  call and exercised with indexed signed field writes, with **zero compiler
  changes**.
- The structural classifier (`t27c icarus-lowerable`) accepts the witness.
- Icarus Verilog simulation (`t27c icarus-simulate`) completes in 17 cycles and
  reports `PASSED`.
- The cocotb/Python reference model (`t27c icarus-cocotb`) matches the
  hardware simulation, confirming row-major flattening with outer stride 277 is
  preserved end-to-end.
- The seal hash was saved, the Icarus baseline was recorded, and all
  conformance suites pass.

## Key numbers

| Metric | Value |
|--------|-------|
| Outer dimension | 277 (non-power-of-two) |
| Total elements | 277 × 2⁶ = 17,792 |
| Packed vector width | 598,912 bits |
| Approximate size | ≈0.571 MiBit |
| Simulation cycles | 17 |
| `t27c --bin t27c` tests | 1494 passed; 0 failed; 2 ignored |
| `tri` tests | 78 passed; 0 failed |
| `icarus_lowerable` tests | 189 passed; 0 failed |
| Compiler changes | 0 |
| Reference-model changes | 0 |
| FROZEN_HASH change | none |

## Files added or modified

- `scripts/gen_w729.py` — new generator script, copied from `gen_w728.py` with
  `OUTER = 277` and `MID_IDX = 138`.
- `specs/scratch/w729_bench_module_277x2p6_aos_var_call_write.t27` — new witness
  (~1,216 KB, ~52,691 lines).
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w729_bench_module_277x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w729_bench_module_277x2p6_aos_var_call_write.json` —
  saved by `t27c seal --save`.
- `.trinity/icarus-baselines/specs/scratch/w729_bench_module_277x2p6_aos_var_call_write.json` —
  empty baseline.
- `.trinity/current-issue.md` — updated for #1700.
- `.claude/plans/wave-loop-729.md` — decomposed 10-task plan with risk register.
- `.trinity/experience.md` — W729 learnings prepended.
- This report: `docs/reports/FPGA_LOOP_CLOSEOUT_W729_2026-07-07.md`.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Graham, Accellera `vlog-pp` discussion (2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W729
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals;
  flat `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Weak points observed

1. **`assert_ne` gap.** The structural classifier accepts `assert_ne`, but the
   Icarus simulation emitter only lowers `assert_eq`. W729 continues to use
   `assert_eq` on changed elements instead of whole-array inequality.
2. **Generator drift.** Copying `gen_W.py` to `gen_W+1.py` and running `sed`
   is fast but requires a manual `MID_IDX` comment correction each time.
3. **Modulo-wrap regression signal is artificial below 16,384 elements.** With
   17,792 elements, the offset-0 schedule never wraps at 32768, so the test
   keeps an explicit `make_grid(32768)` call to retain the wrap check.
4. **No systematic wall-clock limit test yet.** We are approaching 0.57 MiBit
   comfortably, but a stress wave near the 4-MiBit boundary remains on the
   backlog.

## Next Wave Loop 730 cooperation variants

1. **Variant A (recommended): `[279][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 602,624-bit packed vector, 17,856 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 279.
   - **Recommended.**

2. **Variant B: `[277][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W729 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[277][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.571 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done status

- [x] Issue #1700 current-issue and plan written.
- [x] Generator and witness created.
- [x] Integration test added and passing.
- [x] Release build and all direct gates passing.
- [x] Test suites passing.
- [x] Seal and Icarus baseline saved.
- [x] Closeout report and experience/memory updated.
- [x] Branch `wave-loop-730` created and W729 committed with `Closes #1700`.

## Conclusion

Wave Loop 729 successfully validated a 598,912-bit module-scope packed
array-of-structs with non-power-of-two outer dimension 277, initialized from a
function call and mutated via indexed signed field writes. No compiler or
reference-model changes were required. The ladder can continue to 279 and beyond
while staying under the 4-MiBit simulation ceiling.

---

φ² + 1/φ² = 3 | TRINITY
