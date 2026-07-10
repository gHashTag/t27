# Ring 482 — Wave Loop 482

**Date:** 2026-07-10  
**Branch:** `wave-loop-482`  
**Variant:** B — make the W481 Icarus Verilog placeholders functional for imported scalar struct parameters, same-file AOS parameters, and same-file struct-return locals.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Goal

Turn the W481 sized-zero placeholders into real, synthesizable logic for
imported scalar struct parameters, same-file array-of-struct parameters, and
same-file struct-return local declarations, while keeping the Icarus smoke
baseline at zero.

## Outcome

W482 implemented functional lowering for all three placeholder classes. The
Icarus smoke gate remains **135 / 135 PASS** with **0 documented baseline
failures**.

Key backend changes in `bootstrap/src/compiler.rs`:
- Added `local_packed_struct_vars` and a `StmtLocal` branch that declares a
  packed `reg [W-1:0]` for locals initialized by same-file scalar
  struct-returning calls.
- Added `imported_struct_fields` and `load_imported_struct_fields` to read
  struct layouts from imported `.t27` specs and merge them into
  `struct_fields` under `module::Struct` keys.
- Added `same_file_struct_return_call` helper for detecting same-file
  struct-return initializers.
- Added a top-level `ExprFieldAccess` handler for field-access chains rooted at
  packed scalar struct locals, including nested struct paths (`o.inner.a`).
- Updated `field_access_base_is_unresolved` so imported scalar struct
  parameters and packed scalar struct locals are treated as resolved.
- Updated `gen_verilog_struct_field_assign` to copy scalar fields from a packed
  source local via slices.

Spec / witness changes:
- `specs/scratch/w482_imported_struct_param.t27` exercises imported scalar
  struct parameter lowering.
- `specs/scratch/w482_struct_return_local_decl.t27` exercises same-file
  struct-return packed locals.
- `specs/scratch/w482_aos_param_functional.t27` exercises functional
  same-file AOS parameter lowering.
- `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` updated to
  assert real imported struct parameter values.
- All affected seals refreshed under `.trinity/seals/`.

## Artifacts

- `docs/reports/WAVE_LOOP_482_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W483_2026-07-10.md`
- `.claude/plans/wave-loop-482.md`
- `specs/scratch/w482_imported_struct_param.t27`
- `specs/scratch/w482_struct_return_local_decl.t27`
- `specs/scratch/w482_aos_param_functional.t27`

## Verification

- `cargo build --release`: PASS
- `cargo test -p t27c --bin t27c`: 1525 passed, 0 failed, 2 ignored
- `./scripts/tri test`: ACCEPTABLE
  - 655 / 655 non-smoke PASS
  - 135 / 135 yosys smoke PASS
  - 135 / 135 Icarus smoke PASS, 0 documented baseline failures
  - 655 / 655 seal matches
  - 0 fixed-point divergences
  - FPGA board-less smoke gate: OK
  - FPGA standalone lake-package build: OK
  - FPGA smoke gate replay: OK

## Next

- Branch: `wave-loop-483`
- Default Variant B: continue making the remaining `UNSUPPORTED_ICARUS`
  placeholders functional (imported struct-return calls, dynamic array
  methods, wildcard bindings, helper shadowing).
