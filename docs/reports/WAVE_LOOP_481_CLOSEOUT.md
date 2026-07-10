# Wave Loop 481 — Close-out Report

**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Variant:** B — reduce the remaining Icarus Verilog baseline by defending the Verilog backend against unresolved field-access lowering.

---

## 1. Goal

Drive the remaining 4 documented Icarus smoke failures from W480 to zero by making `gen-verilog` emit sized zero placeholders instead of unbound identifiers when it cannot resolve a field-access base, while keeping all non-smoke tests, yosys smoke, seals, and Rust unit tests green.

---

## 2. What was changed

### 2.1 `bootstrap/src/compiler.rs`

| Change | Root-cause class | Effect |
|--------|------------------|--------|
| Added `"f32"` to `VALID_CAST_TYPES` in `parse_cast_target_type`. | `results.len() as f32` in `igla/coder/eval.t27` was dropped by parser recovery, leaving `total` referenced but undeclared. | Casts to `f32` are now accepted and preserved through lowering. |
| Added `local_declared_names` and `unsupported_call_result_locals` tracking per function. | Struct-return results from namespace-qualified calls or non-emitted helpers were later used as field-access bases. | Locals initialized by unsupported calls are classified as unresolved so their field accesses do not emit bare `result_field` identifiers. |
| Added `field_access_base_is_unresolved` helper. | Field access on imported struct parameters, open array-of-struct parameters, and unsupported-call result locals emitted identifiers that had no declared register/memory. | Returns `true` when the base is not a known same-file scalar struct, local array, module-level struct array, or primitive scalar parameter. |
| Updated the three `ExprFieldAccess` fallback sites. | All fallback paths previously emitted `base_field` unconditionally. | Now emit `32'd0 /* UNSUPPORTED_ICARUS: unresolved field access base.field */`, keeping generated Verilog legal for Icarus and yosys. |
| Preserved legacy flattening for same-file scalar struct parameters. | Primitive scalar parameters such as `task: u32` must still emit `task_prompt`; same-file scalar struct parameters must still destructure packed inputs. | The helper only treats a parameter as unresolved if its type is not a known same-file struct and not a primitive scalar. |

### 2.2 `docs/reports/gen_verilog_iverilog_smoke_baseline.json`

Updated the documented Icarus baseline from **4 failures** (W480) to **0 failures** (W481). The previous residue in `igla/coder/eval.t27`, `igla/coder/pipeline.t27`, `igla/race/formal.t27`, and `igla/race/rtl.t27` now compiles under Icarus Verilog 12.0.

### 2.3 Witness specs

- `specs/scratch/w481_struct_supplier.t27` — exports a scalar struct `Metric`, a struct-return constructor `make_metric`, and scalar test helpers. It is imported by the witness and provides the cross-file struct type used in the imported-parameter path.
- `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` — exercises four previously-failing patterns:
  1. imported struct parameter field access,
  2. same-file scalar struct parameter field access,
  3. same-file array-of-struct parameter destructure,
  4. struct-return field access on an unsupported imported call.
- Both specs pass yosys smoke, Icarus smoke, and have seals under `.trinity/seals/`.

---

## 3. Verification

```
./scripts/tri test
```

| Phase | Result |
|-------|--------|
| Parse | 652 / 652 PASS |
| Typecheck | 652 / 652 PASS |
| GF16 conformance | OK |
| Gen Zig | 652 / 652 PASS |
| Gen Rust | 652 / 652 PASS |
| Gen Verilog | 652 / 652 PASS |
| Gen Verilog Yosys Smoke | 132 / 132 PASS, **0 failures** |
| Gen Verilog Icarus Smoke | 132 / 132 PASS, **0 documented baseline failures** |
| FPGA board-less smoke gate | OK |
| FPGA standalone lake-package build | OK (~247 s) |
| FPGA smoke gate replay | OK |
| Gen C | 652 / 652 PASS |
| Seal verify | 652 / 652 PASS |
| Fixed point | 0 divergences |

```
cd bootstrap && cargo test -p t27c --bin t27c
```

- **1525 passed, 0 failed, 2 ignored.**

The suite reports `acceptable: true` — there are zero documented Icarus baseline failures and no other failures.

---

## 4. What was not closed (and why)

The Icarus baseline is now **zero**, but it is achieved with conservative placeholders for imported struct parameters and unsupported struct-return calls. Those constructs still do not produce functionally correct Verilog — they compile and simulate, but the values read from unresolved field accesses are zero. Full functional lowering of imported structs and cross-file function calls requires multi-file import lowering, which is intentionally out of scope for this wave.

---

## 5. Next-wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W482_2026-07-10.md` for three W482 variants:

- **Variant A:** formalize the Icarus-supported t27 subset as a Lean 4 predicate and wire it into `tri test`.
- **Variant B (default):** make the W481 placeholders functional for same-file AOS parameters and imported scalar struct parameters by reading imported seals and emitting packed-vector extracts.
- **Variant C:** FPGA live cold-POR / SPI flash boot evidence if the QMTech Wukong XC7A100T and DLC10 cable are available.

---

*φ² + φ⁻² = 3 | TRINITY*
