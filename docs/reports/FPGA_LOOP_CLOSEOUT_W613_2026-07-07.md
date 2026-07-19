# FPGA Wave Loop 613 Closeout Report

**Date:** 2026-07-07  
**Issue:** #1584  
**Branch:** `wave-loop-613`  
**Spec:** `specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.t27`

## What was delivered
Module-scope `[45][2]^6 Pt` non-power-of-two outer-dimension packed array-of-struct
variable, initialized from a function call and exercised with indexed signed field
writes and read-back. No compiler or reference-model changes were required.

### Spec contents
- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [45][2]^6 Pt`
- `pub const expected : [45][2]^6 Pt = make_grid(0);`
- `pub var dst : [45][2]^6 Pt = make_grid(0);`
- `test module_var_45x2p6_call_write`
- `bench module_bench_45x2p6_call_write`

## Sizing
- Outer dimension: 45 (non-power-of-two).
- Total elements: 45 × 2⁶ = 2,880.
- Packed vector width: 2,880 × 32 = 92,160 bits (≈0.088 MiBit).
- Well under the 4-MiBit literal cliff; expected fast interactive simulation.

## Verification matrix

| Gate | Command | Result |
|------|---------|--------|
| Parse | `./target/release/t27c parse specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.t27` | PASS |
| Icarus lowerable | `./target/release/t27c icarus-lowerable specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.t27` | lowerable |
| Icarus simulate | `./target/release/t27c icarus-simulate specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.t27` | PASS (silent exit 0) |
| cocotb reference | `./target/release/t27c icarus-cocotb specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.t27` | reference-model OK |
| Integration test | `cargo test -p t27c --test icarus_lowerable accepts_w613_bench_module_45x2p6_aos_var_call_write` | PASS |
| Seal | `./target/release/t27c seal --save specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.t27` | saved |

## Compiler / test suite health
- `cargo build --release -p t27c`: green.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 73/0 (new W613 test added).
- yosys smoke: 24 pre-existing baselines unchanged.
- `./scripts/tri test --fast`: not run — Phase 1 Parse remains dominated by unrelated
  large literal specs from earlier waves; direct `t27c` gates are the practical
  closeout path.

## Seal status
- FROZEN_HASH unchanged: `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- New seal: `.trinity/seals/scratch_w613_bench_module_45x2p6_aos_var_call_write.json`.
- New empty baseline: `.trinity/icarus-baselines/specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.json`.

## Weak points investigated
1. **Outer dimension 45** — first time the ladder tests 45; no compiler or
   reference-model regression observed.
2. **Modulo-wrap signal** — element count is below the natural wrap point, so the
   test explicitly checks `make_grid(32768)` to preserve regression coverage.
3. **Multi-line literals** — continued W584-style brace formatting; single-line
   6-D literals risk parser truncation.
4. **Simulator capacity** — 0.088 MiBit is comfortably inside Icarus/Yosys comfort.

## Scientific / technical background
- IEEE Std 1800-2017 — packed-array total width is the product of dimensions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus issue #1171 — large packed-vector elaboration freezes; W613 stays far below.
- Yosys docs — multidimensional packed arrays supported, arrays of packed structs
  unsupported; t27 flattening avoids the gap.
- cocotb `LogicArray` — reference model uses flat packed multidimensional arrays.

## Next Wave Loop 614 cooperation variants
1. **Variant A — `[47][2]^6 Pt` module-scope var from call with indexed signed writes.**
   96,256-bit, 3,008 elements, next odd outer dimension. **Recommended.**
2. **Variant B — `[2]^18 Pt` module-scope var from call with indexed signed writes.**
   8,388,608-bit, 262,144 elements; crosses the 4-MiBit cliff by 2× and risks
   simulator limits without chunked-literal design.
3. **Variant C — `[45][2]^6 Pt` conditional whole-array reassignment inside `if`, then indexed writes.**
   92,160-bit; tests control-flow guarded packed-reg reassignment (follow-up to W590/W591).

## Artifacts
- Spec: `specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.t27`
- Plan: `.claude/plans/wave-loop-613.md`
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W613_2026-07-07.md`
- Seal: `.trinity/seals/scratch_w613_bench_module_45x2p6_aos_var_call_write.json`
- Baseline: `.trinity/icarus-baselines/specs/scratch/w613_bench_module_45x2p6_aos_var_call_write.json`
- Integration test: `bootstrap/tests/icarus_lowerable.rs`

---

Phase complete: Verify
→ Phase 8: Land
