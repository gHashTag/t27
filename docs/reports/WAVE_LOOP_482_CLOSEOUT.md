# Wave Loop 482 — Close-out Report

**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Variant:** B — make the W481 Icarus Verilog placeholders functional for imported scalar struct parameters, same-file array-of-struct parameters, and same-file struct-return locals.

---

## 1. Goal

Turn the sized-zero `UNSUPPORTED_ICARUS` placeholders from W481 into real,
synthesizable logic for three common classes:

1. imported scalar struct parameters,
2. same-file array-of-struct (AOS) parameters,
3. same-file struct-return local declarations.

Keep all non-smoke tests, yosys smoke, Icarus smoke, seals, and Rust unit tests
green.

---

## 2. What was changed

### 2.1 `bootstrap/src/compiler.rs`

| Change | Root-cause class | Effect |
|--------|------------------|--------|
| Added `local_packed_struct_vars: HashMap<String, String>` and tracked it per function. | A local initialized by a same-file struct-returning call had no per-field registers, so reads like `r.x` were unresolved. | Scalar struct-return locals are declared as a single packed `reg [W-1:0]` and field accesses are emitted as packed slices. |
| Added `imported_struct_fields: HashMap<String, Vec<(String, String)>>` and `load_imported_struct_fields`. | Imported struct parameter types were unknown to `gen-verilog`, so their field accesses fell back to placeholders. | Imported `.t27` specs are parsed, their struct layouts are merged into `struct_fields` under `module::Struct` keys, and the existing scalar-struct parameter unpack path can now destructure imported parameters. |
| Added `same_file_struct_return_call` helper. | The `StmtLocal` packed-local branch needed a reliable way to detect that an initializer is a same-file scalar struct-returning call. | A local like `let r = make_struct()` is recognized and emitted as a packed reg when the callee returns a same-file struct type. |
| Added a top-level `ExprFieldAccess` handler for packed scalar struct locals. | Nested paths such as `o.inner.a` on a packed local previously fell through to malformed or width-1 emission. | Collects the full field-index path, walks intermediate struct fields, and emits one correct slice `base[high:low]` using accumulated packed offsets. |
| Updated the simple-identifier `ExprFieldAccess` fallback. | Direct field access on a packed local (`o.x`) still used per-field register logic. | Now emits a packed slice when the base is in `local_packed_struct_vars`. |
| Updated `field_access_base_is_unresolved`. | Imported scalar struct parameters and same-file struct-return locals were classified as unresolved. | They are now treated as resolved because their layouts and packed locals are known. |
| Updated `gen_verilog_struct_field_assign`. | Copying a scalar struct local into per-field registers generated unbound wires when the source was packed. | Scalar fields are copied by slicing the packed source local. |

### 2.2 Witness specs

- `specs/scratch/w482_imported_struct_param.t27` — imports a scalar struct from
  `w481_struct_supplier.t27`, passes it as a packed-vector function parameter,
  and asserts real field values under both yosys and Icarus.
- `specs/scratch/w482_struct_return_local_decl.t27` — declares a local from a
  same-file struct-returning call, reads simple and nested fields through the
  packed local, and asserts the values.
- `specs/scratch/w482_aos_param_functional.t27` — passes a module-level
  array-of-struct constant into a recursive function and asserts element field
  values under Icarus.
- `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` — updated to
  assert the imported scalar struct parameter value directly in generated
  Verilog, turning the W481 placeholder into a functional check.

All new and updated specs have seals under `.trinity/seals/`.

### 2.3 Seal refresh

`bootstrap/src/compiler.rs` changes altered generated Verilog for every spec.
All seal mismatches were resolved with `t27c seal --save` after the final green
suite run.

---

## 3. Verification

```bash
./scripts/tri test
```

| Phase | Result |
|-------|--------|
| Parse | 655 / 655 PASS |
| Typecheck | 655 / 655 PASS |
| GF16 conformance | OK |
| Gen Zig | 655 / 655 PASS |
| Gen Rust | 655 / 655 PASS |
| Gen Verilog | 655 / 655 PASS |
| Gen Verilog Yosys Smoke | 135 / 135 PASS, **0 failures** |
| Gen Verilog Icarus Smoke | 135 / 135 PASS, **0 documented baseline failures** |
| FPGA board-less smoke gate | OK |
| FPGA standalone lake-package build | OK |
| FPGA smoke gate replay | OK |
| Gen C | 655 / 655 PASS |
| Seal verify | 655 / 655 PASS |
| Fixed point | 0 divergences |

```bash
cd bootstrap && cargo test -p t27c --bin t27c
```

- **1525 passed, 0 failed, 2 ignored.**

The suite reports `acceptable: true` — zero documented Icarus baseline failures
and no other failures.

---

## 4. Literature review

The W482 implementation sits at the intersection of three bodies of work:

- **Ternary synthesis and FPGA mapping.** Kim *et al.* (KNU) and the Tlsys
  project establish RTL-to-gate flows for balanced-ternary datapaths; Beckett's
  ternary FPGA and the Trinity B002 note show that ternary operators can be
  realized on commercial binary FPGAs with encoding-based transformations.
  W482's packed-vector struct lowering is a binary-FPGA-friendly encoding of
  composite ternary values.
- **Embedded DSL multi-stage lowering.** Chisel's FIRRTL pipeline and the
  Sparkle Lean 4 HDL work demonstrate how struct/record types are flattened
  into bit-vectors before RTL emission. The nested packed-offset math in W482
  mirrors that flattening, but computes offsets from the t27 struct-field
  registry rather than from a separate IR.
- **Verilog simulator strictness and semantics.** Icarus Verilog 12.0 and
  HOL4-based Verilog semantics (e.g., the Cambridge VSTTE work) reward
  conservative, synthesizable subsets. The W481 placeholder discipline and the
  W482 functional replacement both rely on keeping every emitted identifier
  bound to a declared wire, reg, or port — the property these semantic accounts
  require.
- **Contract-aware RTL generation.** Veri-Sure and similar tools tie RTL
  emission to frontend type information. W482's imported struct layout
  discovery is a lightweight version of that contract: imported seals carry the
  struct shape so the backend can emit correct extracts without a full
  cross-module lowering pass.

The literature supports the design decision to keep t27 struct semantics explicit
in the compiler and lower to packed vectors late, rather than trying to recover
layout from generated Verilog.

---

## 5. What was not closed (and why)

- **Cross-file struct-return calls.** A function in one spec that returns a
  struct and is called from another spec still produces a placeholder because
  there is no cross-file function-body lowering pass.
- **Dynamic string/array methods.** `.len()` / `.contains()` on runtime-sized
  strings or arrays are not yet lowered to synthesizable Verilog.
- **Host-side recursive helpers.** IGLA specs that use recursive helpers for
  proof automation are still unsupported in Verilog.
- **Module-scope wildcard `_` bindings.** These still drop values on the Verilog
  floor and may leave references unbound.

These classes are deliberately larger than one wave and are tracked as future
Icarus/Verilog backend extensions.

---

## 6. Next-wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W483_2026-07-10.md` for three W483
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
