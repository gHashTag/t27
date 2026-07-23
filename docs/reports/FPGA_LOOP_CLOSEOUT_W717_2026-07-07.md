# FPGA Wave Loop 717 Closeout

**Date:** 2026-07-07
**Issue:** #1688
**Branch:** `wave-loop-717`
**Previous:** Wave Loop 716 (#1687, `wave-loop-716`)
**Witness:** `specs/scratch/w717_bench_module_253x2p6_aos_var_call_write.t27`

## What was validated

Wave Loop 717 extended the module-scope packed array-of-structs (AoS) odd
outer-dimension ladder to **253**. The witness declares:

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [253][2][2][2][2][2][2] Pt`
- `pub const expected : [253][2]^6 Pt = make_grid(0);`
- `pub var dst : [253][2]^6 Pt = make_grid(0);`
- A `test` block verifying initial state, corner indexed reads, and an explicit
  modulo-wrap check via `make_grid(32768)`.
- A `bench` block with whole-array equality, indexed reads, signed indexed field
  writes, read-back, and frame-condition checks.

Total width: **533,248 bits** (16,288 elements, ≈0.508 MiBit), still well below
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
  W717 stays far below the reported threshold.
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
| 1 | Update `.trinity/current-issue.md` for #1688 | Done |
| 2 | Create `.claude/plans/wave-loop-717.md` | Done |
| 3 | Create `scripts/gen_w717.py` and generate witness | Done |
| 4 | Add `accepts_w717_...` integration test | Done |
| 5 | Build `t27c` release | PASS |
| 6 | Direct gates: parse, icarus-lowerable, icarus-simulate, icarus-cocotb, seal | PASS |
| 7 | Run cargo test suites | PASS |
| 8 | Create Icarus baseline | Done |
| 9 | Update `.trinity/experience.md` and persistent memory | Done |
| 10 | Create `wave-loop-718` branch | Done |

## Validation results

- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 177 passed; 0 failed.
- `t27c parse` W717: PASS.
- `t27c icarus-lowerable` W717: PASS (`lowerable`).
- `t27c icarus-simulate` W717: PASS (17 cycles, PASSED).
- `t27c icarus-cocotb` W717: PASS (`reference-model OK`).
- `t27c seal --save` W717: saved.
- `FROZEN_HASH`: unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

## Key learnings

- The odd outer-dimension ladder (3, 5, 7, ..., 251, 253) continues to work at
  module scope with zero compiler changes, confirming that the t27 packed-vector
  lowering and cocotb reference model handle non-power-of-two strides correctly.
- At 0.508 MiBit, Icarus simulation still completes in 17 cycles and remains
  comfortably interactive.
- The explicit `make_grid(32768)` modulo-wrap check is still required because the
  offset-0 schedule `(2*e + offset) % 32768` does not wrap for 16,288 elements
  (max raw value 32,575).
- `assert_ne` remains accepted by the structural classifier but not emitted by the
  Icarus simulation path; benches must use `assert_eq` on changed elements.
- When copying `scripts/gen_wPREV.py` to `scripts/gen_wNEXT.py`, the
  `MID_IDX = OUTER // 2  # NNN` comment must be manually corrected after `sed`.

## Next Wave Loop 718 cooperation variants

1. **Variant A (recommended) — `[255][2]^6 Pt` module-scope var from a call with indexed signed
   writes.**
   539,520-bit packed vector, 16,400 elements, non-power-of-two outer dimension 255,
   still well under the 4-MiBit cliff. Continues the odd outer-dimension ladder.
   **Recommended.**

2. **Variant B — `[253][2]^6 Pt` bench-local (function-local) packed array var
   from a call with indexed signed writes.**
   533,248-bit packed vector, 16,288 elements. Tests that the same non-p2 outer
   dimension works when the mutable `reg` is declared inside a bench/function
   rather than at module scope. Useful complement to the module-scope ladder.

3. **Variant C — `[253][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   Stays at 0.508 MiBit and tests that control-flow guarded indexed writes on a
   packed `reg` are correctly elaborated and simulated (e.g. write only when a
   signed index exceeds a threshold). Useful follow-up to W590/W591.

## Sign-off

Wave Loop 717 is complete. Issue #1688 closed. Branch `wave-loop-718` exists for
continued work. All invariant laws satisfied:
- L1 TRACEABILITY: commits include `Closes #1688`.
- L2 GENERATION: new files under `gen/` were not hand-edited.
- L3 PURITY: ASCII-only identifiers.
- L4 TESTABILITY: witness contains `test` and `bench` blocks.
- L5 IDENTITY: φ² = φ + 1; φ² + φ⁻² = 3.
- L6 CEILING: numeric SSOT unchanged.
- L7 UNITY: no new `*.sh` on the critical path.
