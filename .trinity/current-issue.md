# Wave Loop 592 — Current Issue

**Issue #1563** — Module-scope `[3][2]^15 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with indexed
signed field writes.
**Branch:** `wave-loop-592`.
**Previous:** Wave Loop 591 (#1562, branch `wave-loop-591`).

## Chosen cooperation variant

**Variant B — `[3][2]^15 Pt` initialized from a call, with indexed signed field
writes and read-back.**

Witness: `specs/scratch/w592_bench_module_3x2p15_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [3][2]^15 Pt` returning a 3,145,728-bit packed
  literal with leaf values inside signed i16.
- `pub const expected : [3][2]^15 Pt = make_grid(0);`
- `pub var dst : [3][2]^15 Pt = make_grid(0);`
- `test module_var_3x2p15_call_write`: initial state equals `expected`, plus
  corner indexed reads.
- `bench module_bench_3x2p15_call_write`: whole-array equality before writes,
  indexed reads, signed indexed writes, read-back, frame-condition checks,
  whole-array inequality after partial writes.

This variant tests the first module-scope packed AoS with a non-power-of-two
outer dimension, without requiring new compiler support.

## Weak points identified by Agent E

1. **First module-scope non-p2 packed AoS.** The compiler and reference model
   must multiply/stride by 3 at the outer dimension. Prior function-local non-p2
   witnesses indicated this would work, but a module-scope witness was needed
   for end-to-end proof.
2. **Signed i16 overflow.** The schedule `(2*e + offset) % 32768` keeps all 98,304
   elements' leaf values in `[-32768, 32767]`.
3. **Parser tolerance for single-line mega-literals.** Multi-line brace style is
   mandatory for the 16-D literal.
4. **Simulator capacity for 3.1-MiBit packed vector.** Icarus and cocotb handle
   this width, but whole-array `$display` on failure could overflow VPI buffers;
   the local-`expected` workaround remains available.

## Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Rich, DVCon 2023 — upcoming standard clarifications.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus quirks / commit `128c621` / issue #1171 — width bugs and freezes with
  very large packed vectors.
- Yosys docs / PR #4100 — multidimensional packed arrays supported, arrays of
  packed structs not native; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- EquivFusion (arXiv 2026) — MLIR equivalence checking for array lowering.
- CIRCT `HWLegalizeModules.cpp` — production packed-array scalarization.

## Next Wave Loop 593 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector. Crosses the 4-MiBit cliff; not recommended
   interactively without chunked-literal design.

2. **Variant B — `[5][2]^14 Pt` module-scope var from a call with indexed signed
   writes.**
   5,242,880-bit packed vector, 81,920 elements, non-power-of-two outer
   dimension 5 under the 4-MiBit cliff. Tests a larger odd outer dimension.
   **Recommended.**

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement.**
   Stays at the 4-MiBit cliff and tests control-flow guarded whole-array
   reassignment of a packed `reg`.
