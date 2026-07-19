# Wave Loop 600 — Current Issue

**Issue #1571** — Module-scope `[19][2]^8 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with indexed
signed field writes.
**Branch:** `wave-loop-600`.
**Previous:** Wave Loop 599 (#1570, branch `wave-loop-599`).

## Chosen cooperation variant

**Variant A — `[19][2]^8 Pt` initialized from a call, with indexed signed field
writes and read-back.**

Witness: `specs/scratch/w600_bench_module_19x2p8_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [19][2]^8 Pt` returning a 155,648-bit packed
  literal with 4,864 elements, leaf values `x=(2*e + offset)%32768`,
  `y=(2*e + offset + 1)%32768`.
- `pub const expected : [19][2]^8 Pt = make_grid(0);`
- `pub var dst : [19][2]^8 Pt = make_grid(0);`
- `test module_var_19x2p8_call_write`: initial state equals `expected`, plus
  corner indexed reads (first element, last element, mid element, and an explicit
  modulo-wrap check using `make_grid(32768)`).
- `bench module_bench_19x2p8_call_write`: whole-array equality before writes,
  indexed reads, signed indexed writes, read-back, frame-condition checks,
  whole-array inequality after partial writes.

This variant tests a module-scope packed AoS with a non-power-of-two outer
dimension (19), reaching 155,648 bits (≈0.15 MiBit), well under the 4-MiBit
cliff, without requiring new compiler support.

## Background from Wave Loop 599

W599 validated a module-scope `[17][2]^9 Pt` (278,528-bit) mutable packed reg
initialized from a function call, with indexed signed field writes, with zero
compiler changes. Because 8,704 elements are below the natural modulo-wrap
point, the test retained an explicit `make_grid(32768)` call to preserve the
modulo-wrap regression signal.

## Open risks for W600

1. **First outer dimension 19.** The compiler and reference model must
   multiply/stride by 19 at the outer dimension. Prior non-p2 witnesses
   (3, 5, 7, 9, 11, 13, 15, 17) suggest this is safe, but a module-scope witness is
   needed for end-to-end proof.
2. **Element count below the modulo-wrap point.** With only 4,864 elements,
   the offset-0 schedule `(2*e + offset) % 32768` never wraps (max raw 9,727).
   The test must explicitly exercise modulo wrap with a shifted call such as
   `make_grid(32768)` to keep the regression signal equivalent to earlier waves.
3. **Parser tolerance for single-line mega-literals.** Multi-line W584-style
   brace style remains mandatory for the 8-D nested literal inside the 19×
   outer shape.
4. **Simulator capacity.** At 0.15 MiBit the witness is expected to be fast and
   comfortably interactive.

## Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus Verilog Quirks / Extensions pages — width handling and packed-array
  subset behavior.
- Icarus issue #1134 — assertion failures with unpacked arrays of packed
  structs; t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors;
  W600 stays far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Next Wave Loop 601 cooperation variants

1. **Variant A — `[21][2]^7 Pt` module-scope var from a call with indexed signed
   writes.**
   134,400-bit packed vector, 4,200 elements, non-power-of-two outer dimension 21
   well under the 4-MiBit cliff. Continues the odd outer-dimension ladder.
   **Recommended.**

2. **Variant B — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

3. **Variant C — `[19][2]^8 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.**
   Stays at 0.15 MiBit and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W590/W591.
