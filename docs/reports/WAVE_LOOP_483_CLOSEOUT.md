# Wave Loop 483 — Close-out Report

**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Variant:** B — continue making the remaining `UNSUPPORTED_ICARUS` placeholders functional; W483 focused on imported struct-return calls.

---

## 1. Goal

Turn the next sized-zero `UNSUPPORTED_ICARUS` placeholder into real,
synthesizable logic: imported scalar struct-return calls. When a spec imports a
zero-argument constructor from another spec and assigns the result to a local,
gen-verilog must produce a packed local, inline the constructor body as a packed
struct literal, and resolve field accesses through slicing.

Keep all non-smoke tests, yosys smoke, Icarus smoke, seals, and Rust unit tests
green.

---

## 2. What was changed

### 2.1 `bootstrap/src/compiler.rs`

| Change | Root-cause class | Effect |
|--------|------------------|--------|
| Added `imported_struct_return_literals: HashMap<String, (String, Vec<(String, Node)>)>`. | Imported struct-return calls had no value to assign to a packed local, so they were replaced by a sized-zero placeholder. | For each importable zero-argument constructor whose body is exactly `return StructLit;`, the map stores the struct type and ordered field initializer nodes so the call can be inlined. |
| Added `load_imported_struct_return_literals`. | The backend needed a way to discover which imported functions are pure struct-literal constructors without a full cross-module lowering pass. | Parses each imported `.t27` spec, checks function parameters and body, and records inlinable constructors keyed by `module::fn`. |
| Updated `imported_struct_return_call`. | Previously relied only on `fn_return_types`, so it could not distinguish a constructible literal from an arbitrary unsupported imported call. | Only recognizes imported struct-return calls that are present in `imported_struct_return_literals`, ensuring a packed local is declared only when the RHS can actually be emitted. |
| Updated the `ExprCall` unsupported-call path. | Namespace-qualified calls were unconditionally replaced by sized-zero placeholders. | Before falling back, checks `imported_struct_return_literals` and emits a synthetic `ExprStructLit` packed concatenation via `try_emit_struct_literal_packed`. |
| Kept `struct_fields` merge from W482. | Imported struct layouts must be available for width/offset math. | Imported layouts remain merged into `struct_fields` under `module::Struct` keys, so `return_width`, `packed_field_offset`, and field slicing work for imported types without per-site changes. |

### 2.2 `bootstrap/src/main.rs`

- Removed stale duplicate match arms for `Commands::ValidateSeals` and
  `Commands::TernaryEncode` that caused unreachable-pattern compile errors after
  the environment picked up stale code paths.

### 2.3 Witness specs

- `specs/scratch/w483_imported_struct_return.t27` — imports `Metric` and
  `make_metric()` from `w481_struct_supplier.t27`, declares a packed local from
  the imported constructor, passes it to a function with an imported scalar struct
  parameter, and asserts the computed result under Icarus. Includes an
  adversarial test with two independent imported constructor calls in one
  function.
- `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` — updated to
  assert the real value returned by an imported struct-return field access
  (`r.value == 10`), turning the former W481 placeholder into a functional check.

All new and updated specs have seals under `.trinity/seals/`.

### 2.4 Seal refresh

`bootstrap/src/compiler.rs` changes altered the generated Verilog comment for
packed scalar struct locals (from `W482` to `W482/W483`) and changed output for
specs that use imported struct-return calls. All seal mismatches were resolved
with a global `t27c seal --save` after the final green suite run.

---

## 3. Verification

```bash
./scripts/tri test --fast
```

| Phase | Result |
|-------|--------|
| Parse | 656 / 656 PASS |
| Typecheck | 656 / 656 PASS |
| GF16 conformance | OK |
| Gen Zig | 656 / 656 PASS |
| Gen Rust | 656 / 656 PASS |
| Gen Verilog | 656 / 656 PASS |
| Gen Verilog Yosys Smoke | 136 / 136 PASS, **0 failures** |
| Gen Verilog Icarus Smoke | 136 / 136 PASS, **0 documented baseline failures** |
| FPGA board-less smoke gate | OK |
| FPGA standalone lake-package build | skipped (--fast) |
| FPGA smoke gate replay | OK |
| Gen C | 656 / 656 PASS |
| Seal verify | 656 / 656 PASS |
| Fixed point | 0 divergences |

```bash
cd bootstrap && cargo test -p t27c --bin t27c
```

- **1525 passed, 0 failed, 2 ignored.**

The suite reports `acceptable: true` — zero documented Icarus baseline failures
and no other failures.

---

## 4. Literature review

The W483 implementation extends the W482 flattening strategy to cross-file
constructors. The same bodies of work are relevant:

- **Multi-stage DSL lowering.** Chisel/FIRRTL and Sparkle/Lean 4 HDL flatten
  record constructors into bit-vector concatenations before RTL. W83's inlined
  imported constructor is the same flattening step, but performed lazily at the
  call site because imported function bodies are not lowered as separate Verilog
  modules.
- **Separate compilation with contract metadata.** Veri-Sure and Bluespec-style
  imports rely on interface files that carry enough type/constructor information
  to inline small pure functions. The `load_imported_struct_return_literals`
  helper is a minimal contract: it reads the imported spec, recognizes pure
  struct-literal constructors, and exports just enough information for the
  backend to inline them.
- **Synthesizable Verilog subsets.** Icarus Verilog 12.0 and formal Verilog
  semantics reward bound identifiers and explicit widths. The W483 change keeps
  the inlined constructor inside the same synthesizable fragment as W482's
  same-file struct literals, so it passes the same smoke gates without new
  baseline failures.

The literature supports the design decision to inline only pure, parameter-less
constructors initially: this keeps the cross-file contract small and avoids
introducing unresolved identifiers or side effects into the generated Verilog.

---

## 5. What was not closed (and why)

- **Dynamic `.len()` / `.contains()` on fixed-size arrays and string literals.**
  Static lowering exists for fixed-size scalar arrays, but string literals and
  array parameters are still not fully covered. These need their own dedicated
  handling pass.
- **Host-side recursive helper shadowing in IGLA specs.** Helpers used for proof
  automation are not emitted to Verilog and remain unsupported.
- **Module-scope wildcard `_` bindings.** These still leave values unbound in
  generated Verilog.
- **Imported functions with non-literal bodies.** Only zero-argument
  constructors whose entire body is a single scalar struct literal are inlined;
  arbitrary imported functions still require a full cross-file lowering pass.

These classes are deliberately larger than one wave and are tracked as future
Icarus/Verilog backend extensions.

---

## 6. Next-wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W484_2026-07-07.md` for three W484
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
