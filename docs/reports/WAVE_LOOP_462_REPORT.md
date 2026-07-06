# Wave Loop 462 Report

**Date:** 2026-07-07
**Issue:** #1437
**PR:** (to open)
**Branch:** `wave-loop-462`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 462 selected **Variant B** from the W462 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line started in W455–W461. The wave closes three small but user-visible
gaps around array parameters and module-level calls, and keeps the full suite
green:

1. **Literal array arguments for array parameters.** A call like
   `sum_pair([4]u16{1,2,3,4}, 0, 1)` is now accepted. The backend lowers each
   distinct literal to a module-level anonymous ROM (shared across identical
   literals) and includes the ROM name in the array-parameter binding signature,
   so the existing W461 clone machinery handles it without further changes.

2. **Void-return bare module-level calls skip dummy registers.** A module-level
   bare call to a `void` function is now emitted as a Verilog `task` enable
   inside the module-level `always @(*)` block, instead of being assigned to an
   unnecessary 32-bit fallback dummy register.

3. **Bench-local variables + array-parameter integration coverage.** The new
   `w462_array_param_bench_local.t27` regression spec declares bench-local
   variables and calls an array-parameter function from the bench block, using a
   literal array argument. This exercises the W460 hoisting path and the W461/W462
   clone paths together.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `array_param_anon_roms` to collect anonymous ROMs lowered from literal
    array arguments.
  - Added `array_literal_signature_key` to compute a deterministic ROM name from
    an `ExprArrayLiteral`'s size, element type, and literal element values.
  - Added `array_literal_rom_name` to validate that the literal's declared size
    and element type match the array-parameter type before accepting it.
  - Added `gen_verilog_anon_rom` to emit the lowered ROM with the same style as
    `const [N]T{...}` ROMs (reg array + `initial begin ... end`).
  - Extended the W458/W459/W461 array-parameter binding pass to accept
    `ExprArrayLiteral` array arguments, register the anonymous ROM, and use the
    ROM name as the binding-signature part.
  - Extended `call_array_param_signature` to compute the same ROM-name key for
    literal array arguments, so call sites are redirected to the matching clone.
  - Changed `fn_return_types` registration so every function has an entry; void
    functions are recorded as `"void"`.
  - Updated `dummy_reg_width_for_call` to return `0` for void callees.
  - Updated `gen_verilog_module` to emit void bare module-level calls as
    `task`-enable statements inside the `always @(*)` block, skipping the dummy
    register declaration and assignment.
  - Sorted anonymous ROM emission by ROM name to keep generated output
    deterministic across process restarts and seal hashes stable.

- `specs/scratch/w462_array_param_literal.t27`
  - Regression spec calling a `[4]u16` array-parameter function with two
    different literal arrays from `test` blocks.

- `specs/scratch/w462_void_bare_call.t27`
  - Regression spec with a `void` function called at module scope and exercised
    from a `test` block.

- `specs/scratch/w462_array_param_bench_local.t27`
  - Regression spec that declares bench-local variables and calls an
    array-parameter function with a literal array argument from a `bench` block.

- `.trinity/seals/scratch_w462_array_param_literal.json`
- `.trinity/seals/scratch_w462_void_bare_call.json`
- `.trinity/seals/scratch_w462_array_param_bench_local.json`
  - Seals for the three new regression specs.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - W462 competitor boundary section added.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_462_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W462_2026-07-07.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W463_2026-07-07.md`.

---

## Verification

- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `t27c gen-verilog specs/scratch/w462_array_param_literal.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**, emits
  `_lit_4_u16_1_2_3_4` and `_lit_4_u16_10_20_30_40` anonymous ROMs plus
  `sum_pair__lit_4_u16_1_2_3_4` / `sum_pair__lit_4_u16_10_20_30_40` clones.
- `t27c gen-verilog specs/scratch/w462_void_bare_call.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**, emits
  `task check_call; ... endtask` and `always @(*) begin check_call(); end`
  without a dummy register.
- `t27c gen-verilog specs/scratch/w462_array_param_bench_local.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**, hoists bench-local
  variables and uses the anonymous ROM clone inside the bench block.
- `./scripts/tri test --fast --json /tmp/tri_test_w462_fast.json`: **ALL TESTS PASSED**
  - Parse: 590 passed, 0 failed
  - Typecheck: 590 passed, 0 failed
  - Gen Zig: 590 passed, 0 failed
  - Gen Rust: 590 passed, 0 failed
  - Gen Verilog: 590 passed, 0 failed
  - Gen Verilog Yosys Smoke: **70 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - Gen C: 590 passed, 0 failed
  - Seal Verify: 590 passed, 0 failed
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
- GitHub CLI (`gh`) is not authenticated in this environment, so the W462 PR and
  the `wave-loop-463` follow-up cannot be created automatically. They must be
  created manually or after `gh auth login`.

---

## Known limitations

- Literal array arguments are accepted only when every element is a constant
  literal (`ExprLiteral`). Non-literal elements in an array literal passed to an
  array parameter are rejected with a clear error; users must fall back to a
  named module-level ROM for those cases.
- Anonymous ROM names are derived from the literal's size, element type, and
  sanitized element values. Very large literals produce long identifiers, but
  they remain within Verilog identifier length limits for realistic t27 specs.
- Array-parameter functions can still only be called from module-level
  statements and from `test`/`invariant`/`bench` blocks. A function that itself
  calls an array-parameter function would have no module-level binding site and
  remains unsupported.
- Void bare module-level calls are emitted as `task` enables. If a void function
  performs a dead store to a local variable that is not read inside the
  function, the existing optimizer may eliminate the assignment; the regression
  spec avoids this by using an `assert(true)` body.

---

## Next wave

Wave Loop 463 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W463_2026-07-07.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
