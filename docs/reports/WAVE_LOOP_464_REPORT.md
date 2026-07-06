# Wave Loop 464 Report

**Date:** 2026-07-08
**Issue:** #1441
**PR:** (to open)
**Branch:** `wave-loop-464`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 464 selected **Variant B** from the W464 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line started in W455–W463. The wave closes three remaining gaps in the
array-parameter clone machinery:

1. **Mixed direct/indirect array-parameter call sites.** A function `g(data)` that
   is called both directly from a module-level/test/bench site and indirectly
   through another array-parameter function `f(data)` now has its direct and
   propagated binding signatures merged into a single resolved clone set.
2. **Struct-literal array arguments.** Array parameters whose element type is a
   struct can now be passed a literal array of struct literals. The backend
   lowers the argument to one Verilog memory per scalar field and resolves
   indexed field access to the correct field memory.
3. **Deterministic clone-name collision guard.** When multiple binding
   signatures would sanitize to the same Verilog function clone name, the
   backend appends a numeric suffix and sorts signatures before assignment so
   output is deterministic across runs.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `struct_fields: HashMap<String, Vec<(String, String)>>` to record the
    declared field order and type of every struct in the current module.
  - Added `array_param_types: HashMap<String, String>` to record the declared
    type of each array parameter of the function currently being emitted.
  - Extended `array_literal_signature_key` to an instance method and added
    `struct_literal_signature_field_key`, which expands struct-literal array
    elements by declared field order for a deterministic ROM signature.
  - Extended `gen_verilog_const` and `gen_verilog_anon_rom` to emit one memory
    per scalar struct field and initialize each field memory independently when
    the array element type is a declared struct.
  - Extended the `ExprFieldAccess` arm in `gen_verilog_expr` so that
    `data[i].field` on an array-of-struct parameter bound to a module-level array
    resolves to `bound_array_field[i]`.
  - Added `array_param_bound_name` and `array_element_is_struct` helpers.
  - Added `unique_clone_name` and a module-wide `used_clone_names` set. Both
    module-level and propagated multi-signature clone creation sites now sort
    signatures by key and call `unique_clone_name`, ensuring deterministic,
    collision-free clone names.

- `specs/scratch/w464_mixed_array_param_call_site.t27`
  - Regression spec where `lookup(data)` is called directly from a `test` block
    and indirectly through `sum_pair(data)`. Emits `lookup_rom_a` / `lookup_rom_b`
    clones merged from direct and propagated signatures.

- `specs/scratch/w464_struct_array_literal.t27`
  - Regression spec with a `Pt{x: u16, y: u16}` struct, a module-level `[3]Pt`
    literal, and `sum_x`/`sum_y` functions that read fields from the array
    parameter. Verifies per-field memory lowering (`pts_x`, `pts_y`) and
    field-indexed access.

- `specs/scratch/w464_clone_name_collision.t27`
  - Regression spec with a two-array-parameter function `lookup2(a, b, idx)`
    called from two different module-level array pairs, exercising multi-array-param
    clone creation with deterministic naming.

- `.trinity/seals/scratch_w464_mixed_array_param_call_site.json`
- `.trinity/seals/scratch_w464_struct_array_literal.json`
- `.trinity/seals/scratch_w464_clone_name_collision.json`
  - Seals for the three new regression specs.

- Resealed affected existing specs:
  - `compiler_Lexing.json`
  - `numeric_GoldenFloatFamily.json`
  - `scratch_w463_nested_array_param_call.json`

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - W464 competitor boundary section added.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - W464 triage section added.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_464_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W464_2026-07-08.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W465_2026-07-08.md`.

---

## Verification

- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `t27c gen-verilog specs/scratch/w464_mixed_array_param_call_site.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION ...'`: **PASS**. Emits merged
  `lookup_rom_a` / `lookup_rom_b` clones.
- `t27c gen-verilog specs/scratch/w464_struct_array_literal.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION ...'`: **PASS**. Emits per-field
  memories `pts_x` / `pts_y` and field-indexed function bodies.
- `t27c gen-verilog specs/scratch/w464_clone_name_collision.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION ...'`: **PASS**. Emits distinct
  `lookup2_rom_a_rom_b` and `lookup2_rom_c_rom_d` clones.
- `./scripts/tri test --fast --json /tmp/tri_test_w464_fast.json`: **ALL TESTS PASSED**
  - Parse: 594 passed, 0 failed
  - Typecheck: 594 passed, 0 failed
  - Gen Zig: 594 passed, 0 failed
  - Gen Rust: 594 passed, 0 failed
  - Gen Verilog: 594 passed, 0 failed
  - Gen Verilog Yosys Smoke: **74 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - Gen C: 594 passed, 0 failed
  - Seal Verify: 594 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`

- Full `./scripts/tri test` (without `--fast`): not completed in this
  environment. Phase 3c-standalone still stalls while `lake` fetches the
  `batteries` dependency from `reservoir.lean-lang.org`; the board-less
  smoke-gate report itself passes.

---

## Blockers

- Physical bench remains unavailable (`dlc10 idcode` reports "DLC10 cable not
  found (VID=0x03FD)"), P12 is unwired, and no automated cold-POR relay gate
  exists.
- No live XADC capture or cold-POR SPI boot this wave.
- Full `./scripts/tri test` Phase 3c-standalone lake build is blocked by a stuck
  `curl` download from `reservoir.lean-lang.org`; the `--fast` path is green.
- GitHub CLI (`gh`) is not authenticated in this environment, so the W464 PR and
  the `wave-loop-465` follow-up cannot be created automatically. They must be
  created manually or after `gh auth login`.

---

## Known limitations

- Mixed direct/indirect merging is exercised by a dedicated scratch spec, but the
  merge still requires the inner call to pass the array parameter as a bare
  identifier. Sliced or reordered arrays are not propagated.
- Struct-literal array arguments require the struct literal to use the
  `.field = value` syntax that the parser already supports; the `field: value`
  syntax is not yet accepted by `parse_struct_literal`.
- Arrays of structs are lowered to one memory per scalar field. This matches the
  field-access lowering but may use more BRAM/LUT resources than a packed
  representation; no resource estimate is performed this wave.
- The clone-name collision guard appends `_1`, `_2`, … when the sanitized base
  name collides. With current t27 identifiers (alphanumeric + underscore) the
  collision path is defensive; real collisions would require array argument names
  that differ only by characters removed during sanitization.

---

## Next wave

Wave Loop 465 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W465_2026-07-08.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
