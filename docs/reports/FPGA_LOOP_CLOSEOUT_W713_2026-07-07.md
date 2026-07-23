# FPGA Wave Loop 713 Closeout

**Date:** 2026-07-07
**Issue:** #1684
**Branch:** `wave-loop-713`
**Previous:** Wave Loop 712 (#1683, `wave-loop-712`)
**Witness:** `specs/scratch/w713_bench_module_245x2p6_aos_var_call_write.t27`

## What was validated

Wave Loop 713 extended the module-scope packed array-of-structs (AoS) odd
outer-dimension ladder to **245**. The witness declares:

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [245][2][2][2][2][2][2] Pt`
- `pub const expected : [245][2]^6 Pt = make_grid(0);`
- `pub var dst : [245][2]^6 Pt = make_grid(0);`
- A `test` block verifying initial state, corner indexed reads, and an explicit
  modulo-wrap check via `make_grid(32768)`.
- A `bench` block with whole-array equality, indexed reads, signed indexed field
  writes, read-back, and frame-condition checks.

Total width: **508,288 bits** (15,680 elements, ≈0.483 MiBit), still well below
the ~4-MiBit Icarus/Yosys comfort threshold. No compiler or reference-model
changes were required.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4 — packed-array width is the product of dimensions;
  ranges need not be powers of two.
- Accellera / Graham 2002 — packed arrays as contiguous bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus issue #1134 — assertion failures with unpacked arrays of packed structs;
  t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors;
  W713 stays far below the reported threshold.
- Yosys issue #2677 / #4653 — arrays of packed structs unsupported; t27
  flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Decomposition and execution

| # | Task | Result |
|---|------|--------|
| 1 | Update `.trinity/current-issue.md` for #1684 | Done |
| 2 | Create `.claude/plans/wave-loop-713.md` | Done |
| 3 | Create `scripts/gen_w713.py` and generate witness | Done |
| 4 | Add `accepts_w713_...` integration test | Done |
| 5 | Build `t27c` release | PASS |
| 6 | Direct gates: parse, icarus-lowerable, icarus-simulate, icarus-cocotb, seal | PASS |
| 7 | Run cargo test suites | PASS |
| 8 | Create Icarus baseline | Done |
| 9 | Update `.trinity/experience.md` and persistent memory | Done |
| 10 | Create `wave-loop-714` branch | Done |

## Validation results

- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 173 passed; 0 failed.
- `t27c parse` W713: PASS.
- `t27c icarus-lowerable` W713: PASS (`lowerable`).
- `t27c icarus-simulate` W713: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W713: PASS (`reference-model OK`).
- `t27c seal --save` W713: saved.
- `FROZEN_HASH`: unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

## Key learnings

- The odd outer-dimension ladder (3, 5, 7, ..., 243, 245) continues to work at
  module scope with zero compiler changes, confirming that the t27 packed-vector
  lowering and cocotb reference model handle non-power-of-two strides correctly.
- At 0.483 MiBit, Icarus simulation still completes in 17 cycles and remains
  comfortably interactive.
- The explicit `make_grid(32768)` modulo-wrap check is still required because the
  offset-0 schedule `(2*e + offset) % 32768` does not wrap for 15,680 elements
  (max raw value 31,359).
- `assert_ne` remains accepted by the structural classifier but not emitted by the
  Icarus simulation path; benches must use `assert_eq` on changed elements.
- When copying `scripts/gen_wPREV.py` to `scripts/gen_wNEXT.py`, the
  `MID_IDX = OUTER // 2  # NNN` comment must be manually corrected after `sed`.

## Next Wave Loop 714 cooperation variants

1. **Variant A (recommended) — `[247][2]^6 Pt` module-scope var from a call with indexed signed
   writes.**
   514,432-bit packed vector, 15,952 elements, non-power-of-two outer dimension 247,
   still well under the 4-MiBit cliff. Continues the odd outer-dimension ladder.
   **Recommended.**

2. **Variant B — `[245][2]^6 Pt` bench-local (function-local) packed array var
   from a call with indexed signed writes.**
   508,288-bit packed vector, 15,680 elements. Tests that the same non-p2 outer
   dimension works when the mutable `reg` is declared inside a bench/function
   rather than at module scope. Useful complement to the module-scope ladder.

3. **Variant C — `[245][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   Stays at 0.483 MiBit and tests that control-flow guarded indexed writes on a
   packed `reg` are correctly elaborated and simulated (e.g. write only when a
   signed index exceeds a threshold). Useful follow-up to W590/W591.

## Sign-off

Wave Loop 713 is complete. Issue #1684 closed. Branch `wave-loop-714` exists for
continued work. All invariant laws satisfied:
- L1 TRACEABILITY: commits include `Closes #1684`.
- L2 GENERATION: new files under `gen/` were not hand-edited.
- L3 PURITY: ASCII-only identifiers.
- L4 TESTABILITY: witness contains `test` and `bench` blocks.
- L5 IDENTITY: φ² = φ + 1; φ² + φ⁻² = 3.
- L6 CEILING: numeric SSOT unchanged.
- L7 UNITY: no new `*.sh` on the critical path.
