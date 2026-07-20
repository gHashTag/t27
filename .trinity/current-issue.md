# Wave Loop 655 — Current Issue

**Issue #1626** — Module-scope `[129][2]^6 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with indexed
signed field writes.
**Branch:** `wave-loop-655`.
**Previous:** Wave Loop 654 (#1625, branch `wave-loop-654`).

## Chosen cooperation variant

**Variant A — `[129][2]^6 Pt` initialized from a call, with indexed signed field
writes and read-back.**

Witness: `specs/scratch/w655_bench_module_129x2p6_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [129][2][2][2][2][2][2] Pt` returning a 264,192-bit packed
  literal with 8,256 elements, leaf values `x=(2*e + offset)%32768`,
  `y=(2*e + offset + 1)%32768`.
- `pub const expected : [129][2][2][2][2][2][2] Pt = make_grid(0);`
- `pub var dst : [129][2][2][2][2][2][2] Pt = make_grid(0);`
- `test module_var_129x2p6_call_write`: initial state equals `expected`, plus
  corner indexed reads (first element, last element, mid element, and an explicit
  modulo-wrap check using `make_grid(32768)`).
- `bench module_bench_129x2p6_call_write`: whole-array equality before writes,
  indexed reads, signed indexed writes, read-back, frame-condition checks,
  changed-element checks after partial writes.

This variant continues the module-scope packed AoS odd outer-dimension ladder
(129), reaching 264,192 bits (≈0.252 MiBit), well under the 4-MiBit cliff, without
requiring new compiler support.

## Background from Wave Loop 654

W654 validated a module-scope `[127][2]^6 Pt` (260,096-bit) mutable packed reg
initialized from a function call, with indexed signed field writes, with zero
compiler changes. Because 8,128 elements are below the natural modulo-wrap
point, the test retained an explicit `make_grid(32768)` call to preserve the
modulo-wrap regression signal. W605–W654 all use the same module-scope
lowerable style after W606 showed that alternative syntax can parse but produce
invalid Verilog.

## Open risks for W655

1. **First outer dimension 129.** The compiler and reference model must
   multiply/stride by 129 at the outer dimension. Prior non-p2 witnesses
   (3, 5, 7, ..., 125, 127) suggest this is safe, but a module-scope
   witness is needed for end-to-end proof.
2. **Element count below the modulo-wrap point.** With only 8,256 elements,
   the offset-0 schedule `(2*e + offset) % 32768` never wraps (max raw 16,511).
   The test must explicitly exercise modulo wrap with a shifted call such as
   `make_grid(32768)` to keep the regression signal equivalent to earlier waves.
3. **Parser tolerance for single-line mega-literals.** Multi-line W584-style
   brace style remains mandatory for the 6-D nested literal inside the 129×
   outer shape.
4. **Simulator capacity.** At 0.252 MiBit the witness is expected to be very fast
   and comfortably interactive.
5. **`assert_ne` is not emitted by the Icarus simulation path.** The structural
   classifier accepts it, but `gen_verilog_test_stmt` only lowers `assert_eq`.
   W655 replaces the whole-array `assert_ne(dst, expected)` with checks on the
   changed elements to keep the simulation gate passing without compiler changes.

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
  W655 stays far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Next Wave Loop 656 cooperation variants

1. **Variant A — `[131][2]^6 Pt` module-scope var from a call with indexed signed
   writes.**
   268,288-bit packed vector, 8,384 elements, non-power-of-two outer dimension 131
   well under the 4-MiBit cliff. Continues the odd outer-dimension ladder.
   **Recommended.**

2. **Variant B — `[129][2]^6 Pt` bench-local (function-local) packed array var
   from a call with indexed signed writes.**
   264,192-bit packed vector, 8,256 elements. Tests that the same non-p2 outer
   dimension works when the mutable `reg` is declared inside a bench/function
   rather than at module scope. Useful complement to the module-scope ladder.

3. **Variant C — `[129][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   Stays at 0.252 MiBit and tests that control-flow guarded indexed writes on a
   packed `reg` are correctly elaborated and simulated (e.g. write only when a
   signed index exceeds a threshold). Useful follow-up to W590/W591.
