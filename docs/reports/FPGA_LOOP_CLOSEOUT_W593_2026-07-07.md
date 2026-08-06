# FPGA Loop Closeout — Wave Loop 593

**Date:** 2026-07-07
**Issue:** #1564
**Branch:** `wave-loop-593`
**Previous:** Wave Loop 592 (#1563, `wave-loop-592`)

## Chosen cooperation variant

**Variant B — `[5][2]^15 Pt` module-scope mutable array-of-struct initialized from a
function call, with indexed signed field writes.**

Witness: `specs/scratch/w593_bench_module_5x2p15_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [5][2]^15 Pt` returning a 5,242,880-bit packed
  literal (163,840 elements, leaf values `x=(2*e)%32768`, `y=(2*e+1)%32768`).
- `pub const expected : [5][2]^15 Pt = make_grid(0);`
- `pub var dst : [5][2]^15 Pt = make_grid(0);`
- `test module_var_5x2p15_call_write`: initial state equals `expected` plus
  corner indexed reads (first element, last element, and two inner elements).
- `bench module_bench_5x2p15_call_write`: whole-array equality before writes,
  indexed reads, signed indexed field writes, read-back, frame-condition checks,
  and whole-array inequality after partial writes.

This variant reaches 5,242,880 bits (≈5.0 MiBit), slightly past the 4-MiBit cliff
validated by W591/W592, and exercises the first module-scope packed AoS with an
outer dimension of 5.

## What changed

- **No compiler or reference-model changes.** The W589 module-scope wholesale
  initializer and the generic indexed field-write paths already handle a
  non-power-of-two outer dimension of 5. The compiler paths
  (`parse_array_type`, `packed_width`, `emit_packed_array_literal_concat`,
  `try_emit_struct_array_access`) and the cocotb reference model multiply/stride
  by the actual dimension sizes, not by power-of-two assumptions.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `specs/scratch/w593_bench_module_5x2p15_aos_var_call_write.t27` (new witness,
  ~38.6 MB / ~492k lines)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w593_bench_module_5x2p15_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w593_bench_module_5x2p15_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with W593 details + W594 variants)
- `.trinity/experience.md` (W593 learnings appended)
- `.claude/plans/wave-loop-593.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W593_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the relevant risks before implementation:

1. **First module-scope outer dimension 5.** The compiler and reference model
   had to multiply/stride by 5 at the outer dimension. Code inspection and prior
   function-local non-p2 witnesses (`w569`, `w571`) indicated this would work,
   but it needed a module-scope witness to prove end-to-end.
2. **Signed i16 overflow in witness values.** With 163,840 elements, the schedule
   `(2*e + offset) % 32768` keeps every stored leaf value in `[-32768, 32767]`.
3. **Parser tolerance for huge single-line literals.** Multi-line W584-style
   brace style is mandatory; single-line literals parse silently but truncate
   the AST.
4. **Simulator capacity near 5 MiBit.** Icarus and cocotb both handle this
   width without crashing, but whole-array `$display` in a failing `assert_eq`
   could still overflow VPI buffers; the existing local-`expected` workaround
   remains available.

Scientific / technical references consulted:

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors from the earliest LRM discussions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are first-class synthesizable objects.
- Icarus Verilog quirks page and commit `128c621` — width calculation for packed
  array bounds; historical overflow bugs in open-source simulators.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors.
- Yosys documentation and PR #4100 — multidimensional packed arrays supported,
  arrays of packed structs not yet native; t27's flattening avoids this gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays requiring manual bit slicing
  in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads into cascaded case/mux
  structures.
- EquivFusion (arXiv 2026) — MLIR-based equivalence checking for array-to-vector
  lowering.
- CIRCT `HWLegalizeModules.cpp` — production scalarization of packed arrays.

## Verification matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test -p t27c --test icarus_lowerable` | 53/0 (new W593 test) |
| `./scripts/tri test --fast` | 697 passed / 0 seal mismatches (152 yosys smoke PASS / 24 pre-existing failures) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | in progress (currently simulating W590) |
| yosys smoke | 152 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged |
| Direct `t27c icarus-simulate` W593 | PASS (silent, exit 0) |
| Direct `t27c icarus-cocotb` W593 | PASS (reference-model OK) |

## Key learning

The module-scope packed AoS path is genuinely dimension-agnostic: an outer
dimension of 5 lowers, simulates, and cross-checks against the cocotb reference
model without any compiler change. Pushing to 5.0 MiBit (~20% above the 4-MiBit
cliff) remains feasible for Icarus and cocotb with a single giant literal,
confirming that the practical limit is the simulator's compile/run time rather
than a hard width cutoff at 4 MiBit. For future waves, this suggests that
non-power-of-two dimensions and modest cliff crossings can be combined without
new compiler support.

## Next Wave Loop 594 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

2. **Variant B — `[7][2]^14 Pt` module-scope var from a call with indexed signed
   writes.**
   3,670,016-bit packed vector, 114,688 elements, non-power-of-two outer
   dimension 7 under the 4-MiBit cliff. Tests a larger odd outer dimension while
   staying comfortably under the cliff. **Recommended.**

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement.**
   Stays at the 4-MiBit cliff and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W591.
