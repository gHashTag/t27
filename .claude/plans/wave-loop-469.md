# Wave Loop 469 Plan — Multi-dimensional struct arrays + scalar struct lowering

**Issue:** #1447 (to create)  
**Branch:** `wave-loop-469`  
**Variant:** B (default) — continue `gen-verilog` compiler-backend hardening while the physical bench is blocked.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goals

Close four remaining `gen-verilog` struct/array lowering gaps so that the t27 → Verilog pipeline can handle:

1. **Multi-dimensional arrays of structs** — `[M][N]Pt`, including arrays of structs whose fields are themselves arrays.
2. **Module-level scalar struct variables and constants** — `var state : Pt = Pt{...}` / `const state : Pt = Pt{...}`.
3. **Scalar struct parameters** — `fn f(p : Pt)` flattened into multiple Verilog inputs.
4. **Whole-struct comparison** — `a == b` and `a != b` lowered to field-wise equality.

Keep the full suite green and reseal affected IGLA specs. Produce a final report and three W470 cooperation variants.

---

## 2. Decomposed tasks

| # | Task | Owner | Files | Acceptance |
|---|------|-------|-------|------------|
| 1 | **Investigate weak points / refresh competitors** | Queen | `bootstrap/src/compiler.rs`, `docs/reports/T27_VS_FORMAL_HDL_2026.md` | Weak-point list documented; competitor snapshot updated with July 2026 signals. |
| 2 | **Create this plan + W470 cooperation variants** | Queen | `.claude/plans/wave-loop-469.md`, `docs/reports/FPGA_LOOP_COOPERATION_W470_2026-07-08.md` | Plan approved by user (implicit via directive); three variants written. |
| 3 | **Implement multi-dimensional arrays of structs** | Creator (C) | `bootstrap/src/compiler.rs`, `specs/scratch/w469_2d_struct_array*.t27` | New scratch specs compile and pass `tri test --fast` + yosys smoke. |
| 4 | **Implement module-level scalar struct vars/consts** | Creator (C) | `bootstrap/src/compiler.rs`, `specs/scratch/w469_module_scalar_struct*.t27` | Per-field registers emitted; reads/writes/assignments work. |
| 5 | **Implement scalar struct parameters** | Creator (C) | `bootstrap/src/compiler.rs`, `specs/scratch/w469_struct_param*.t27` | Parameter flattened into multiple inputs; field access inside callee works; call sites supply packed concatenation or field list. |
| 6 | **Implement whole-struct comparison** | Creator (C) | `bootstrap/src/compiler.rs`, `specs/scratch/w469_struct_compare*.t27` | `==`/`!=` on struct-typed operands lowered to field-wise AND/OR. |
| 7 | **Run conformance tests and reseal** | Verifier (V) | `.trinity/seals/`, `specs/igla/` | `./scripts/tri test --fast` all PASS; `cargo test -p t27c --bin t27c` green; yosys smoke gate clean; seals regenerated. |
| 8 | **Produce report + W470 cooperation + memory** | Queen | `docs/reports/WAVE_LOOP_469_REPORT.md`, `docs/reports/FPGA_LOOP_EVIDENCE_W469_2026-07-08.md`, `docs/reports/FPGA_LOOP_COOPERATION_W470_2026-07-08.md`, `docs/NOW.md`, `.trinity/current-issue.md`, `~/.claude/projects/-Users-playra-t27/memory/wave-loop-469.md` | All documents updated; branch pushed. |

---

## 3. Detailed implementation notes

### 3.1 Multi-dimensional arrays of structs (`[M][N]Pt`)

**Current state:**
- `parse_array_dimensions("[2][3]u8")` returns `[(2,"[3]u8"),(3,"u8")]`.
- `local_array_elem_is_struct()` only handles one dimension and returns `None` for `[M][N]Pt`.
- `gen_verilog_local_struct_array_decl()` expects a flat `(fname, ftype)` list and a single `array_size`.

**Changes needed:**
1. Introduce `parse_array_elem_type(dims: &[(usize,String)]) -> String` returning the final scalar element type.
2. Change `local_array_elem_is_struct()` to peel off all array brackets and test the ultimate element type against `struct_fields`.
3. Change `gen_verilog_local_struct_array_decl()` / `_init()` / `_elem_init()` to accept multi-dimensional shapes and emit registers named `{base}_{i0}_{i1}_..._{fname}`.
4. Add `try_resolve_local_multi_dim_struct_index()` analogous to the scalar 2D path; emit literal-index reads directly and variable-index reads as a priority mux over all Cartesian combinations.
5. Update `gen_verilog_try_struct_array_element_assign()` and `gen_verilog_try_struct_array_assign()` to handle multi-dimensional indices.
6. Handle struct fields that are themselves arrays (already partly supported for 1D arrays in W467; extend to N-D).

### 3.2 Module-level scalar struct vars/consts

**Current state:**
- `gen_verilog_const()` only lowers *array* consts whose element type is a struct.
- `gen_verilog_var()` only lowers *array* vars.
- `gen_verilog_struct()` emits `struct_type_field` regs but the comment says it falls back to the struct type name when no `var` name is found.

**Changes needed:**
1. In `gen_verilog_const()` scalar branch, detect `struct_fields.contains_key(&node.extra_type)` and emit per-field regs/memories using `gen_verilog_struct_field_decl()` with prefix = sanitized variable name.
2. In `gen_verilog_var()` scalar branch, do the same.
3. Register the flattened fields in a new `module_scalar_struct_fields: HashMap<String, Vec<(String,String)>>` so `ExprFieldAccess` on the variable knows which Verilog identifier to emit.
4. In `ExprFieldAccess` for `ExprIdentifier` base, check `module_scalar_struct_fields` and emit `{base}_{fname}`.
5. Support whole-struct assignment at module level (`state = Pt{...}`) via the existing `StmtAssign` → `gen_verilog_try_struct_var_assign()` path extended to module-level scalar structs.

### 3.3 Scalar struct parameters

**Current state:**
- `gen_verilog_fn_internal()` emits one `input` per parameter using `type_to_width`, so `p : Pt` becomes a single `[31:0]` input.
- `ExprFieldAccess` on a parameter treats it as `{param}_{field}`.

**Changes needed:**
1. Detect scalar struct parameters in `gen_verilog_fn_internal()` and emit one `input` per flattened leaf field, named `{pname}_{fname}`.
2. Track scalar struct parameter metadata (e.g. `scalar_struct_params: HashMap<String, Vec<(String,String)>>`) and, for a given parameter name, treat field access as the corresponding input.
3. At call sites, when the argument is a struct variable/literal/call, supply the arguments in the order expected by the flattened inputs (most likely `{p_x, p_y}` as individual args, or a packed concatenation if the callee accepts one input). Simpler: emit individual inputs and pass individual field expressions.

### 3.4 Whole-struct comparison

**Current state:**
- `ExprBinary ==/!=` emits a single Verilog `==` on the operands.
- For struct operands this produces a width mismatch or silent wrong result.

**Changes needed:**
1. In `ExprBinary ==/!=`, when either operand is a struct-typed identifier/call/literal, resolve the struct type.
2. Emit a field-wise comparison:
   - `==` → `((a_x == b_x) && (a_y == b_y))`
   - `!=` → `((a_x != b_x) || (a_y != b_y))`
3. Support local struct vars, module-level scalar struct vars, struct-return calls, and struct literals.

---

## 4. Regression specs to add under `specs/scratch/`

- `w469_2d_struct_array.t27` — `[2][3]Pt`, literal init, literal read, variable-index read/write, nested struct fields.
- `w469_2d_struct_array_param.t27` — function parameter of type `[2][3]Pt` passed from module-level const.
- `w469_struct_field_array_2d.t27` — `Pt { coords : [2][3]u8 }` at module/local/array-element level.
- `w469_module_scalar_struct.t27` — module-level `var` and `const` of struct type.
- `w469_module_scalar_struct_assign.t27` — whole-struct assignment between module-level scalar struct vars.
- `w469_struct_param.t27` — `fn f(p : Pt) -> u32` and call sites with literal/variable/returned struct.
- `w469_struct_compare.t27` — `==` and `!=` on local and module-level struct variables and literals.
- `w469_struct_compare_return.t27` — comparison of struct-return function results.

---

## 5. Verification commands

```bash
# Build
./scripts/tri build

# Fast conformance sweep (non-smoke)
./scripts/tri test --fast

# Rust unit tests
cargo test -p t27c --bin t27c

# Yosys smoke gate
./scripts/tri test --smoke

# Specific scratch specs
cd bootstrap && cargo run --release -- gen-verilog ../specs/scratch/w469_2d_struct_array.t27
```

Success criteria:
- `./scripts/tri test --fast` ≥ previous count (610/610 baseline) and no new failures.
- yosys smoke gate ≥ previous count (90/90 baseline) and no new failures.
- `cargo test -p t27c --bin t27c` 1524 passed, 0 failed.
- All new scratch specs pass.

---

## 6. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Multi-dimensional struct flattening touches many code paths (decl/init/read/write/assign/field access). | Add scratch specs first, implement one dimension at a time, run smoke after each sub-feature. |
| Scalar struct parameter flattening changes function signatures, potentially breaking existing struct-array specs if parameters are mis-classified. | Only flatten parameters whose type is in `struct_fields` and not an array type. |
| Module-level scalar struct vars may collide with existing `gen_verilog_struct()` emission when the struct type is used as a fallback prefix. | Use the variable name as the prefix and skip the old `struct_type_field` fallback for variables that have a dedicated scalar struct entry. |
| Whole-struct comparison could recurse into nested structs and array fields incorrectly. | Reuse `flatten_struct_fields` for leaf scalar comparison; skip array fields in v1 or compare element-by-element only for small fixed sizes. |
| Time overruns one wave. | Variant C fallback ready: formal lemmas for struct-return packing and 2D indexing if implementation cannot be completed cleanly. |

---

## 7. Deliverables

- [ ] `bootstrap/src/compiler.rs` changes (single commit with `Closes #1447`).
- [ ] 8 new scratch regression specs.
- [ ] Resealed IGLA specs (if any changed).
- [ ] `docs/reports/WAVE_LOOP_469_REPORT.md`.
- [ ] `docs/reports/FPGA_LOOP_EVIDENCE_W469_2026-07-08.md`.
- [ ] `docs/reports/FPGA_LOOP_COOPERATION_W470_2026-07-08.md`.
- [ ] Updated `docs/NOW.md` and `.trinity/current-issue.md`.
- [ ] Memory entry `~/.claude/projects/-Users-playra-t27/memory/wave-loop-469.md`.
- [ ] Branch `wave-loop-469` pushed.

---

*φ² + φ⁻² = 3 | TRINITY*
