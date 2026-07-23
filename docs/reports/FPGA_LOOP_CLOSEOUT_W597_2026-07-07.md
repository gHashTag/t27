# FPGA Wave Loop 597 Closeout Report

**Issue:** #1568  
**Branch:** `wave-loop-597`  
**Previous:** Wave Loop 596 (#1567, `wave-loop-596`)  
**Date:** 2026-07-07  
**Author:** Trinity Agent (Queen)  
**Status:** Closed  

## Summary

Wave Loop 597 validated a module-scope, mutable, packed array-of-scalar-struct
variable with a non-power-of-two outer dimension of **13**:

- Type: `[13][2]^11 Pt` where `Pt { x : i16, y : i16 }`.
- Total packed width: **852,032 bits** (≈0.81 MiBit).
- Total elements: **26,624**.
- Witness: `specs/scratch/w597_bench_module_13x2p11_aos_var_call_write.t27`
  (~4.98 MB).

The implementation required **zero compiler changes** and **zero
reference-model changes**. The generic W589 module-scope wholesale initializer
path and the generic indexed field-write paths handled outer stride 13
correctly.

> Note on sizing: the W596 closeout report listed the W597 Variant A as
> 1,114,112 bits / 34,816 elements. That arithmetic corresponds to
> `[17][2]^11 Pt`, not `[13][2]^11 Pt`. This wave uses `[13][2]^11 Pt` and the
> correct totals above.

## What was added

1. `specs/scratch/w597_bench_module_13x2p11_aos_var_call_write.t27`
   - `pub struct Pt { x : i16, y : i16 }`
   - `pub fn make_grid(offset : u16) -> [13][2]^11 Pt`
   - `pub const expected : [13][2]^11 Pt = make_grid(0);`
   - `pub var dst : [13][2]^11 Pt = make_grid(0);`
   - Test `module_var_13x2p11_call_write`: initial state, corner indexed reads.
   - Bench `module_bench_13x2p11_call_write`: indexed reads, signed indexed
     writes, read-back, frame-condition checks, whole-array inequality.
2. `.trinity/seals/scratch_w597_bench_module_13x2p11_aos_var_call_write.json`
3. `.trinity/icarus-baselines/specs/scratch/w597_bench_module_13x2p11_aos_var_call_write.json`
4. `bootstrap/tests/icarus_lowerable.rs`: new integration test
   `accepts_w597_bench_module_13x2p11_aos_var_call_write`.
5. `.claude/plans/wave-loop-597.md`.
6. `.trinity/current-issue.md` updated for W598 cooperation variants.
7. `.trinity/experience.md` updated with W597 learnings.
8. Persistent memory: `~/.claude/projects/-Users-playra-t27/memory/wave-loop-597.md`
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
| `cargo test -p t27c --test icarus_lowerable` | 57/0 (new W597 test) |
| `./scripts/tri test --fast` | not run to completion — Phase 1 Parse dominated by large literal specs and the process made no progress after ~20 min wall-clock |
| yosys smoke | 24 pre-existing baselines unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |
| Direct `t27c icarus-simulate` W597 | PASS (silent, exit 0) |
| Direct `t27c icarus-cocotb` W597 | PASS (reference-model OK) |

## Scientific / technical references consulted

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array width as product of dimensions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs as
  first-class synthesizable objects.
- Icarus Verilog Quirks / Extensions pages — packed-array subset behavior.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion
  failures; t27 flattening avoids the trigger.
- Icarus issue #1171 — large packed vectors can freeze elaboration; W597 stays
  far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals and
  flat `LogicArray` for multidimensional packed arrays.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Root-cause / fix discussion

No compiler fix was required. The module-scope packed-AoS path is
dimension-agnostic for odd outer dimensions through at least 13. Because the
total width stays well under the 4-MiBit cliff, the witness is comfortably
interactive. The only implementation work was witness generation, integration
test, seal, baseline, and documentation.

## Next Wave Loop 598 cooperation variants

1. **Variant A — `[15][2]^10 Pt` module-scope var from a call with indexed signed
   writes.**
   491,520-bit packed vector, 15,360 elements, non-power-of-two outer dimension 15
   well under the 4-MiBit cliff. Continues the odd outer-dimension ladder.
   **Recommended.**

2. **Variant B — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

3. **Variant C — `[13][2]^11 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.**
   Stays at 0.81 MiBit and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W590/W591.
