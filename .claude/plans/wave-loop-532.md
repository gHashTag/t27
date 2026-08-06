# Wave Loop 532 — Decomposed Plan

**Date:** 2026-07-07  
**Issue:** #1503  
**Branch:** `wave-loop-532`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points found in the audit

1. **Wrong field-width computation for array-typed struct fields.**
   `VerilogCodegen::type_to_width` returns 32 for `[3]i16` because it only
   recognises bare scalar types. `ExprStructLit` therefore emits scalar-struct
   literals with mis-sized array fields and the packed-vector layout is
   corrupted.

2. **Single scalar structs with array fields are not lowered as packed vectors.**
   `gen_verilog_var` / `gen_verilog_struct` still emit one `reg` per scalar
   field (`pt_data`), so `p.data[1]` compiles to `p_data[1]`, which is a
   non-existent or wrong register.

3. **1-D arrays of scalar structs with array fields are not handled.**
   `try_emit_struct_array_access` rejects arrays whose `dims.len() < 2`, so
   `[2]Pt` with `Pt.data: [3]u16` falls through to the broken per-field
   fallback.

4. **2-D array-of-struct `.data[i]` access is lowered as a bit-select.**
   The current packed slice only supports scalar fields (`grid[i][j].x`);
   `grid[i][j].data[k]` emits the whole 48-bit element followed by `[k]`,
   which selects a bit instead of the k-th 16-bit word.

5. **Signed values are not preserved.**
   Signed literals are emitted as `{width}'d{value}` (invalid for negatives
   in Icarus), and part-selects of signed fields are not wrapped in
   `$signed(...)`, so negative values become large unsigned numbers.

6. **No witnesses.**
   There are no positive or negative scratch specs covering signed
   scalar-array struct fields.

---

## Relevant scientific / technical literature

1. **Vericert — verified C-to-Verilog HLS.**
   Herklotz et al., OOPSLA 2021. The closest academic analogue to t27's
   compiler-to-Icarus pipeline: a CompCert-style verified HLS path that maps
   memory, arrays, and fixed-width integer operations to a Verilog target.
   Takeaway: bit-accurate signed/unsigned integer semantics must be preserved
   explicitly at the Verilog level; the source type system does not carry
   over automatically.

2. **Vitis HLS UG1399 — `ap_[u]int<>` semantics.**
   AMD/Xilinx, 2025–2026. Defines the industry reference for sign-extension
   of narrower signed values assigned to wider destinations and for signed
   operator result widths. Takeaway: signed fixed-width array elements must
   be sign-extended to the element width before concatenation and must keep
   their signedness on read.

3. **Sutherland & Mills — *Standard Gotchas* (SNUG 2006).**
   Documents that Verilog bit-select and part-select results are **always
   unsigned**, even when the vector is declared `signed`. Takeaway: t27 must
   wrap signed field slices in `$signed(...)` or `signed'(...)` to preserve
   negative values.

4. **Chen et al. — *The Essence of Verilog* (OOPSLA 2023).**
   Formal Verilog semantics tested against Icarus Verilog and Verilator,
   reporting real divergences. Takeaway: the Icarus-lowerable subset must
   avoid constructs whose simulation semantics are ambiguous or tool-specific.

---

## Variant A — Signed scalar-array struct fields in packed-vector layout (recommended)

**Goal:** Close the largest remaining gap in the packed-vector Icarus-lowerable
subset: scalar structs whose fields are fixed-size signed scalar arrays.

**Subtasks (decomposed):**

1. **Width helper.** Add / fix a helper that returns the total bit width of a
   scalar-struct field, including `[N]u8/u16/u32/i8/i16/i32` arrays. Use it in
   `element_width`, `struct_field_offset`, and `ExprStructLit`.

2. **Scalar-struct literal emission.** Update `ExprStructLit` so that array-typed
   fields are rendered as nested concatenations (or sized per-element values)
   in the correct order and width. Signed elements must be emitted as
   `-{w}'sd{abs}` for negatives and `{w}'sd{value}` for positives.

3. **Single-scalar-struct packed lowering.** Lower a module/function scalar
   struct variable with array fields as one packed `reg` (or
   `localparam`/`parameter`) of total width, instead of separate per-field
   registers. Initialise it from a struct literal using the same concatenation
   path.

4. **Field-with-index access.** Support `p.data[i]` for a packed scalar struct
   by adding the inner index to the field offset before emitting the part
   select.

5. **Generalise array-of-struct access.** Allow `try_emit_struct_array_access`
   to handle 1-D arrays of scalar structs and to compute the inner index for
   `.data[k]` inside a 2-D array-of-struct element.

6. **Signed slice reads.** When the accessed field type is signed, wrap the
   part-select in `$signed(...)`.

7. **Witnesses.** Add positive scratch specs:
   - `w532_signed_struct_field_read.t27` — read an element from a local signed
     array field.
   - `w532_signed_struct_field_copy.t27` — copy from a module-level const signed
     array field into a local signed array field.
   - `w532_signed_struct_field_param.t27` — pass a scalar struct with a signed
     array field as a function parameter.
   - `w532_signed_struct_field_return.t27` — return a scalar struct with a
     signed array field by value and read it.
   Add negative scratch specs:
   - `w532_negative_enum_field.t27` — struct with an enum field must stay
     non-lowerable.
   - `w532_negative_string_field.t27` — struct with a string field must stay
     non-lowerable.

8. **Verification.** Run `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
   `./scripts/tri test --icarus-simulate --icarus-lowerable`, reseal affected
   specs, record Icarus baselines, and update `FROZEN_HASH` if `compiler.rs`
   changed.

**Why recommended:** It directly removes the largest class of
`UNSUPPORTED_ICARUS` placeholders that still occur on real t27 specs, while
staying inside the existing packed-vector architecture.

---

## Variant B — Adversarial lowerability boundary proofs

**Goal:** Make the lowerability classifier falsifiable in both Rust and Lean 4.

**Subtasks:**
1. Add negative witnesses for non-lowerable constructs: unresolved imports,
   host-only helpers, casts (`as`), enum/string fields in packed arrays, and
   unbounded dynamic loops.
2. State `¬ Module.isLowerable env m` theorems in Lean 4 and discharge them with
   `native_decide` or the classifier predicate.
3. Add a Rust integration test that checks the classifier rejects exactly the
   specs the Lean predicate rejects.
4. Document the boundary so future compiler changes cannot silently expand the
   lowerable subset.

**Why valuable:** A soundness proof is only as strong as the gate it protects.
Adversarial witnesses catch classifier regressions in both proof and code.

---

## Variant C — cocotb reference-model cosimulation

**Goal:** Add a reference-model simulation layer on top of the existing Icarus
simulation gate.

**Subtasks:**
1. Generate a cocotb-compatible testbench wrapper for lowerable t27 specs.
2. Implement a minimal Python reference model that mirrors t27 semantics for the
   lowerable subset.
3. Drive the DUT with pseudo-random inputs and compare outputs.
4. Keep the existing Icarus gate as the fast first line, and run the cocotb gate
   in CI on a scheduled cadence.

**Why valuable:** Reference-model cosimulation is the standard way to catch
value-level semantic drift and produces independently runnable artifacts.

---

## Recommended variant

**Variant A.** Wave Loops 530 and 531 proved that the Icarus simulation gate
finds real bugs and grows safely one shape at a time. Signed scalar-array
struct fields are the largest remaining hole in the packed-vector path;
closing them first is lower risk than adding a new simulation framework or a
full formal adversarial proof before the subset is complete.

---

*φ² + φ⁻² = 3 | TRINITY*
