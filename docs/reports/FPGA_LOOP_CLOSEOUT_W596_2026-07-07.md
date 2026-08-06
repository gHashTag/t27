# FPGA Loop Closeout — Wave Loop 596

**Date:** 2026-07-07
**Issue:** #1567
**Branch:** `wave-loop-596`
**Previous:** Wave Loop 595 (#1566, `wave-loop-595`)

## Chosen cooperation variant

**Variant A — `[11][2]^12 Pt` module-scope mutable array-of-struct initialized from a
function call, with indexed signed field writes.**

Witness: `specs/scratch/w596_bench_module_11x2p12_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [11][2]^12 Pt` returning a 1,441,792-bit packed
  literal (45,056 elements, leaf values `x=(2*e)%32768`, `y=(2*e+1)%32768`).
- `pub const expected : [11][2]^12 Pt = make_grid(0);`
- `pub var dst : [11][2]^12 Pt = make_grid(0);`
- `test module_var_11x2p12_call_write`: initial state equals `expected` plus
  corner indexed reads (first element, last element, mid element, and one element
  where the modulo schedule wraps).
- `bench module_bench_11x2p12_call_write`: whole-array equality before writes,
  indexed reads, signed indexed field writes, read-back, frame-condition checks,
  and whole-array inequality after partial writes.

This variant reaches 1,441,792 bits (≈1.37 MiBit), well under the 4-MiBit
cliff, and exercises the first module-scope packed AoS with an outer dimension of 11.

## What changed

- **No compiler or reference-model changes.** The W589 module-scope wholesale
  initializer and the generic indexed field-write paths already handle a
  non-power-of-two outer dimension of 11. The compiler paths
  (`parse_array_type`, `packed_width`, `emit_packed_array_literal_concat`,
  `try_emit_struct_array_access`) and the cocotb reference model multiply/stride
  by the actual dimension sizes, not by power-of-two assumptions.
- Added a new scratch witness (~8.98 MB), integration test, seal, and Icarus baseline.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `specs/scratch/w596_bench_module_11x2p12_aos_var_call_write.t27` (new witness,
  ~8.98 MB)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w596_bench_module_11x2p12_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w596_bench_module_11x2p12_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with W596 details + W597 variants)
- `.trinity/experience.md` (W596 learnings appended)
- `.claude/plans/wave-loop-596.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W596_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the relevant risks before implementation:

1. **First module-scope outer dimension 11.** The compiler and reference model
   had to multiply/stride by 11 at the outer dimension. Code inspection and prior
   function-local non-p2 witnesses (`w569`, `w571`) and module-scope witnesses
   (`w592`, `w593`, `w594`, `w595`) indicated this would work, but a module-scope
   witness with outer dimension 11 was needed for end-to-end proof.
2. **Signed i16 overflow in witness values.** With 45,056 elements, the schedule
   `(2*e + offset) % 32768` keeps every stored leaf value in `[-32768, 32767]`.
   The maximum raw value is `2 * 45055 + 1 = 90111`, and `90111 % 32768 = 24574`.
3. **Parser tolerance for huge single-line literals.** Multi-line W584-style
   brace style is mandatory; single-line literals parse silently but truncate
   the AST.
4. **Simulator capacity below 4 MiBit.** Icarus and cocotb both handle this
   width without issue; the 1.37 Mbit point is a safe, interactive data point.
5. **Temporary-file disk hygiene.** The W595 cleanup left `/tmp/claude-501` with
   minimal stale directories, so W596 cocotb gate ran without disk exhaustion.

Scientific / technical references consulted:

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors from the earliest LRM discussions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are first-class synthesizable objects.
- Icarus Verilog Quirks / Extensions pages — width handling and packed-array
  subset behavior.
- Icarus issue #1134 — assertion failures with unpacked arrays of packed structs;
  t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors;
  W596 stays well under the 4-MiBit cliff.
- Yosys documentation and PR #4100 / issue #4653 / issue #2677 —
  multidimensional packed arrays supported, arrays of packed structs still
  unsupported; t27's flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays requiring manual bit slicing
  in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads into cascaded case/mux
  structures.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production scalarization of packed
  arrays.

## Verification matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test -p t27c --test icarus_lowerable` | 56/0 (new W596 test) |
| `./scripts/tri test --fast` | 700/0/0/0/0/0/0 non-smoke phases, 0 seal mismatches, 24 pre-existing yosys smoke failures unchanged (overall exit 1 only because of those known failures) |
| yosys smoke | 155 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |
| Direct `t27c icarus-simulate` W596 | PASS (silent, exit 0) |
| Direct `t27c icarus-cocotb` W596 | PASS (reference-model OK) |

## Key learning

The module-scope packed AoS path is dimension-agnostic for odd outer dimensions
through at least 11. Because the total width stays well under the 4-MiBit cliff,
the witness remains comfortably interactive and smaller than the previous wave
(W596 ~8.98 MB vs W595 ~15.6 MB). This confirms that the practical limit is
simulator compile/run time, not a hard width cutoff, and that the non-p2
outer-dimension ladder can be extended incrementally without new compiler support.

## Next Wave Loop 597 cooperation variants

1. **Variant A — `[13][2]^11 Pt` module-scope var from a call with indexed signed
   writes.**
   1,114,112-bit packed vector, 34,816 elements, non-power-of-two outer
   dimension 13 well under the 4-MiBit cliff. Continues the odd outer-dimension
   ladder and is expected to remain comfortably interactive. **Recommended.**

2. **Variant B — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

3. **Variant C — `[11][2]^12 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.**
   Stays at 1.37 MiBit and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W590/W591.
