# Wave Loop 490 — Close-out Report

**Date:** 2026-07-07  
**Branch:** `wave-loop-490`  
**Issue:** #1460  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant B — Continue gen-verilog struct/call lowering hardening**
from `docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md`.

W490 closes three expression-context gaps that W489 deliberately left on the
statement-local side:

1. Field access with an index on an array-typed field of a scalar struct-return
   call (`make_pt(a, b).coords[i]`).
2. Imported/same-file scalar-struct constructor calls used directly in
   expression context with array-typed fields.
3. Host-only classification for string/enum helper functions.

---

## What was implemented

### 1. `make_pt(a, b).coords[i]` lowering

`bootstrap/src/compiler.rs`:

- `try_emit_scalar_struct_call_field` now accepts index nodes when the leaf
  field is array-typed.
- Literal indices are lowered to a bit slice of a packed temporary.
- Variable indices are lowered to a bounded priority mux over every reachable
  element slice, using the existing `array_of_struct_field_slice` helper by
  treating the scalar struct packed value as a one-element array-of-struct.
- `gen_verilog_expr` ExprIndex branch now dispatches to the scalar struct call
  helper, because the parser builds `make_pt(...).coords[i]` as an ExprIndex
  whose base is a field-access chain.

### 2. Imported constructor calls in expression context

- The imported constructor inliner (`try_emit_imported_struct_return_call`) is
  already reached from the generic ExprCall path when the callee is not emitted.
- The scalar struct call helper materializes a packed temporary for the inlined
  result and slices the selected array element, so patterns such as
  `make_pt(9, 4, 5, 6).coords[1]` now work across module boundaries.

### 3. Host-only string/enum helper hardening

`bootstrap/src/compiler.rs`:

- `fn_body_has_unlowerable_construct` now flags `ExprEnumValue`, string
  literals (`extra_kind == "string"`), and `+` on string operands.
- `compute_host_only_functions` now takes the set of declared enum types and
  flags functions whose return type or any parameter type is `string` or an
  enum type.
- The reachability fixpoint is unchanged, so functions used in tests/benches
  are still emitted.

### 4. Adversarial witness specs

Added to `specs/scratch/`:

| Spec | Covers |
|------|--------|
| `w490_call_field_array_index_literal.t27` | `make_pt(...).coords[2]` with literal index. |
| `w490_call_field_array_index_var.t27` | `make_pt(...).coords[i]` with variable index. |
| `w490_imported_call_field_array.t27` | Imported constructor call field access with array-typed field. |
| `w490_module_aos_const_array_field_2d.t27` | `const grid : [2][3]Pt = [2][3]Pt{...}` with array-typed field. |
| `w490_module_var_aos_call_array_field.t27` | `var pts : [2]Pt = make_pts();` where `Pt` has array field. |
| `w490_host_only_enum_string_helper.t27` | String/enum helper skipped in Verilog. |

### 5. NMSE reseal

Because `bootstrap/src/compiler.rs` changed:

- `bootstrap/stage0/FROZEN_HASH` refreshed.
- `repro/numerics/nmse_manifest.json` and
  `repro/numerics/nmse_manifest_protocol_v1.json` regenerated.
- Per-spec seals in `.trinity/seals/` updated for all 687 specs.

---

## Verification

- `cargo build --release`: green.
- `cargo test -p t27c --bin t27c`: **1525 passed, 0 failed, 2 ignored**.
- `./target/release/t27c suite --repo-root . --fast`:
  - **687 / 687 non-smoke PASS** (681 base + 6 new scratch witnesses).
  - **167 / 167 yosys smoke PASS**, 0 failures.
  - **166 / 166 Icarus smoke PASS**, 0 documented baseline failures.
  - **687 / 687 seal matches**.
  - 0 `UNSUPPORTED_ICARUS` placeholders.
  - 0 FPGA smoke failures.
- NMSE seal: FRESH.

---

## Risk and mitigation

| Risk | Mitigation |
|------|------------|
| Priority mux for variable array-field index grows with array size. | Only generated for fixed-size array-typed struct fields; no unbounded expansion. |
| Host-only classification incorrectly skips a function used in tests. | Reachability fixpoint unchanged; any function reachable from tests/benches/module logic is forced to `must_emit`. |
| Seal churn from compiler change. | Resealed FROZEN_HASH, NMSE manifests, and all per-spec seals. |

---

## Deliverables

- `bootstrap/src/compiler.rs` — W490 lowering changes.
- `specs/scratch/w490_*.t27` — adversarial witness specs.
- `docs/reports/WAVE_LOOP_490_CLOSEOUT.md` — this report.
- `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md` — three W491 variants.
- `docs/NOW.md` — updated.
- `.trinity/current-issue.md` — updated for W491.
- `.trinity/experience.md` — updated.
- Persistent memory entry — `wave-loop-490.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
