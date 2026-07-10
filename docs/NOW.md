# NOW — Wave Loop 483 close-out / Wave Loop 484 next (2026-07-07)

**Last updated:** 2026-07-07

## Wave Loop 484 — Next wave (to be selected from cooperation plan)

- Branch: `wave-loop-484` (to create from `wave-loop-483`)
- Issue: (to be opened)
- PR: (to open after close-out)
- Plan: (to be written at W484 close-out)
- Cooperation W485: (to be written at W484 close-out)

### Not started

- Select one of the three W484 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W484_2026-07-07.md`.
- Default Variant B: make the remaining `UNSUPPORTED_ICARUS` placeholders
  functional for dynamic `.len()` / `.contains()` on fixed-size arrays and
  string literals, host-side recursive helper shadowing in IGLA specs, and
  module-scope wildcard `_` bindings.

---

## Wave Loop 483 — compiler-backend Icarus placeholder hardening: imported struct-return calls (Variant B default)

- Branch: `wave-loop-483`
- Issue: #1453 (to be opened)
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_483_CLOSEOUT.md`
- Cooperation W484: `docs/reports/FPGA_LOOP_COOPERATION_W484_2026-07-07.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - `imported_struct_return_literals` map keyed by `module::fn`, storing the
    fully-qualified struct type and ordered scalar struct-literal initializer
    nodes for imported zero-argument constructors.
  - `load_imported_struct_return_literals` parses imported `.t27` specs and
    recognizes functions whose body is exactly `return Struct { ... };`.
  - `imported_struct_return_call` uses the new map so `StmtLocal` declares a
    packed `reg [W-1:0]` for locals initialized by imported struct-returning
    calls.
  - The `ExprCall` unsupported-call path inlines mapped imported constructors as
    packed concatenations before falling back to a sized-zero placeholder.
  - Existing field-access slicing on packed scalar struct locals works for
    imported struct types because their layouts are already merged into
    `struct_fields` under `module::Struct` keys.

- Witness specs
  - `specs/scratch/w483_imported_struct_return.t27` exercises a packed local
    initialized from `w481_struct_supplier::make_metric()` and an adversarial
    test with two independent imported constructor calls.
  - `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` updated to
    assert the real value of an imported struct-return field access.

- Seals: global reseal of every `.trinity/seals/*.json` because the generated
  Verilog comment for packed scalar struct locals changed from `W482` to
  `W482/W483`.

### Verification

- `cargo build --release`: PASS
- `cargo test -p t27c --bin t27c`: 1525 passed, 0 failed, 2 ignored
- `./scripts/tri test --fast`: ACCEPTABLE
  - 656 / 656 non-smoke PASS
  - 136 / 136 yosys smoke PASS
  - 136 / 136 Icarus smoke PASS, 0 documented baseline failures
  - 656 / 656 seal matches
  - 0 fixed-point divergences
  - FPGA board-less smoke gate: OK
  - FPGA standalone lake-package build: skipped (--fast)
  - FPGA smoke gate replay: OK

---

## Wave Loop 482 — compiler-backend Icarus placeholder hardening: imported scalar struct params, same-file AOS params, struct-return packed locals (Variant B default)

- Branch: `wave-loop-482`
- Issue: #1452 (to be opened)
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_482_CLOSEOUT.md`
- Cooperation W483: `docs/reports/FPGA_LOOP_COOPERATION_W483_2026-07-10.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - `local_packed_struct_vars` and a `StmtLocal` branch that declares a packed
    `reg [W-1:0]` for locals initialized by same-file scalar struct-returning
    calls.
  - `imported_struct_fields` and `load_imported_struct_fields` to parse imported
    `.t27` specs and merge their struct layouts into `struct_fields`.
  - `same_file_struct_return_call` helper.
  - Top-level `ExprFieldAccess` handler for packed scalar struct locals,
    including nested struct paths (`o.inner.a`).
  - Updated `field_access_base_is_unresolved` to treat imported scalar struct
    parameters and packed scalar struct locals as resolved.
  - `gen_verilog_struct_field_assign` copies scalar fields from packed source
    locals via slices.

- Witness specs
  - `specs/scratch/w482_imported_struct_param.t27`
  - `specs/scratch/w482_struct_return_local_decl.t27`
  - `specs/scratch/w482_aos_param_functional.t27`
  - `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` updated to
    assert real imported struct parameter values.

- Seals: global reseal of every `.trinity/seals/*.json` because generated
  Verilog changed for all specs.

### Verification

- `cargo test -p t27c --bin t27c`: 1525 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: ALL TESTS PASSED
  - 655/655 non-smoke PASS
  - 135/135 yosys smoke PASS
  - 135/135 Icarus smoke PASS, 0 documented baseline failures
  - 0 seal mismatches.

---

## Wave Loop 478 — compiler-backend Icarus hardening: packed-vector struct-array lowering + warning gate + adversarial witness (Variant B default)

- Branch: `wave-loop-478`
- Issue: (to be opened)
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_478_CLOSEOUT.md`
- Cooperation W479: `docs/reports/FPGA_LOOP_COOPERATION_W479_2026-07-08.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - Sized literal / cast emission for packed struct/array literal leaves:
    `<width>'d<value>` and `<width>'(expr)` replace `(expr & {width{1'b1}})`.
  - Struct-return slicing expands packed slices into per-element memory writes
    for array-typed struct fields.
  - `packed_width` now recurses through all array dimensions; `packed_field_offset`
    uses it for correct 2-D/3-D packed-vector slicing.
  - Scalar array-typed struct fields are lowered with full outer+inner index
    chains, fixing illegal forms like `pts[31:0][0][0]`.
  - `module_declared_regs` deduplicates per-field `reg` declarations already
    emitted for module-level scalar struct constants/variables.
  - `test_block_names` deduplicates generated `begin : <name>` labels so
    duplicate source test names do not produce illegal named blocks.
  - `gen_verilog_try_local_struct_array_assign` lowers whole-array copy for
    local array-of-struct variables by per-element per-field memory copy.
  - `assert_eq` now emits `assert(...) else $fatal(1, "assertion failed")` so
    simulation-time assertion violations actually fail.

- Spec fixes
  - `specs/scratch/w469_2d_struct_array.t27`: removed the extraneous third
    argument to `set_and_sum_2d`.
  - `specs/scratch/w473_3d_module_var_struct_array.t27`: corrected expected
    value from 1332 to 666.
  - `specs/scratch/w476_adversarial_aggregate_tail.t27`: corrected expected
    values (12→13, 30→27, 17→16).
  - `specs/scratch/w382_ram_lowering.t27`: moved module-level memory writes
    into a function so the fatal assertion observes them.

- Regression spec:
  - `specs/scratch/w478_icarus_struct_array.t27`
    Adversarial witness with local AOS copy, packed scalar-array-field
    parameters, variable-index packed parameter access, module-level element
    access, and fatal `assert_eq`; passes both yosys and Icarus.

- Seals: global reseal of every `.trinity/seals/*.json` because generated
  Verilog changed for all specs.

- `bootstrap/stage0/FROZEN_HASH` and `repro/numerics/nmse_manifest*.json`
  refreshed via `RESEAL_YES=1 ./scripts/reseal-apply.sh`.

### Verification

- `cargo test -p t27c --bin t27c`: 1524 passed; 0 failed; 2 ignored.
- `./scripts/tri test --fast`: ALL TESTS PASSED
  - 646/646 non-smoke PASS
  - 126/126 yosys smoke PASS
  - 106/126 Icarus smoke PASS, 20 failed (documented `igla/` dynamic-method
    baseline)
  - 0 seal mismatches.

---

## Wave Loop 477 — compiler-backend hygiene: function-body declaration hoisting + Icarus Verilog simulation gate (Variant B default)

- Branch: `wave-loop-477`
- Issue: (to be opened)
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_477_CLOSEOUT.md`
- Cooperation W478: `docs/reports/FPGA_LOOP_COOPERATION_W478_2026-07-08.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - Added `hoist_verilog_decls` to move `reg` / `integer` declarations to the top
    of each procedural `begin...end` block.
  - Added `hoist_function_scope_decls` to move function/task-scope declarations
    that appear between `input` lines and the function body to before the first
    executable statement.
  - Added `mask_comments_and_strings` and `line_has_token` so generated string
    literals and comments do not corrupt `begin`/`end` tracking.
  - Pre-split `end else begin` lines before stack processing to avoid duplicating
    the `else begin` branch.
  - Dropped standalone `(* ... *)` attribute lines inside procedural blocks;
    Icarus rejects them and they have no effect on local registers.
  - Hardened `gen_verilog_test_stmt` to emit
    `assert(cond) else $fatal(1, "assertion failed");` for Icarus evaluation.

- `bootstrap/src/suite.rs`
  - Added `iverilog_available()` helper.
  - Added `cmd_gen_verilog_iverilog_smoke` and a new suite phase
    `gen-verilog-iverilog-smoke` after yosys smoke.

- Regression spec:
  - `specs/scratch/w477_hoisting_and_iverilog.t27`
    Adversarial witness with interleaved local-array declarations, assignments,
    and variable-index reads; passes both yosys and Icarus.

- Seals: global reseal of all `.trinity/seals/*.json` because the hoisting
  pass changed generated Verilog for all specs.

- `bootstrap/stage0/FROZEN_HASH` remains stable.

### Verification

- `cargo test -p t27c --bin t27c`: 1524 passed; 0 failed; 2 ignored.
- `./scripts/tri test --fast`: 645/645 non-smoke PASS, 125/125 yosys smoke PASS,
  0 seal mismatches.
- Icarus smoke: 92 passed, 33 failed (baseline — pre-existing W475/W476
  packed-vector struct-array lowering gaps, not introduced by hoisting).

---

## Wave Loop 476 — compiler-backend aggregate tail: local AOS copy initializers + module-array packed parameters + nested whole-struct assignment (Variant B default)

- Branch: `wave-loop-476`
- Issue: (to be opened)
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_476_CLOSEOUT.md`
- Cooperation W477: `docs/reports/FPGA_LOOP_COOPERATION_W477_2026-07-08.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- Regression specs:
  - `specs/scratch/w476_local_aos_copy_init.t27`
    Function-local `[2]Shape` arrays initialized from another local array variable.
  - `specs/scratch/w476_module_aos_param.t27`
    Module-level const `[2]Shape` passed to scalar-struct and AOS parameter functions.
  - `specs/scratch/w476_nested_whole_struct_assign.t27`
    Whole-struct assignment for scalar nested structs and AOS elements.
  - `specs/scratch/w476_adversarial_aggregate_tail.t27`
    Adversarial yosys-elaboration witness combining the three features.

- Seals: added four new scratch seals and resealed
  `specs/scratch/w469_struct_field_array_2d.t27`.

- `bootstrap/stage0/FROZEN_HASH` remains stable.

---

## Wave Loop 475 — compiler-backend aggregate hardening: function-local arrays of structs passed as array parameters + nested-array-field equality + adversarial yosys witness (Variant B default)

- Branch: `wave-loop-475`
- Issue: (to be opened)
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_475_CLOSEOUT.md`
- Cooperation W476: `docs/reports/FPGA_LOOP_COOPERATION_W476_2026-07-08.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - Extended the array-parameter binding pass to treat function-local array
    identifiers as packed-vector arguments, using a shared `__local__` signature
    marker and tracking local-packed indices per function/clone
    (`array_param_local_packed_indices`, `fn_array_param_types`,
    `fn_array_param_names`).
  - Emitted local-packed array parameters as scalar packed-vector inputs whose
    width is the total packed bit width of the declared array type.
  - Added `try_emit_local_packed_array_param_field` to lower field access on a
    packed-vector array parameter to a direct bit slice (literal index) or a
    priority mux (variable index), matching the packing order used at call sites.
  - Updated `ExprCall` to pack local-array arguments into a packed concatenation
    when the callee's array parameter is local-packed, instead of dropping them.
  - Extended `gen_verilog_pack_array_of_struct_expr` to pack memory-mode local
    arrays and module-level arrays whose element struct has array-typed fields,
    so nested-array-field equality can compare both operands as packed vectors.
  - Added scalar-struct equality support for struct parameters whose fields are
    arrays by packing the identifier directly instead of attempting per-field
    register expansion.

- Regression specs:
  - `specs/scratch/w475_local_aos_param.t27`
    Function-local `[3]Pt` arrays passed to `[3]Pt` array-parameter functions.
  - `specs/scratch/w475_nested_field_equality.t27`
    Scalar-struct equality and AOS equality for structs with array-typed fields.
  - `specs/scratch/w475_adversarial_nested_equality.t27`
    Adversarial yosys-elaboration witness combining nested AOS equality,
    local-array parameter passing, and variable-index field access on a
    packed-vector parameter.

- `.trinity/seals/scratch_w475_*.json` — seals for the three new specs.
- All affected `.trinity/seals/*.json` files re-sealed to the new gen-verilog output.
- `bootstrap/stage0/FROZEN_HASH` — refrozen after compiler changes.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
