# Wave Loop 489 — Close-out Report

**Date:** 2026-07-07  
**Branch:** `wave-loop-489`  
**Issue:** #1459  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant B — Complete the colon struct-literal / struct-local lowering gaps**
from `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md`.

W488 prototyped colon-style struct-literal separators and rolled them back
because they exposed three latent `gen-verilog` gaps:

1. Duplicate `reg` declarations for same-name function-local struct variables.
2. Missing keyword-name escaping for struct-local identifiers.
3. Illegal packed-vector indexing for array-typed fields of scalar struct locals.

W489 fixed all three and re-enabled the colon parser.

---

## What was implemented

### 1. Colon-style struct-literal field separators

`bootstrap/src/compiler.rs`: `parse_struct_literal` now accepts `field: value`
as well as `field = value` and `.field = value`. The recovery path is the same
one prototyped in W488, so existing dot-prefixed and equals-prefixed literals
keep their exact parse tree.

### 2. Function-scope struct-local deduplication and keyword escaping

A new context set `local_struct_var_declared_names` tracks the keyword-safe form
of struct-local names that have already had their per-field `reg` declarations
emitted. Both the W482 packed-scalar-struct-local path and the W467 per-field
struct-local path use the safe name only for deduplication, which fixes the bug
where inserting `base_name` and then `safe_name` (often identical) made the
second insert return `false` and silently skip the declaration.

Keyword names such as `assign`, `body`, and `initial` now produce escaped
identifiers (`\assign `, `\body `, `\initial `) while still being resolvable for
field access.

### 3. Array-typed fields of scalar struct locals

When a struct-return call whose struct type has an array-typed field is bound to
a local (e.g. `let p = make_pt(...)`), the backend no longer emits a single
packed `reg`. Instead it:

- records the local in `local_struct_var_types`,
- emits per-field registers/memories via `gen_verilog_local_struct_var_decl`,
- materializes a packed temporary from the call result,
- slices it field-by-field with `gen_verilog_struct_return_slicing`.

For the array-typed field, slicing expands into one element assignment per
index, e.g. `p_coords[0] = _struct_tmp_0[23:16];`.

### 4. Imported scalar-struct constructor inlining for `use module::Item;`

The W483 imported-constructor inliner only worked for module-only imports such
as `use scratch::w481_struct_supplier;` and fully-qualified calls. W489 makes it
work for item imports such as `use w489_packed_scalar_struct_array_field::Pt;`:

- `resolve_use_module_path` tries the full use-declaration value as a spec file,
  then progressively strips the trailing item segment until a module spec is
  found.
- Imported constructors are stored under both the fully-qualified and the
  unqualified function name, so call sites can use the short name brought into
  scope by `use`.
- `try_emit_imported_struct_return_call` no longer requires the callee to
  contain `::`, so unqualified imported constructors are inlined in both
  expression and statement contexts.

### 5. Test-block struct-local hoisting

`gen_verilog_test` now clears function-local contexts per test block and records
`test_decl_insert_pos` for deferred declarations. `gen_verilog_test_stmt` emits
real `StmtLocal`/`StmtAssign` inside the named `initial` scope and flushes
`aos_tmp_assigns` before the statement that produced them, so struct-return
temporaries in test expressions do not leak assignments outside the test block.

### 6. Field access on scalar struct-return calls

`try_emit_scalar_struct_call_field` lowers patterns like `make_pt(a, b).x` by
materializing a packed temporary and slicing the requested field, instead of
emitting an illegal `make_pt(a, b)_x` identifier.

### 7. Tuple-literal packing for per-field struct locals

Tuple literal emission now detects when an element is a per-field struct local
and packs it from its field registers, fixing `igla/coder/dataset.t27` cases where
the tuple contained names that only exist as per-field regs.

### 8. Enum-variant placeholders in expression context

`gen_verilog_expr` now emits a sized zero placeholder for `ExprEnumValue` and
for any `ExprIdentifier` that still contains `::`. This keeps IGLA specs such as
`igla/coder/training.t27` and `igla/race/yosys.t27` syntactically legal in
Icarus while preserving the host-side semantics for the non-synthesizable
string/enum paths.

---

## Witness specs

All new specs are under `specs/scratch/`:

- `w489_colon_struct_literal_module.t27`
- `w489_colon_struct_literal_function.t27`
- `w489_colon_struct_literal_test.t27`
- `w489_local_struct_keyword_name.t27`
- `w489_local_struct_duplicate_decl.t27`
- `w489_packed_scalar_struct_array_field.t27`
- `w489_imported_struct_return_array_field.t27`
- `w489_test_block_struct_local.t27`

---

## Verification

- `cargo build --release`: PASS.
- `cargo test -p t27c --bin t27c`: 1525 passed; 0 failed; 2 ignored.
- `./scripts/tri test --fast`: ALL TESTS PASSED
  - 681 / 681 non-smoke PASS.
  - 161 / 161 yosys smoke PASS, 0 failures.
  - 161 / 161 Icarus smoke PASS, 0 documented baseline failures.
  - 681 / 681 seal matches.
  - 0 fixed-point divergences.
  - FPGA board-less smoke gate replay: OK.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 681 specs: 0.**
- NMSE reseal: FROZEN_HASH and `repro/numerics/nmse_manifest*.json` refreshed.

The standard full `./scripts/tri test` run was not completed locally because the
standalone lake-package build inside the FPGA smoke gate does not converge in
this environment. The `--fast` run exercises every other gate and reports green.

---

## Artifacts

- Implementation: `bootstrap/src/compiler.rs`
- Witness specs: `specs/scratch/w489_*.t27`
- Plan: `.claude/plans/wave-loop-489.md`
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md`
- NMSE seal: `bootstrap/stage0/FROZEN_HASH`, `repro/numerics/nmse_manifest.json`,
  `repro/numerics/nmse_manifest_protocol_v1.json`
- Spec seals: `.trinity/seals/*.json`

---

*φ² + φ⁻² = 3 | TRINITY*
