# Ring 483 — Wave Loop 483

**Date:** 2026-07-07  
**Branch:** `wave-loop-483`  
**Variant:** B — make the remaining `UNSUPPORTED_ICARUS` placeholders functional (imported struct-return calls, dynamic array methods, wildcard bindings, helper shadowing).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Goal

Continue turning `UNSUPPORTED_ICARUS` placeholders in the gen-verilog backend
into real, synthesizable logic. The W483 focus was imported struct-return calls
(cross-file scalar struct constructors) because they blocked the most common
pattern of assigning an imported constructor result to a local and reading its
fields.

## Outcome

W483 implemented functional lowering for imported scalar struct-return calls.
When an imported function has no parameters and its body is exactly `return
Struct { ... };`, gen-verilog now inlines the call as a packed struct literal at
the call site. The imported struct layout is already loaded into `struct_fields`
under a `module::Struct` key, so the packed concatenation has the correct bit
order and field-access slicing works unchanged.

The Icarus smoke gate remains **136 / 136 PASS** with **0 documented baseline
failures**.

Key backend changes in `bootstrap/src/compiler.rs`:
- Added `imported_struct_return_literals` map keyed by fully-qualified imported
  function name. Each entry stores the fully-qualified struct type and the
  ordered scalar struct-literal initializer nodes.
- Added `load_imported_struct_return_literals` to parse imported specs and
  recognize zero-argument functions whose body is a single `return StructLit;`.
- Updated `imported_struct_return_call` to use the new map, so `StmtLocal`
  declares a packed `reg [W-1:0]` for locals initialized by imported
  struct-returning calls.
- Updated the `ExprCall` unsupported-call path to inline mapped imported
  constructors as packed concatenations before falling back to the sized-zero
  placeholder.
- Removed stale duplicate match arms for `Commands::ValidateSeals` and
  `Commands::TernaryEncode` in `bootstrap/src/main.rs`.

Spec / witness changes:
- `specs/scratch/w483_imported_struct_return.t27` exercises a packed local
  initialized from `w481_struct_supplier::make_metric()` and passed to a
  function with an imported scalar struct parameter.
- `specs/scratch/w483_imported_struct_return.t27` also contains an adversarial
  test with two independent imported struct-return calls in one function.
- `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` updated to
  assert the real value returned by an imported struct-return call (`r.value ==
  10`) and renamed the formerly "unsupported" helper.
- Global reseal of every `.trinity/seals/*.json` because the generated Verilog
  comment for packed scalar struct locals changed from `W482` to `W482/W483`.

## Artifacts

- `docs/reports/WAVE_LOOP_483_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W484_2026-07-07.md`
- `.claude/plans/wave-loop-483.md` (to be written)
- `specs/scratch/w483_imported_struct_return.t27`

## Verification

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

## Next

- Branch: `wave-loop-484`
- Default Variant B: continue making the remaining `UNSUPPORTED_ICARUS`
  placeholders functional (dynamic `.len()` / `.contains()` on fixed-size
  arrays and string literals, host-side recursive helper shadowing in IGLA
  specs, module-scope wildcard `_` bindings).
