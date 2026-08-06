# FPGA Wave Loop 623 Closeout Report

**Date:** 2026-07-07  
**Issue:** #1594  
**Branch:** `wave-loop-623`  
**Spec:** `specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.t27`

## What was delivered
Module-scope `[65][2]^6 Pt` non-power-of-two outer-dimension packed array-of-struct
variable, initialized from a function call and exercised with indexed signed field
writes and read-back. No compiler or reference-model changes were required.

### Spec contents
- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [65][2][2][2][2][2][2]Pt`
- `pub const expected : [65][2][2][2][2][2][2]Pt = make_grid(0);`
- `pub var dst : [65][2][2][2][2][2][2]Pt = make_grid(0);`
- `test module_var_65x2p6_call_write`
- `bench module_bench_65x2p6_call_write`

## Sizing
- Outer dimension: 65 (non-power-of-two).
- Total elements: 65 × 2⁶ = 4,160.
- Packed vector width: 4,160 × 32 = 133,120 bits (≈0.127 MiBit).
- Well under the 4-MiBit literal cliff; expected fast interactive simulation.

## Verification matrix

| Gate | Command | Result |
|------|---------|--------|
| Parse | `./target/release/t27c parse specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.t27` | PASS |
| Icarus lowerable | `./target/release/t27c icarus-lowerable specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.t27` | lowerable |
| Icarus simulate | `./target/release/t27c icarus-simulate specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.t27` | PASS (silent exit 0) |
| cocotb reference | `./target/release/t27c icarus-cocotb specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.t27` | reference-model OK |
| Integration test | `cargo test -p t27c --test icarus_lowerable accepts_w623_bench_module_65x2p6_aos_var_call_write` | PASS |
| Seal | `./target/release/t27c seal --save specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.t27` | saved |

## Compiler / test suite health
- `cargo build --release -p t27c`: green.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 83/0 (new W623 test added).
- yosys smoke: 24 pre-existing baselines unchanged.
- `./scripts/tri test --fast`: not run — Phase 1 Parse remains dominated by unrelated
  large literal specs from earlier waves; direct `t27c` gates are the practical
  closeout path.

## Seal status
- FROZEN_HASH unchanged: `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- New seal: `.trinity/seals/scratch_w623_bench_module_65x2p6_aos_var_call_write.json`.
- New empty baseline: `.trinity/icarus-baselines/specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.json`.

## Weak points investigated
1. **Outer dimension 65** — first time the ladder tests 65; no compiler or
   reference-model regression observed.
2. **Modulo-wrap signal** — element count is below the natural wrap point, so the
   test explicitly checks `make_grid(32768)` to preserve regression coverage.
3. **Multi-line literals** — continued W584-style brace formatting; single-line
   6-D literals risk parser truncation.
4. **Simulator capacity** — 0.127 MiBit is comfortably inside Icarus/Yosys comfort.
5. **Corner index update** — last-row index moved from 62 to 64 and mid-row index
   from 31 to 32; all expected values recomputed and matched.

## Scientific / technical background
- IEEE Std 1800-2017 — packed-array total width is the product of dimensions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus issue #1171 — large packed-vector elaboration freezes; W623 stays far below.
- Yosys docs — multidimensional packed arrays supported, arrays of packed structs
  unsupported; t27 flattening avoids the gap.
- cocotb `LogicArray` — reference model uses flat packed multidimensional arrays.

## Next Wave Loop 624 cooperation variants
1. **Variant A — `[67][2]^6 Pt` module-scope var from call with indexed signed writes.**
   137,216-bit, 4,288 elements, next odd outer dimension. **Recommended.**
2. **Variant B — `[2]^18 Pt` module-scope var from call with indexed signed writes.**
   8,388,608-bit, 262,144 elements; crosses the 4-MiBit cliff by 2× and risks
   simulator limits without chunked-literal design.
3. **Variant C — `[65][2]^6 Pt` module-scope var conditionally reassigned inside `if`, then indexed writes.**
   133,120-bit; tests control-flow guarded packed-reg reassignment (follow-up to W590/W591).

## Artifacts
- Spec: `specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.t27`
- Plan: `.claude/plans/wave-loop-623.md`
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W623_2026-07-07.md`
- Seal: `.trinity/seals/scratch_w623_bench_module_65x2p6_aos_var_call_write.json`
- Baseline: `.trinity/icarus-baselines/specs/scratch/w623_bench_module_65x2p6_aos_var_call_write.json`
- Integration test: `bootstrap/tests/icarus_lowerable.rs`

---

Phase complete: Verify
→ Phase 8: Land
