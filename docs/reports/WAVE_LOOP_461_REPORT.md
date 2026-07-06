# Wave Loop 461 Report

**Date:** 2026-07-06
**Issue:** #1435
**PR:** (to open)
**Branch:** `wave-loop-461`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 461 selected **Variant B** from the W461 cooperation plan: with the
physical bench still blocked, finish the `gen-verilog` compiler-backend hardening
items deferred from W460. The wave closes two remaining debt items and keeps the
full suite green:

1. **Legalize module-level bare function calls.** Verilog-2001 interprets a bare
   function-name invocation at module scope as a task enable, which is an error
   when the callee is a `function` (not a `task`). W461 detects bare `ExprCall`
   module-level statements, declares a module-scope dummy `reg` wide enough to
   hold the return value, and emits the call as a blocking assignment inside an
   `always @(*)` block. This lets users write side-effect-free top-level
   expressions naturally.

2. **Array-parameter function cloning for multiple bound arrays.** W458/W459
   required every call site of a function with an array parameter to pass the
   same module-level array. W461 removes that restriction: the binding pass now
   groups call sites by their array-parameter binding signature and emits one
   Verilog `function` clone per unique signature. Each clone references the
   bound module-level array directly and drops the array argument from its
   scalar input port list; call sites are redirected to the appropriate clone.
   The unspecialized original is skipped when clones are emitted.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `toplevel_tmp_counter`, `dummy_reg_width_for_call`, and
    `module_stmt_is_bare_call` to support module-level bare-call legalization.
  - `gen_verilog_module` now pre-declares `_toplevel_<n>_tmp` dummy registers for
    bare calls and assigns them inside the module-level `always @(*)` block.
  - Added `array_param_clones` and `array_param_clone_bindings` fields plus
    `current_array_param_bindings`, `call_array_param_signature`, and
    `gen_verilog_fn_clone` helpers.
  - The W458/W459 binding pass now groups call sites by binding signature.
    When all sites agree it keeps the fast single-binding path; when sites pass
    different module-level arrays it mints a Verilog-safe clone name per
    signature and stores the per-clone array-parameter bindings.
  - `gen_verilog_fn` and `gen_verilog_fn_internal` emit either the original
    function or a clone. Clones reuse the same AST body but use the clone's
    own array-parameter bindings when resolving parameter identifiers and when
    deciding which scalar inputs to emit.
  - `gen_verilog_expr` redirects `ExprCall` to the correct clone based on the
    call site's binding signature, and continues to drop bound array arguments
    from the emitted argument list.
  - `ExprIdentifier` resolves array-parameter names through
    `current_array_param_bindings` so both original functions and clones
    reference the right module-level array.

- `specs/scratch/w461_bare_call_module.t27`
  - Regression spec with a module-level `const [4]u16` ROM, a `sum` function
    reading from it, and a bare `sum(0, 1);` call at module scope. A `test`
    block validates the function still returns the expected value.

- `specs/scratch/w461_array_param_multi_array.t27`
  - Regression spec with two different module-level `const [4]u16` ROMs
    (`rom_a`, `rom_b`) and a `sum_pair` function taking a `[4]u16` parameter.
    The function is called from three `test` blocks, binding first `rom_a`,
    then `rom_b`, then both in the same test. This exercises the clone path.

- `.trinity/seals/scratch_w461_bare_call_module.json`
- `.trinity/seals/scratch_w461_array_param_multi_array.json`
  - Seals for the two new regression specs.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - W461 competitor boundary section added.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_461_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W461_2026-07-06.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W462_2026-07-06.md`.

---

## Verification

- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `t27c gen-verilog specs/scratch/w461_bare_call_module.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**, emits
  `_toplevel_0_tmp` dummy reg and `always @(*) _toplevel_0_tmp = sum(0, 1);`.
- `t27c gen-verilog specs/scratch/w461_array_param_multi_array.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**, emits
  `sum_pair_rom_a` and `sum_pair_rom_b` clones and redirects each call site.
- `./scripts/tri test --fast --json /tmp/tri_test_w461_fast.json`: **ALL TESTS PASSED**
  - Parse: 587 passed, 0 failed
  - Typecheck: 587 passed, 0 failed
  - Gen Zig: 587 passed, 0 failed
  - Gen Rust: 587 passed, 0 failed
  - Gen Verilog: 587 passed, 0 failed
  - Gen Verilog Yosys Smoke: **67 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - Gen C: 587 passed, 0 failed
  - Seal Verify: 587 passed, 0 failed
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
- GitHub CLI (`gh`) is not authenticated in this environment, so the W461 PR and
  the `wave-loop-462` follow-up cannot be created automatically. They must be
  created manually or after `gh auth login`.

---

## Known limitations

- Array-parameter function cloning mints a new Verilog `function` for every
  unique binding signature. If a function is called with many different
  module-level array combinations, the generated module grows linearly with the
  number of distinct signatures. This is bounded by the number of call sites and
  is intended for small on-chip tables, not for dynamic memory virtualization.
- Literal array arguments (e.g. `sum_pair([4]u16{1,2,3,4}, 0, 1)`) are still not
  supported for array parameters.
- Functions with array parameters can still only be called from module-level
  statements and from `test`/`invariant`/`bench` blocks. A function that itself
  calls an array-parameter function would have no module-level binding site and
  remains unsupported.
- Bare module-level calls that ignore the return value are legalized by writing
  to a dummy register. The register is not otherwise consumed, but synthesizers
  treat it as a combinational output of the function and do not flag it as
  unused in the current yosys smoke gate.
- Clone names are derived from the sanitized function name plus sanitized bound
  array names. Very long function or array names could produce long identifiers,
  but they remain within Verilog identifier length limits for realistic t27
  specs.

---

## Next wave

Wave Loop 462 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W462_2026-07-06.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
