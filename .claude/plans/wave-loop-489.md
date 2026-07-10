# Wave Loop 489 Plan — Variant B: close colon struct-literal / struct-local lowering gaps

**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Date:** 2026-07-07  
**Issue:** #1459  
**Branch:** `wave-loop-489`

## 1. Research summary

### Weak points identified
1. **Colon-style struct literals are truncated.** `parse_struct_literal` accepts only `field = value` / `.field = value`. Source specs already contain `{ field: value }`; the parser stops after the field name and truncates the rest of the function/test body.
2. **Function-scope struct locals redeclare per-field regs.** `gen_verilog_stmt` for `StmtLocal` with a struct type unconditionally calls `gen_verilog_local_struct_var_decl`, which emits `s_x`, `s_y`, etc. A second `var s : Pt` in the same scope duplicates those declarations.
3. **Keyword-named struct locals are not escaped.** The base name is passed to `gen_verilog_local_struct_var_decl` raw; names like `assign`, `body`, `initial` collide with Verilog keywords. `verilog_safe_identifier` is only applied to the flattened `base_field` name, not to the base, so `assign_x` is safe but the `assign` reg declaration itself is emitted unescaped.
4. **Array-typed fields of packed scalar struct locals produce illegal Verilog.** When a struct-returning call is bound to a local without an explicit type (`let p = make_pt();`), the backend creates a packed reg `p` and later slices fields with `p[high:low]`. If the field is an array, `p.coords[i]` becomes `p[23:0][i]`, which is illegal.
5. **Test-block struct locals are half-commented.** `gen_verilog_test_stmt` prefixes only the first line with `// `; the recursive `gen_verilog_stmt` emits the remaining field declarations as real Verilog, so Icarus sees duplicate/unhoisted regs.

### Literature / precedent
- **LLHD** (Schuiki et al.) — aggregate-to-structural lowering motivates the t27 per-field memory representation.
- **CIRCT VerilogGeneration** — `disallowLocalVariables` / `disallowPackedArrays` options mirror our Icarus/yosys compatibility concerns.
- **Verilator `V3SplitVar.cpp`** — practical field-split lowering reference for packed/unpacked arrays and structs.
- **Icarus issues #536 / #1134** — packed/unpacked slice restrictions; reinforces that t27 must avoid packed-vector slices for array-typed struct fields.
- **FIRRTL/LoFIRRTL** — staged lowering with hoisting/deduplication passes; bench hoisting in t27 already follows this pattern.

## 2. Work decomposition

### Subtask B.1 — Colon struct-literal parser
**File:** `bootstrap/src/compiler.rs` (`parse_struct_literal`, ~2889)

- Accept `field: value` in addition to `field = value` and `.field = value`.
- Keep the existing guarded recovery: if a field name is followed by neither `=` nor `:`, break out of the loop so the rest of the module is not swallowed.
- Ensure dot-prefixed fields still work.
- Add a parse error for `.field:` missing a value that is consistent with the existing `.field =` error path.

### Subtask B.2 — Function-scope struct-local deduplication
**File:** `bootstrap/src/compiler.rs` (`gen_verilog_stmt` StmtLocal branch, ~14284)

- Before calling `gen_verilog_local_struct_var_decl`, check `local_declared_names` (which already exists from W481) for the struct base name.
- If already declared, skip the declaration and emit only the initializer assignment.
- Record the base name in `local_declared_names` on first declaration.
- Do the same for packed scalar struct locals created in the W482/W483 branch (`local_packed_struct_vars`).

### Subtask B.3 — Keyword/label escaping for struct locals
**File:** `bootstrap/src/compiler.rs` (`gen_verilog_local_struct_var_decl`, `gen_verilog_struct_field_decl`, W482/W483 packed-local branch)

- Escape the base name before it is used as a Verilog identifier: apply `verilog_safe_identifier` to `base_name` when emitting the top-level `reg` for a packed scalar struct local.
- In `gen_verilog_local_struct_var_decl`, pass the escaped base into the recursive field declarations so every generated `base_field` name starts from a safe base. The existing `format!("{}_{}", base_name, fname)` + `verilog_safe_identifier` will then produce `\assign _x` correctly.
- Ensure whole-struct assignment paths (`gen_verilog_try_struct_var_assign`, `gen_verilog_local_struct_var_init`) also look up variables by the original source name while emitting the escaped Verilog name.

### Subtask B.4 — Array-typed fields of packed scalar struct locals
**File:** `bootstrap/src/compiler.rs` (`gen_verilog_stmt` W482/W483 branch, ~14250; field access slicing, ~15347)

- Detect struct types whose flattened fields include an array-typed field.
- For such return values, do **not** create a packed reg. Instead, emit per-field regs/memories exactly like an explicitly typed local struct variable (call `gen_verilog_local_struct_var_decl`), then assign from the packed function result using the existing unpack helpers or a new element-by-element unpack for array fields.
- Update field access so that array-typed fields of these locals resolve to the per-field memory (`p_coords[i]`) rather than a packed slice.
- Keep the existing packed-vector path for scalar-only structs to avoid destabilizing W482/W483 seals.

### Subtask B.5 — Test-block struct-local hoisting
**File:** `bootstrap/src/compiler.rs` (`gen_verilog_test`, `gen_verilog_test_stmt`, ~12396)

- Add a `collect_test_locals` pass (mirror `collect_bench_locals`) and a `test_local_names` / `test_local_prefix` context.
- Hoist `StmtLocal` declarations to module scope before the `initial begin : name_test` block, using the same `gen_verilog_local_decl_hoisted` path benches use.
- After hoisting, `gen_verilog_test_stmt` with `hoist_locals=true` should emit only assignments, not declarations.
- Clear the test-local context after each test block.
- Keep the existing `// ` fallback for non-struct locals until hoisting is proven for all types, but at minimum make struct locals work.

### Subtask B.6 — Witness specs
Add to `specs/scratch/`:

| Spec | Covers |
|------|--------|
| `w489_colon_struct_literal_module.t27` | Module-level const/var using `:` separators and a keyword field name. |
| `w489_colon_struct_literal_function.t27` | Function returning a `:`-style struct literal, assigned to a typed local. |
| `w489_colon_struct_literal_test.t27` | Test block with `var s : S = S{ field: value }`. |
| `w489_colon_struct_literal_recovery.t27` | Malformed colon literal stops at bad field without swallowing module. |
| `w489_local_struct_keyword_name.t27` | `let assign = S{...}`, `let body = S{...}`, `let initial = S{...}`. |
| `w489_local_struct_duplicate_decl.t27` | Redeclare the same struct local name twice in a function and in a test. |
| `w489_packed_scalar_struct_array_field.t27` | `let p = make_pt();` where `Pt` has an array-typed field; access `p.coords[i]`. |
| `w489_imported_struct_return_array_field.t27` | Imported constructor returns a struct with an array field, bound to a packed scalar local. |
| `w489_test_block_struct_local_hoist.t27` | Test block declares a struct local and uses it in assertions. |

### Subtask B.7 — Verification & reseal
- `cd bootstrap && cargo build --release`
- `cargo test -p t27c --bin t27c`
- `./scripts/tri test`
- Gate: 673/673 non-smoke PASS, 0 yosys failures, 0 Icarus failures, 673/673 seal matches, 0 `UNSUPPORTED_ICARUS` placeholders.
- `RESEAL_YES=1 ./scripts/reseal-apply.sh` because `bootstrap/src/compiler.rs` changes.

### Subtask B.8 — Close-out and W490 cooperation variants
- `docs/reports/WAVE_LOOP_489_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md` with three ranked variants.
- Update `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, persistent memory.
- Push `wave-loop-489` and create `wave-loop-490`.

## 3. Acceptance criteria

- Existing `igla/` specs with colon struct literals and keyword-named locals compile and simulate (no new `UNSUPPORTED_ICARUS` placeholders).
- All 673 non-smoke specs PASS.
- All 153 yosys smoke specs PASS.
- All 153 Icarus smoke specs PASS with 0 documented baseline failures.
- 673 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 passed, 0 failed, 2 ignored.
- No new `*.sh` on the critical path (L7 UNITY).
- Every `.t27` spec contains `test`/`invariant`/`bench` (L4 TESTABILITY).

## 4. Risk and mitigation

| Risk | Mitigation |
|------|------------|
| Colon parser exposes many latent backend bugs at once. | Enable parser first, fix each exposed bug under a dedicated subtask, add adversarial witnesses before touching unrelated specs. |
| Array-field packed-local fallback destabilizes W482/W483. | Keep scalar-only structs on the packed path; only array-field structs take the per-field path. |
| Test hoisting duplicates module-level names. | Use the same `_test_{name}_` prefix pattern as bench hoisting and clear context per test. |
| Keyword escaping changes existing seal hashes. | NMSE reseal is expected; run `reseal-apply.sh` once after all changes. |

## 5. Recommended variant

**Variant B** (this plan). It is the natural continuation of W487/W488, has a bounded scope, and directly unlocks existing `igla/` specs that already use colon struct literals.

---

*φ² + φ⁻² = 3 | TRINITY*
