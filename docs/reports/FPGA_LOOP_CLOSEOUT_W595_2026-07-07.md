# FPGA Loop Closeout — Wave Loop 595

**Date:** 2026-07-07
**Issue:** #1566
**Branch:** `wave-loop-595`
**Previous:** Wave Loop 594 (#1565, `wave-loop-594`)

## Chosen cooperation variant

**Variant B — `[9][2]^13 Pt` module-scope mutable array-of-struct initialized from a
function call, with indexed signed field writes.**

Witness: `specs/scratch/w595_bench_module_9x2p13_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [9][2]^13 Pt` returning a 2,359,296-bit packed
  literal (73,728 elements, leaf values `x=(2*e)%32768`, `y=(2*e+1)%32768`).
- `pub const expected : [9][2]^13 Pt = make_grid(0);`
- `pub var dst : [9][2]^13 Pt = make_grid(0);`
- `test module_var_9x2p13_call_write`: initial state equals `expected` plus
  corner indexed reads (first element, last element, mid element, and one element
  where the modulo schedule wraps).
- `bench module_bench_9x2p13_call_write`: whole-array equality before writes,
  indexed reads, signed indexed field writes, read-back, frame-condition checks,
  and whole-array inequality after partial writes.

This variant reaches 2,359,296 bits (≈2.25 MiBit), comfortably under the 4-MiBit
cliff, and exercises the first module-scope packed AoS with an outer dimension of 9.

## What changed

- **No compiler or reference-model changes.** The W589 module-scope wholesale
  initializer and the generic indexed field-write paths already handle a
  non-power-of-two outer dimension of 9. The compiler paths
  (`parse_array_type`, `packed_width`, `emit_packed_array_literal_concat`,
  `try_emit_struct_array_access`) and the cocotb reference model multiply/stride
  by the actual dimension sizes, not by power-of-two assumptions.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `specs/scratch/w595_bench_module_9x2p13_aos_var_call_write.t27` (new witness,
  ~15.6 MB)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w595_bench_module_9x2p13_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w595_bench_module_9x2p13_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with W595 details + W596 variants)
- `.trinity/experience.md` (W595 learnings appended)
- `.claude/plans/wave-loop-595.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W595_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the relevant risks before implementation:

1. **First module-scope outer dimension 9.** The compiler and reference model
   had to multiply/stride by 9 at the outer dimension. Code inspection and prior
   function-local non-p2 witnesses (`w569`, `w571`) and module-scope witnesses
   (`w592`, `w593`, `w594`) indicated this would work, but a module-scope witness
   with outer dimension 9 was needed for end-to-end proof.
2. **Signed i16 overflow in witness values.** With 73,728 elements, the schedule
   `(2*e + offset) % 32768` keeps every stored leaf value in `[-32768, 32767]`.
3. **Parser tolerance for huge single-line literals.** Multi-line W584-style
   brace style is mandatory; single-line literals parse silently but truncate
   the AST.
4. **Simulator capacity below 4 MiBit.** Icarus and cocotb both handle this
   width without issue; the 2.25 Mbit point is a safe, interactive data point
   between W594 (3.5 MiBit) and smaller witnesses.
5. **Temporary-file disk exhaustion from prior waves.** Before the final cocotb
   gate, ~7,596 old `t27c_cocotb_*` directories in `/tmp/claude-501` consuming
   ~81 GB were removed. After cleanup the main filesystem returned to ~39 GB free
   and the W595 cocotb gate passed.

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
  W595 stays well under the 4-MiBit cliff.
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
| `cargo test -p t27c --test icarus_lowerable` | 55/0 (new W595 test) |
| `./scripts/tri test --fast` | 699 non-smoke phases passed, 0 seal mismatches, 24 pre-existing yosys smoke failures unchanged (overall exit 1 only because of those known failures) |
| yosys smoke | 154 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |
| Direct `t27c icarus-simulate` W595 | PASS (silent, exit 0) |
| Direct `t27c icarus-cocotb` W595 | PASS (reference-model OK) |

## Key learning

The module-scope packed AoS path is dimension-agnostic for odd outer dimensions
through at least 9. Because the total width stays well under the 4-MiBit cliff,
the witness remains comfortably interactive. This confirms that the practical
limit is simulator compile/run time, not a hard width cutoff, and that odd
outer dimensions can be probed in a controlled way without new compiler support.
Additionally, long-running cocotb waves accumulate large temporary directories;
periodic cleanup of `/tmp/claude-501/t27c_cocotb_*` is now part of the Wave Loop
maintenance checklist.

## Next Wave Loop 596 cooperation variants

1. **Variant A — `[11][2]^12 Pt` module-scope var from a call with indexed signed
   writes.**
   1,441,792-bit packed vector, 45,056 elements, non-power-of-two outer
   dimension 11 well under the 4-MiBit cliff. Continues the odd outer-dimension
   ladder and is expected to remain comfortably interactive. **Recommended.**

2. **Variant B — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

3. **Variant C — `[9][2]^13 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.**
   Stays at 2.25 MiBit and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W590/W591.
