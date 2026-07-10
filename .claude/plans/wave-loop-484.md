# Wave Loop 484 Plan — Dynamic array/string `.len()` / `.contains()` lowering

**Date:** 2026-07-07  
**Branch:** `wave-loop-484`  
**Variant:** B (functional lowering of the next most common `UNSUPPORTED_ICARUS` placeholder class)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Goal

Make `.len()` and `.contains(needle)` functional in gen-verilog for:
1. Fixed-size scalar arrays (function-local, bench-local, module-level, and parameters).
2. Fixed-size arrays of structs (return the element count).
3. String literals and string-typed variables/fields whose value is statically known.

Keep the Icarus smoke gate at **0 documented baseline failures** and keep all
non-smoke tests, yosys smoke, seals, and Rust unit tests green.

## Weak spots addressed

- `try_gen_verilog_static_len` currently emits a placeholder for string `.len()`
  because the receiver name is flattened and the literal value is lost.
- `try_gen_verilog_static_contains` currently skips `u8` arrays (treated as byte
  buffers) and has no path for string literals or array parameters.
- IGLA specs use `.len()` on string fields and array variables; these currently
  become `32'd0 /* UNSUPPORTED_ICARUS: dynamic string/array method (...) */` in
  generated Verilog, which is legal but semantically incorrect.

## Literature context

- **Synthesizable subset of Verilog / Icarus strictness.** Icarus Verilog 12.0
  accepts constant expressions and OR-reductions over bounded ranges, but does
  not support runtime string operations or unbounded memories. Static lowering
  of `.len()` to a compile-time constant and `.contains()` to a finite OR tree
  keeps the result inside the synthesizable subset.
- **DSL lowering of collection operations.** FIRRTL and Sparkle/Lean 4 HDL
  lower bounded collection queries (length, membership) into bit-vector
  operations. W484 applies the same idea directly to t27's fixed-size arrays and
  string literals.
- **Static vs dynamic string handling.** Runtime strings are unsynthesizable in
  Verilog; t27 string fields used in invariants/tests are typically supplied as
  literals, so the length can be computed at emission time from the literal node.

## Subtasks

1. **String-literal length lowering.**
   - Track string-literal locals/fields so `.len()` can emit the byte count.
   - For `ExprCall` with method `.len()` on a receiver that resolves to a string
     literal, emit the literal length in bytes.

2. **Fixed-size array `.len()` for all known dimensions.**
   - Extend `static_array_len` to cover array parameters (`array_param_types`).
   - Already covers function-local, bench-local, and module-level scalar arrays.
   - Add array-of-struct support by returning the outer element count.

3. **Fixed-size array `.contains(needle)` lowering.**
   - Already supports non-u8 scalar arrays. Extend to `u8` arrays (byte buffers)
    as an OR-reduction over element equality.
   - Extend to array-of-struct by comparing packed element vectors, or emit a
     placeholder if element comparison is not yet supported.

4. **String `.contains(substring)` lowering.**
   - For string literals, emit a Verilog constant boolean based on literal
     substring search.
   - For string variables/fields whose initializer is a known literal, use the
     tracked literal value.

5. **Witness specs.**
   - `specs/scratch/w484_dynamic_len.t27` — `.len()` on fixed-size arrays and
     string literals.
   - `specs/scratch/w484_contains.t27` — `.contains()` on fixed-size arrays and
     string literals.
   - Include adversarial tests under Icarus simulation.

6. **Validation.**
   - `./scripts/tri test --fast`: 656/656 non-smoke, 136/136 yosys, 136/136
     Icarus, 0 seal mismatches.
   - `cargo test -p t27c --bin t27c`: 1525/0/2.

7. **Close-out and next-wave cooperation.**
   - `docs/reports/WAVE_LOOP_484_CLOSEOUT.md`
   - `docs/reports/FPGA_LOOP_COOPERATION_W485_2026-07-07.md`
   - Update `.trinity/current-issue.md`, `.trinity/ring-484.md`,
     `.trinity/experience.md`, `docs/NOW.md`, and memory.
