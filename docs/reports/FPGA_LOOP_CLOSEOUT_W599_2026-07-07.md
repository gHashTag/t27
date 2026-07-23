# FPGA Wave Loop 599 Closeout Report

**Issue:** #1570  
**Branch:** `wave-loop-599`  
**Previous:** Wave Loop 598 (#1569, `wave-loop-598`)  
**Date:** 2026-07-07  
**Author:** Trinity Agent (Queen)  
**Status:** Closed  

## Summary

Wave Loop 599 validated a module-scope, mutable, packed array-of-scalar-struct
variable with a non-power-of-two outer dimension of **17**:

- Type: `[17][2]^9 Pt` where `Pt { x : i16, y : i16 }`.
- Total packed width: **278,528 bits** (≈0.27 MiBit).
- Total elements: **8,704**.
- Witness: `specs/scratch/w599_bench_module_17x2p9_aos_var_call_write.t27`
  (~1.42 MB).

The implementation required **zero compiler changes** and **zero
reference-model changes**. The generic W589 module-scope wholesale initializer
path and the generic indexed field-write paths handled outer stride 17
correctly.

## Weak-point analysis and mitigations

| Weak point | Why it matters | Mitigation |
|------------|----------------|------------|
| Outer dimension 17 untested at module scope | Stride-17 multiplication and row-major flattening must be correct end-to-end. | Added direct Icarus simulation and cocotb reference-model cross-check. |
| Element count below the modulo-wrap point | With 8,704 elements the offset-0 schedule `(2*e + offset) % 32768` never wraps (`max raw = 17,407`). Earlier waves used `e_wrap = 16384` to assert wrap behavior. | Added an explicit shifted call `make_grid(32768)` and asserted its first/last elements equal `(offset + raw) % 32768`, proving the compiler and reference model both evaluate modulo correctly even when the offset-0 path does not wrap. |
| Single-line mega-literal parser truncation | Extreme-rank literals parsed silently but dropped trailing declarations in earlier waves. | Mandatory multi-line W584 brace style for the 9-D inner literal. |
| Full batch sweep blocked by unrelated giant specs | `./scripts/tri test --fast` Phase 1 Parse can stall on unrelated 4-MiBit specs. | Rely on direct Icarus/cocotb gates and document batch status. |

## What was added

1. `specs/scratch/w599_bench_module_17x2p9_aos_var_call_write.t27`
   - `pub struct Pt { x : i16, y : i16 }`
   - `pub fn make_grid(offset : u16) -> [17][2]^9 Pt`
   - `pub const expected : [17][2]^9 Pt = make_grid(0);`
   - `pub var dst : [17][2]^9 Pt = make_grid(0);`
   - Test `module_var_17x2p9_call_write`: initial state, corner indexed
     reads, and explicit modulo-wrap check via `make_grid(32768)`.
   - Bench `module_bench_17x2p9_call_write`: indexed reads, signed indexed
     writes, read-back, frame-condition checks, whole-array inequality.
2. `.trinity/seals/scratch_w599_bench_module_17x2p9_aos_var_call_write.json`
3. `.trinity/icarus-baselines/specs/scratch/w599_bench_module_17x2p9_aos_var_call_write.json`
4. `bootstrap/tests/icarus_lowerable.rs`: new integration test
   `accepts_w599_bench_module_17x2p9_aos_var_call_write`.
5. `.claude/plans/wave-loop-599.md`.
6. `.trinity/current-issue.md` updated for W600 cooperation variants.
7. `.trinity/experience.md` updated with W599 learnings.
8. Persistent memory: `~/.claude/projects/-Users-playra-t27/memory/wave-loop-599.md`
   plus `MEMORY.md` index update.

## What was changed

- `bootstrap/src/compiler.rs`: **no changes**.
- `scripts/cocotb_ref_model.py`: **no changes**.
- FROZEN_HASH: **unchanged** `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

## Verification matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test -p t27c --test icarus_lowerable` | 59/0 (new W599 test) |
| `./scripts/tri test --fast` | not run — Phase 1 Parse dominated by unrelated large literal specs in prior waves |
| yosys smoke | 24 pre-existing baselines unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |
| Direct `t27c icarus-simulate` W599 | PASS (silent, exit 0) |
| Direct `t27c icarus-cocotb` W599 | PASS (reference-model OK) |

## Scientific / technical references consulted

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  dimensions; no power-of-two restriction.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs as
  synthesizable first-class objects.
- Icarus Verilog Quirks / Extensions pages — packed-array subset behavior.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion
  failures; t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors;
  W599 stays far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Root-cause / fix discussion

No compiler fix was required. The module-scope packed-AoS path is
dimension-agnostic for odd outer dimensions through at least 17. Because the
total width stays well under the 4-MiBit cliff, the witness is comfortably
interactive. The only implementation work was witness generation, integration
test, seal, baseline, and documentation.

The explicit modulo-wrap check (`make_grid(32768)`) again preserves the
regression signal that earlier large waves achieved naturally: it proves both
the compiler's constant-folder and the Python reference model correctly apply
`% 32768`, even though the offset-0 path for 8,704 elements does not trigger a
wrap.

## Next Wave Loop 600 cooperation variants

1. **Variant A — `[19][2]^8 Pt` module-scope var from a call with indexed signed
   writes.**
   194,560-bit packed vector, 6,080 elements, non-power-of-two outer dimension 19
   well under the 4-MiBit cliff. Continues the odd outer-dimension ladder.
   **Recommended.**

2. **Variant B — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

3. **Variant C — `[17][2]^9 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.**
   Stays at 0.27 MiBit and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W590/W591.
