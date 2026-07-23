# FPGA / Simulation Wave Loop 733 Closeout

**Issue:** #1704  
**Branch:** `wave-loop-733`  
**Date:** 2026-07-07  
**Previous:** Wave Loop 732 (#1703, branch `wave-loop-732`)

## Chosen variant

**Variant A — module-scope `[285][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable initialized from a function call, with indexed signed field writes.**

Witness: `specs/scratch/w733_bench_module_285x2p6_aos_var_call_write.t27`.

```t27
pub struct Pt { x : i16, y : i16 }
pub fn make_grid(offset : u16) -> [285][2][2][2][2][2][2] Pt { ... }
pub const expected : [285][2][2][2][2][2][2] Pt = make_grid(0);
pub var dst : [285][2][2][2][2][2][2] Pt = make_grid(0);

test module_var_285x2p6_call_write { ... }
bench module_bench_285x2p6_call_write { ... }
```

This continues the module-scope packed AoS odd outer-dimension ladder from 283
(W732) to 285. Total vector width: **285 × 64 × 32 = 583,680 bits**
(18,240 elements, ≈0.557 MiBit), still well under the ~4-MiBit Icarus/Yosys
comfort threshold.

## What we validated

- A module-level `pub var dst : [285][2]^6 Pt` can be initialized from a function
  call and exercised with indexed signed field writes, with **zero compiler
  changes**.
- The structural classifier (`t27c icarus-lowerable`) accepts the witness.
- Icarus Verilog simulation (`t27c icarus-simulate`) completes in 17 cycles and
  reports `PASSED`.
- The cocotb/Python reference model (`t27c icarus-cocotb`) matches the
  hardware simulation, confirming row-major flattening with outer stride 285 is
  preserved end-to-end.
- The seal hash was saved, the Icarus baseline was recorded, and all
  conformance suites pass.

## Key numbers

| Metric | Value |
|--------|-------|
| Outer dimension | 285 (non-power-of-two) |
| Total elements | 285 × 2⁶ = 18,240 |
| Packed vector width | 583,680 bits |
| Approximate size | ≈0.557 MiBit |
| Simulation cycles | 17 |
| `t27c --bin t27c` tests | 1494 passed; 0 failed; 2 ignored |
| `tri` tests | 78 passed; 0 failed |
| `icarus_lowerable` tests | 193 passed; 0 failed |
| Compiler changes | 0 |
| Reference-model changes | 0 |
| FROZEN_HASH change | none |

## Files added or modified

- `scripts/gen_w733.py` — new generator script, copied from `gen_w732.py` with
  `OUTER = 285` and `MID_IDX = 142`.
- `specs/scratch/w733_bench_module_285x2p6_aos_var_call_write.t27` — new witness
  (~1,250 KB, ~54,211 lines).
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w733_bench_module_285x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w733_bench_module_285x2p6_aos_var_call_write.json` —
  saved by `t27c seal --save`.
- `.trinity/icarus-baselines/specs/scratch/w733_bench_module_285x2p6_aos_var_call_write.json` —
  empty baseline.
- `.trinity/current-issue.md` — updated for #1704.
- `.claude/plans/wave-loop-733.md` — decomposed 10-task plan with risk register.
- `.trinity/experience.md` — W733 learnings prepended.
- This report: `docs/reports/FPGA_LOOP_CLOSEOUT_W733_2026-07-07.md`.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Graham, Accellera `vlog-pp` discussion (2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W733
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
   Icarus simulation emitter only lowers `assert_eq`. W733 continues to use
   `assert_eq` on changed elements instead of whole-array inequality.
2. **Generator header f-string drift.** The module header in `gen_W.py` uses an
   f-string (`{OUTER}`). A naive global `wN -> wN+1` replacement misses it, so the
   copy/edit workflow must explicitly fix the module name. This was verified by
   `t27c parse` in the W733 workflow before any simulation gate was run.
3. **Offset-32768 check is a period-identity check, not a first-time wrap test.**
   With 18,240 elements, the offset-0 schedule already wraps naturally
   (last raw `x = 2·18239 = 36478`, `36478 mod 32768 = 3710`). Adding offset
   32768 is congruent to adding 0 modulo 32768, so `make_grid(32768)` returns
   exactly the same values as `make_grid(0)`. The test still exercises the
   modulo operator on every element and verifies period identity, but a future
   wave could use a non-trivial offset (e.g., 1 or 16383) to stress true wrap
   behavior.
4. **No systematic wall-clock limit test yet.** At 0.557 MiBit we remain
   comfortable, but a stress wave near the 4-MiBit boundary remains on the
   backlog.
5. **Manual `MID_IDX` correction.** Each generator copy still needs a manual
   update of the `MID_IDX` comment, which is error-prone.

## Next Wave Loop 734 cooperation variants

1. **Variant A (recommended): `[287][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 587,776-bit packed vector, 18,368 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 287.
   - **Recommended.**

2. **Variant B: `[285][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W733 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[285][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.557 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done status

- [x] Issue #1704 current-issue and plan written.
- [x] Generator and witness created.
- [x] Integration test added and passing.
- [x] Release build and all direct gates passing.
- [x] Test suites passing.
- [x] Seal and Icarus baseline saved.
- [x] Closeout report and experience/memory updated.
- [ ] Branch `wave-loop-734` created and W733 committed with `Closes #1704`.

## Conclusion

Wave Loop 733 successfully validated a 583,680-bit module-scope packed
array-of-structs with non-power-of-two outer dimension 285, initialized from a
function call and mutated via indexed signed field writes. No compiler or
reference-model changes were required. The ladder can continue to 287 and beyond
while staying under the 4-MiBit simulation ceiling.

---

φ² + 1/φ² = 3 | TRINITY
