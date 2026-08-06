# FPGA Loop Closeout — Wave Loop 592

**Date:** 2026-07-07  
**Issue:** #1563  
**Branch:** `wave-loop-592`  
**Previous:** Wave Loop 591 (#1562, `wave-loop-591`)

## Chosen cooperation variant

**Variant B — `[3][2]^15 Pt` module-scope mutable array-of-struct initialized from a
function call, with indexed signed field writes.**

Witness: `specs/scratch/w592_bench_module_3x2p15_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [3][2]^15 Pt` returning a 3,145,728-bit packed
  literal (98,304 elements, leaf values `x=(2*e)%32768`, `y=(2*e+1)%32768`).
- `pub const expected : [3][2]^15 Pt = make_grid(0);`
- `pub var dst : [3][2]^15 Pt = make_grid(0);`
- `test module_var_3x2p15_call_write`: initial state equals `expected` plus
  corner indexed reads (first element, last element, and two inner elements).
- `bench module_bench_3x2p15_call_write`: whole-array equality before writes,
  indexed reads, signed indexed field writes, read-back, frame-condition checks,
  and whole-array inequality after partial writes.

This variant was chosen because it stays **under the validated 4-MiBit cliff**
(3.1 MiBit vs. 4.2 MiBit), avoids the duplicate-giant-literal cost of staying at
the cliff, and exercises the first module-scope packed AoS with a non-power-of-two
outer dimension.

## What changed

- **No compiler or reference-model changes.** The W589 module-scope wholesale
  initializer and the generic indexed field-write paths already handle a
  non-power-of-two outer dimension of 3. The compiler paths
  (`parse_array_type`, `packed_width`, `emit_packed_array_literal_concat`,
  `try_emit_struct_array_access`) and the cocotb reference model multiply/stride
  by the actual dimension sizes, not by power-of-two assumptions.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `specs/scratch/w592_bench_module_3x2p15_aos_var_call_write.t27` (new witness)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w592_bench_module_3x2p15_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w592_bench_module_3x2p15_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with W592 details + W593 variants)
- `.trinity/experience.md` (W592 learnings appended)
- `.claude/plans/wave-loop-592.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W592_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the relevant risks before implementation:

1. **First module-scope non-p2 packed AoS.** The compiler and reference model
   had to multiply/stride by 3 at the outer dimension. Code inspection and prior
   function-local non-p2 witnesses (`w569`, `w571`) indicated this would work,
   but it needed a module-scope witness to prove end-to-end.
2. **Signed i16 overflow in witness values.** With 98,304 elements, the schedule
   `(2*e + offset) % 32768` keeps every stored leaf value in `[-32768, 32767]`.
3. **Parser tolerance for huge single-line literals.** Multi-line W584-style
   brace style is mandatory; single-line literals parse silently but truncate
   the AST.
4. **Simulator capacity for a 3.1-MiBit packed vector.** Icarus and cocotb both
   handle this width without crashing, but whole-array `$display` in a failing
   `assert_eq` could still overflow VPI buffers; the existing local-`expected`
   workaround remains available.

Scientific / technical references consulted:

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors from the earliest LRM discussions.
- Rich, DVCon 2023 — upcoming standard clarifications for packed-array syntax.
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
| `cargo test -p t27c --test icarus_lowerable` | 52/0 (new W592 test) |
| `./scripts/tri test --fast` | 696 passed / 0 seal mismatches (151 yosys smoke PASS) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 79 Icarus PASS / 79 cocotb PASS / 0 seal mismatches / 24 pre-existing yosys smoke baselines |
| yosys smoke | 151 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |
| Direct `t27c icarus-simulate` W592 | PASS (silent, exit 0) |
| Direct `t27c icarus-cocotb` W592 | PASS (reference-model OK) |

## Key learning

The module-scope packed AoS path is genuinely dimension-agnostic: a
non-power-of-two outer dimension of 3 lowers, simulates, and cross-checks against
the cocotb reference model without any compiler change. Staying under the 4-MiBit
cliff keeps the interactive wall-clock modest and avoids the file-size doubling
that occurs when two 4-MiBit literals share one module. For future waves, this
confirms that the practical limit is the 4-MiBit cliff itself, not the shape of
the dimensions below it.

## Next Wave Loop 593 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff and
   will likely hit Icarus/Yosys compile-time or memory limits interactively. Not
   recommended without chunked-literal design.

2. **Variant B — `[5][2]^14 Pt` module-scope var from a call with indexed signed
   writes.**
   5,242,880-bit packed vector, 81,920 elements, non-power-of-two outer
   dimension 5 under the 4-MiBit cliff. Tests a larger odd outer dimension while
   staying under the cliff. **Recommended.**

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement.**
   Stays at the 4-MiBit cliff and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W591.
