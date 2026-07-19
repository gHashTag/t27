# Wave Loop 593 — Current Issue

**Issue #1564** — Module-scope `[5][2]^15 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with indexed
signed field writes.
**Branch:** `wave-loop-593`.
**Previous:** Wave Loop 592 (#1563, branch `wave-loop-592`).

## Chosen cooperation variant

**Variant B — `[5][2]^15 Pt` initialized from a call, with indexed signed field
writes and read-back.**

Witness: `specs/scratch/w593_bench_module_5x2p15_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [5][2]^14 Pt` returning a 5,242,880-bit packed
  literal with leaf values inside signed i16.
- `pub const expected : [5][2]^14 Pt = make_grid(0);`
- `pub var dst : [5][2]^14 Pt = make_grid(0);`
- `test module_var_5x2p14_call_write`: initial state equals `expected`, plus
  corner indexed reads.
- `bench module_bench_5x2p14_call_write`: whole-array equality before writes,
  indexed reads, signed indexed writes, read-back, frame-condition checks,
  whole-array inequality after partial writes.

This variant tests a larger module-scope packed AoS with a non-power-of-two
outer dimension (5), reaching 5,242,880 bits (≈5.0 MiBit), slightly past the
4-MiBit cliff validated by W591/W592, without requiring new compiler support.

## Background from Wave Loop 592

W592 validated a module-scope `[3][2]^15 Pt` (3,145,728-bit) mutable packed reg
initialized from a function call, with indexed signed field writes, with zero
compiler changes. The generic `gen_verilog_var`/`gen_verilog_const` wholesale
paths and the indexed field-write paths handled outer stride 3 correctly.

## Open risks for W593

1. **First outer dimension 5.** The compiler and reference model must
   multiply/stride by 5 at the outer dimension. Prior non-p2 witnesses (3, and
   function-local 3/5) suggest this is safe, but a module-scope witness is
   needed for end-to-end proof.
2. **Signed i16 overflow.** With 81,920 elements, the schedule
   `(2*e + offset) % 32768` keeps every leaf value in `[-32768, 32767]`.
3. **Parser tolerance for single-line mega-literals.** Multi-line W584-style
   brace style remains mandatory for the 15-D literal.
4. **Simulator capacity near 5 MiBit.** Icarus and cocotb handled 3.1 MiBit and
   4.2 MiBit smoothly; 5.2 MiBit is the next step toward the cliff.

## Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus Verilog quirks / commit `128c621` / issue #1171 — width bugs and freezes
  with very large packed vectors.
- Yosys docs / PR #4100 — multidimensional packed arrays supported, arrays of
  packed structs not native; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- EquivFusion (arXiv 2026) — MLIR equivalence checking for array lowering.
- CIRCT `HWLegalizeModules.cpp` — production packed-array scalarization.

## Next Wave Loop 594 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector. Crosses the 4-MiBit cliff; not recommended
   interactively without chunked-literal design.

2. **Variant B — `[7][2]^14 Pt` module-scope var from a call with indexed signed
   writes.**
   3,670,016-bit packed vector, 114,688 elements, non-power-of-two outer
   dimension 7 under the 4-MiBit cliff. Tests a larger odd outer dimension.
   **Recommended.**

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement.**
   Stays at the 4-MiBit cliff and tests control-flow guarded whole-array
   reassignment of a packed `reg`.
