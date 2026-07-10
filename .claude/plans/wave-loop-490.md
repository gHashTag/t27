# Wave Loop 490 Plan — Variant B: continue gen-verilog struct/call lowering hardening

**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Date:** 2026-07-07  
**Issue:** #1460  
**Branch:** `wave-loop-490`

## 1. Research summary

### Weak points identified

1. **Field access on scalar struct-return calls does not support array-typed leaf fields with indices.**
   `try_emit_scalar_struct_call_field` returns `false` as soon as `index_nodes` is non-empty. For a pattern such as `make_pt(a, b).coords[i]` or `make_grid()[0].coords[1]` the backend falls back to generic field-access emission, producing either an unbound identifier or a packed-vector slice of an unpacked array, which is illegal in Icarus.

2. **Bare imported/same-file constructor calls in expression context may be missed for array-typed struct fields.**
   W489 inlines imported constructors when the call is assigned to a local or passed as an argument. A bare call whose struct type contains an array-typed field and whose result is used directly in an arithmetic expression or a comparison does not currently take the per-field-memory path.

3. **Module-scope `const` / `var` arrays of structs with array-typed fields lack adversarial coverage.**
   The lowering path in `gen_verilog_const` / `gen_verilog_var` already emits multi-dimensional per-field memories for this case, but there is no witness spec that exercises multi-dimensional element types combined with array-typed fields, and the `gen_verilog_struct_rom_elem_init` path may mishandle inferred array types or nested struct literals.

4. **Host-only classification does not consider string/enum-only bodies.**
   `fn_body_has_unlowerable_construct` flags namespace-qualified calls, dynamic `.len`/`.contains` methods, and a small list of builtins, but it does **not** flag functions whose only operation is string manipulation or enum construction. Such functions are still emitted to Verilog and rely on expression-level placeholders to stay syntactically legal, increasing the surface area for Icarus regressions.

### Literature / precedent

- **Icarus Verilog issue #536** — unpacked-array slices are unsupported; reinforces the need to avoid packed-vector slices for array-typed struct fields ([source](https://github.com/steveicarus/iverilog/issues/536)).
- **FIRRTL `LowerTypes.scala`** — canonical per-field flattening of bundles/vectors to ground-typed registers, with `_` delimiters and special handling for memories ([source](https://github.com/freechipsproject/firrtl/blob/master/src/main/scala/firrtl/passes/LowerTypes.scala)).
- **LLHD** — multi-level IR with `extf`/`insf` field/slice operations and memory-to-register promotion; shows the value of keeping field-level structure until late lowering ([paper](https://doi.org/10.48550/arxiv.2004.03494)).
- **Apache TVM PR #14020** — `kIsHostFunc` attribute for backend host-only function classification, similar to t27’s `compute_host_only_functions` ([source](https://github.com/apache/tvm/pull/14020)).
- **Sutherland-HDL synthesizable SystemVerilog papers** — standard synthesis subset distinguishes synthesizable RTL from verification-only constructs, supporting a stricter host-only policy ([source](https://sutherland-hdl.com/papers/2013-SNUG-SV_Synthesizable-SystemVerilog_paper.pdf)).

## 2. Work decomposition

### Subtask B.1 — Array-typed field access on scalar struct-return calls

**File:** `bootstrap/src/compiler.rs` (`try_emit_scalar_struct_call_field`, ~8465)

- Detect when the leaf field type is an array (parsed dimensions non-empty).
- Materialize a packed temporary for the call result as today.
- For **literal indices**, compute the exact element slice and emit `tmp[high:low]`.
- For **variable indices** inside a procedural context, emit a bounded priority mux over all element positions: `(idx == 0) ? tmp[h0:l0] : (idx == 1) ? tmp[h1:l1] : ...`. Use `index_combinations` so multi-dimensional array fields are covered.
- Keep scalar leaf fields on the existing direct-slice path.

### Subtask B.2 — Imported/same-file constructor calls used directly in expressions

**File:** `bootstrap/src/compiler.rs` (`gen_verilog_expr` ExprCall branch, `try_emit_imported_struct_return_call`, `try_emit_struct_literal_packed`)

- Ensure that a bare constructor call whose struct type has array-typed fields is inlined as a packed concatenation in expression context, so downstream field access / arithmetic can operate on the packed vector.
- Add a fast path: if the call result is a scalar struct with only numeric fields, leave the existing packed concatenation behavior untouched.

### Subtask B.3 — Module-scope AOS constants with array-typed fields

**Files:** `bootstrap/src/compiler.rs` (`gen_verilog_const`, `gen_verilog_var`, `gen_verilog_struct_rom_elem_init`)

- Add adversarial witness specs covering:
  - `const pts : [2]Pt = [2]Pt{...}` with `Pt` having `coords: [3]u8`,
  - `const grid : [2][3]Pt = [2][3]Pt{...}`,
  - inferred element type `const pts = [2]Pt{...}`,
  - `var pts : [2]Pt = ...` with a struct-return call initializer.
- Fix any uncovered initialization bug in `gen_verilog_struct_rom_elem_init` for inferred multi-dimensional array literals or nested struct literals.

### Subtask B.4 — Host-only enum/string helper hardening

**File:** `bootstrap/src/compiler.rs` (`fn_body_has_unlowerable_construct`, `compute_host_only_functions`, `gen_verilog_function`)

- Extend the unlowerable detector to flag:
  - string literal construction / concatenation (`+` on string operands),
  - enum literals (`ExprEnumValue`) and `::`-containing identifiers,
  - assignments or returns of `string`, `f32`, `f64`, or enum types.
- Keep the existing reachability algorithm so functions used in test/bench/module contexts are still emitted.
- For newly classified host-only functions, ensure `gen_verilog_function` skips emission and call sites emit a placeholder consistent with the current `UNSUPPORTED_ICARUS` style.

### Subtask B.5 — Witness specs

Add to `specs/scratch/`:

| Spec | Covers |
|------|--------|
| `w490_call_field_array_index_literal.t27` | `make_pt(a,b).coords[2]` with literal index. |
| `w490_call_field_array_index_var.t27` | `make_pt(a,b).coords[i]` with variable index inside a procedural block. |
| `w490_imported_call_field_array.t27` | Imported constructor call field access with array-typed field. |
| `w490_module_aos_const_array_field_2d.t27` | `const grid : [2][3]Pt = [2][3]Pt{...}` and element access. |
| `w490_module_var_aos_call_array_field.t27` | `var pts : [2]Pt = make_pts();` where `Pt` has array field. |
| `w490_host_only_enum_string_helper.t27` | Function using enum/string helpers that must be skipped in Verilog. |

### Subtask B.6 — Verification & reseal

- `cd bootstrap && cargo build --release`
- `cargo test -p t27c --bin t27c`
- `./scripts/tri test --fast` (full run if local FPGA lake build converges)
- Gate: 681/681 non-smoke PASS, 0 yosys failures, 0 Icarus failures, 681/681 seal matches, 0 `UNSUPPORTED_ICARUS` placeholders.
- `RESEAL_YES=1 ./scripts/reseal-apply.sh` because `bootstrap/src/compiler.rs` changes.

### Subtask B.7 — Close-out and W491 cooperation variants

- `docs/reports/WAVE_LOOP_490_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md` with three ranked variants.
- Update `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, persistent memory.
- Push `wave-loop-490` and create `wave-loop-491`.

## 3. Acceptance criteria

- All new witness specs compile, pass yosys/Icarus smoke, and have seals.
- No regression in 681 non-smoke specs, 161 yosys smoke specs, 161 Icarus smoke specs.
- `cargo test -p t27c --bin t27c`: 1525 passed, 0 failed, 2 ignored.
- Zero new `UNSUPPORTED_ICARUS` placeholders.
- No new `*.sh` on the critical path (L7 UNITY).
- Every new `.t27` spec contains `test`/`invariant`/`bench` (L4 TESTABILITY).

## 4. Risk and mitigation

| Risk | Mitigation |
|------|------------|
| Array-field call-access mux is large for big arrays. | Only generate muxes for small fixed arrays; leave a TODO/unsupported fallback for large or unbounded cases. |
| Host-only classification incorrectly skips a function used in tests. | Keep the existing reachability fixpoint; only classify a function host-only when it is dead to all emitted Verilog contexts. |
| Multi-dimensional AOS const initializer edge cases. | Add one witness per dimension pattern and fix uncovered bugs incrementally. |
| NMSE seal churn. | Reseal once at the end after all compiler changes are verified. |

## 5. Recommended variant

**Variant B** (this plan). It is the direct continuation of W487/W488/W489, closes expression-context lowering gaps, and has a bounded, test-driven scope.

---

*φ² + φ⁻² = 3 | TRINITY*
