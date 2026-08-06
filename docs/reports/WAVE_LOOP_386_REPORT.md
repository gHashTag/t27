# Wave Loop 386 Close-Out Report

**Date:** 2026-07-01  
**Branch:** `wave-loop-385` (work continues on `trinity-rust-rings`)  
**Tracking issue:** #1282  
**Selected variant:** Variant B (proof push to 288 `ternaryMac` generic ∀ + `for` loop coverage over function-local arrays)  
**Commit:** (see `git log` on `trinity-rust-rings`)  

---

## Summary

Wave Loop 386 executed the recommended **Variant B** scope: it pushed the `ternaryMac` Lean 4 generic ∀ proof lattice from 284 to **288**, extended the IGLA CODER+RACE zero-failure streak to **120 waves**, and closed the **function-local array `for` loop** coverage gap.

The new regression specs `specs/scratch/w386_for_local_array.t27`, `w386_for_local_array_i8.t27`, and `w386_for_local_array_param.t27` exercise:
- constant-bound `for` loops that write to and read from local arrays,
- signed-element local arrays inside loops,
- parameter-bound `for` loops with variable-index array access.

The gen-verilog backend already correctly lowered these patterns: constant-bound loops are unrolled into scalar per-element assignments; parameter-bound loops are emitted as Verilog `for` loops with variable-index reads via priority mux chains and writes via if-else chains. No compiler backend change was required; the wave added regression coverage and CI smoke enforcement.

Full conformance reached **570/570 PASS**.

## Quantified results

| Metric | W385 | W386 | Δ |
|---|---|---|---|
| Lean 4 `ternaryMac` generic ∀ | 284 | **288** | +4 |
| Pool A floor | 129 | **130** | +1 |
| CODER minimum | 119 | **120** | +1 |
| Pool B depth (`systolic_ternary`) | 147 | **148** | +1 |
| Integration depth (`ternary_inference`) | 128 | **129** | +1 |
| Full-repo tests | 13,544 | **13,605** | +61 |
| Full-repo invariants | 5,962 | **5,989** | +27 |
| Conformance specs | 567 | **570** | +3 (scratch) |
| Conformance pass rate | 567/567 | **570/570** | 100% |
| Gen-verilog yosys smoke targets | 48 | **51** | +3 scratch specs |
| Zero-IGLA-failure streak | 119 waves | **120 waves** | +1 |

Test/invariant counts are from `t27c stats` and include all spec files (`specs/` and `compiler/`).

## New theorems in `proofs/lean4/Trinity/TernaryInference.lean`

Wave Loop 386 added **4** new `ternaryMac` generic ∀ theorems:

1. `ternaryMacAccumulateSixtyFourPlusGeneric` — 64-variable plus accumulation (**285 generic ∀ milestone**).
2. `ternaryMacAccumulateSixtyThreeMinusGeneric` — 63-variable minus accumulation lattice.
3. `ternaryMacQuadragintupleSexCancellationGeneric` — `mac^46(x, a, [.plus,.minus,...]) = x` (depth-46 identity cancellation; even depth collapses to identity).
4. `ternaryMacZeroWeightTwentyOnePairClosureGeneric` — 21 zero-weight MACs before and after a plus-weight MAC are transparent (**288 generic ∀ milestone**).

`lake build Trinity.TernaryInference` completed successfully.

## Gen-verilog: `for` loops over function-local arrays

### What was missing

Function-local arrays with variable-index access were implemented in W384, and signed element types plus array-literal initialization were added in W385. However, there was **no regression coverage** for using local arrays inside `for` loops, which is the natural next consumer of variable-index read/write.

### Observation

The existing backend already handled the pattern correctly:

- Constant-bound loops (e.g. `for i in 0..4`) are unrolled by the optimizer into scalar per-element assignments.
- Parameter-bound loops (e.g. `for i in 0..n`) remain as Verilog `for` loops, with:
  - reads lowered to a priority mux chain over the per-element regs,
  - writes lowered to an if-else chain selecting the matching element reg.

No compiler change was required.

### Regression evidence

Three new scratch specs were added:

- `specs/scratch/w386_for_local_array.t27` — constant-bound `for` loops over `[4]u16`, fill-and-sum and copy-reverse patterns.
- `specs/scratch/w386_for_local_array_i8.t27` — constant-bound `for` loops over `[4]i8`, signed sum and in-place negation.
- `specs/scratch/w386_for_local_array_param.t27` — parameter-bound `for` loops with variable-index write/read on `[4]u16`.

All three pass `t27c gen-verilog` + `yosys read_verilog -sv` + `synth`. The in-runner smoke gate now covers 51 targets (27 IGLA + 24 scratch).

## CI smoke gate

- The in-runner smoke gate covers all 27 IGLA specs plus 24 scratch specs = **51 targets**.
- New W386 scratch specs are part of the smoke set.

## FPGA / hardware status

- The board bring-up is **no longer blocked**. W385 successfully loaded a 200T-compatible bitstream via `openFPGALoader -c digilent_hs2`; `done 1` confirmed the FPGA accepted the SRAM load.
- `fpga/HARDWARE_SSOT.md` was updated to record the connected board as **XC7A200T-FGG676** (IDCODE `0x03636093`) and the cable as a **Digilent FTDI** device (`0x0403:0x6014`).
- No new hardware evidence was produced in W386; the canonical bitstream remains `fpga/verilog/ternary_mac_demo_top_200t.bit`.
- See `docs/reports/FPGA_EVIDENCE_W386.md` for the latest hardware note.

## Seal / conformance

- 27 core IGLA seals plus all other spec seals regenerated from `/Users/playra/t27` using `t27c seal --save`.
- Full suite result: **570/570 PASS**, zero seal mismatches, zero yosys smoke failures.

---

*φ² + 1/φ² = 3 | TRINITY*
