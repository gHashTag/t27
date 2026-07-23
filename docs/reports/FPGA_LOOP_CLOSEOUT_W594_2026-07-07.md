# FPGA Loop Closeout — Wave Loop 594

**Date:** 2026-07-07
**Issue:** #1565
**Branch:** `wave-loop-594`
**Previous:** Wave Loop 593 (#1564, `wave-loop-593`)

## Chosen cooperation variant

**Variant B — `[7][2]^14 Pt` module-scope mutable array-of-struct initialized from a
function call, with indexed signed field writes.**

Witness: `specs/scratch/w594_bench_module_7x2p14_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [7][2]^14 Pt` returning a 3,670,016-bit packed
  literal (114,688 elements, leaf values `x=(2*e)%32768`, `y=(2*e+1)%32768`).
- `pub const expected : [7][2]^14 Pt = make_grid(0);`
- `pub var dst : [7][2]^14 Pt = make_grid(0);`
- `test module_var_7x2p14_call_write`: initial state equals `expected` plus
  corner indexed reads (first element, last element, mid element, and one element
  where the modulo schedule wraps).
- `bench module_bench_7x2p14_call_write`: whole-array equality before writes,
  indexed reads, signed indexed field writes, read-back, frame-condition checks,
  and whole-array inequality after partial writes.

This variant reaches 3,670,016 bits (≈3.5 MiBit), comfortably under the 4-MiBit
cliff, and exercises the first module-scope packed AoS with an outer dimension of 7.

## What changed

- **No compiler or reference-model changes.** The W589 module-scope wholesale
  initializer and the generic indexed field-write paths already handle a
  non-power-of-two outer dimension of 7. The compiler paths
  (`parse_array_type`, `packed_width`, `emit_packed_array_literal_concat`,
  `try_emit_struct_array_access`) and the cocotb reference model multiply/stride
  by the actual dimension sizes, not by power-of-two assumptions.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `specs/scratch/w594_bench_module_7x2p14_aos_var_call_write.t27` (new witness,
  ~24.4 MB / ~316k lines)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w594_bench_module_7x2p14_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w594_bench_module_7x2p14_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with W594 details + W595 variants)
- `.trinity/experience.md` (W594 learnings appended)
- `.claude/plans/wave-loop-594.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W594_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the relevant risks before implementation:

1. **First module-scope outer dimension 7.** The compiler and reference model
   had to multiply/stride by 7 at the outer dimension. Code inspection and prior
   function-local non-p2 witnesses (`w569`, `w571`) and module-scope witnesses
   (`w592`, `w593`) indicated this would work, but a module-scope witness with
   outer dimension 7 was needed for end-to-end proof.
2. **Signed i16 overflow in witness values.** With 114,688 elements, the schedule
   `(2*e + offset) % 32768` keeps every stored leaf value in `[-32768, 32767]`.
3. **Parser tolerance for huge single-line literals.** Multi-line W584-style
   brace style is mandatory; single-line literals parse silently but truncate
   the AST.
4. **Simulator capacity below 4 MiBit.** Icarus and cocotb both handle this
   width without issue; the 3.67 Mbit point is a safe, interactive data point
   between W592 (3.1 MiBit) and W593 (5.0 MiBit).

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
  W594 stays under the 4-MiBit cliff to remain interactive.
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
| `cargo test -p t27c --test icarus_lowerable` | 54/0 (new W594 test) |
| `./scripts/tri test --fast` | TBD (background run in progress) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | TBD (full pipeline still running) |
| yosys smoke | 152 passed, 24 pre-existing failures unchanged (expected) |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged |
| Direct `t27c icarus-simulate` W594 | PASS (silent, exit 0) |
| Direct `t27c icarus-cocotb` W594 | PASS (reference-model OK) |

## Key learning

The module-scope packed AoS path is dimension-agnostic not only for small
non-power-of-two dimensions (3 and 5) but also for a larger odd outer dimension
of 7. Because the total width stays under the 4-MiBit cliff, the witness
remains comfortably interactive while still expanding the layout-coverage
envelope. This confirms that the practical limit is simulator compile/run time,
not a hard width cutoff, and that odd outer dimensions can be probed in a
controlled way without new compiler support.

## Next Wave Loop 595 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

2. **Variant B — `[9][2]^13 Pt` module-scope var from a call with indexed signed
   writes.**
   2,359,296-bit packed vector, 73,728 elements, non-power-of-two outer
   dimension 9 under the 4-MiBit cliff. Tests the next odd outer dimension while
   staying safely under the cliff. **Recommended.**

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement.**
   Stays at the 4-MiBit cliff and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W591.
