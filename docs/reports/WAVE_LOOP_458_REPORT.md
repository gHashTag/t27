# Wave Loop 458 Report

**Date:** 2026-07-01
**Issue:** #1429
**Branch:** `wave-loop-458`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 458 selected **Variant B** from the W458 cooperation plan: with the
physical bench still blocked, continue `gen-verilog` backend hardening. The wave
closes two related debt items:

1. **Module-level array access from functions.** Functions declared inside a
   module can now reference module-level `const` / `var` arrays by name, and a
   `pub fn` can declare an array parameter that is bound to a module-level array
   through a single module-level call site. The bound array is passed by direct
   name reference rather than by scalar value-copy.
2. **Yosys warning hygiene.** The legacy `// synthesis translate_off` /
   `// synthesis translate_on` guards around test and bench blocks are replaced
   with standard `` `ifndef SIMULATION `` / `` `endif ``; `f32` / `f64` scalar
   constants are emitted as `parameter real` / `localparam real`; and string
   literals are escaped before being written into generated Verilog.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Parser now consumes `endmodule` for semicolon-style modules and allows bare
    expression / assignment statements at module level (emitted inside an
    `always @(*)` block in Verilog).
  - String literals are tagged with `extra_kind = "string"` in the AST so the
    Verilog backend can distinguish them from numeric literals.
  - `VerilogCodegen` gained `type_is_float`, array-parameter binding tables,
    and `current_fn_name_original` for binding lookup.
  - `gen_verilog_const` emits `parameter real` / `localparam real` for `f32`/`f64`
    scalar constants instead of bit-vector declarations.
  - `gen_verilog_expr` escapes `\`, `\n`, `\t`, and embedded `"` in string
    literals before writing them.
  - `gen_verilog_module` replaces all `// synthesis translate_off/on` guards
    with `` `ifndef SIMULATION `` / `` `endif `` around test bodies, bench
    counter declarations, and bench `initial` blocks.
  - `gen_verilog_fn` skips array parameters that are bound to a module-level
    array; the function body references the module array by sanitized name.
  - `gen_verilog_expr` for `ExprCall` drops bound array arguments from the
    emitted argument list.
  - Array-parameter binding is resolved by inspecting the single module-level
    `StmtExpr` call site for each function with an array parameter. Conflicting
    or non-identifier arguments produce an emitted error comment and the
    function is skipped.
  - New `tests_w458` unit-test module with four tests:
    - `array_param_read_emitted`
    - `float_param_emits_real`
    - `string_newline_escaped`
    - `no_translate_off_comments`
  - Fixed an infinite-loop edge case in `parse_module_body` when recovery lands
    on a top-level keyword such as `module`.

- `specs/scratch/w458_array_param_read.t27`
  - Regression spec with a module-level `const [4]u16` ROM and a function that
    reads directly from it.

- `specs/scratch/w458_array_param_write.t27`
  - Regression spec with a module-level `var [4]u16` RAM and functions that write
    to and read from it.

- `.trinity/seals/scratch_w458_array_param_read.json`
- `.trinity/seals/scratch_w458_array_param_write.json`
  - Seals for the new regression specs.

- All 581 `.trinity/seals/*.json` files re-sealed to account for the intentional
  gen-verilog output changes.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md` (to be updated in-place at close-out)
  - W458 competitor boundary section added.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_458_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W458_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W459_2026-07-01.md`.

---

## Verification

- `cargo test -p t27c --bin t27c tests_w458`: **PASS** (4/4).
- `t27c gen-verilog specs/scratch/w458_array_param_read.t27` +
  `yosys read_verilog -sv; synth -top w458_array_param_read`: **PASS**.
- `t27c gen-verilog specs/scratch/w458_array_param_write.t27` +
  `yosys read_verilog -sv; synth -top w458_array_param_write`: **PASS**.
- `./scripts/tri test --fast --json /tmp/tri_test_w458_fast.json`: **ALL TESTS PASSED**
  - Parse: 581 passed, 0 failed
  - Typecheck: 581 passed, 0 failed
  - Gen Zig: 581 passed, 0 failed
  - Gen Rust: 581 passed, 0 failed
  - Gen Verilog: 581 passed, 0 failed
  - Gen Verilog Yosys Smoke: **61 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - Gen C: 581 passed, 0 failed
  - Seal Verify: 581 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `ACCEPTABLE: yes`

- Full `./scripts/tri test` (without `--fast`): **could not complete** in this
  environment. It reaches Phase 3c successfully (the smoke-gate report is
  generated and `passed: true`), but hangs in Phase 3c-standalone while
  `lake` fetches the `batteries` dependency from
  `reservoir.lean-lang.org` via `curl`. The `curl` download stalls indefinitely;
  this is an external network / dependency-resolution issue, not a code
  regression. The `--fast` path, which skips the standalone lake-package build,
  is fully green and exercises all compiler changes.

- `cargo test -p t27c --bin t27c`: 1518 passed, **3 pre-existing failures**
  (`let_binding_is_lowered_1401`, `test_let_binding_emitted_c_1401`,
  `test_let_binding_emitted_rust_1401`). These failures also occur on `HEAD~1`
  and are unrelated to the W458 changes.

---

## Blockers

- Physical bench remains unavailable (DLC10 cable not detected, P12 unwired).
- No live XADC capture or cold-POR SPI boot this wave.
- Full `./scripts/tri test` Phase 3c-standalone lake build is blocked by a stuck
  `curl` download from `reservoir.lean-lang.org`; the `--fast` path is green.

---

## Known limitations

- Array parameters can only be bound from a **single** module-level call site that
  passes a module-level array identifier. Functions with array parameters cannot
  yet be called from test/invariant/bench blocks in a way that exercises the
  parameter binding; the regression specs demonstrate direct module array access
  instead.
- The `set` function in `w458_array_param_write.t27` is emitted as a Verilog
  `function` that assigns to the module RAM. Because it is referenced only from
  a test block (which is guarded by `` `ifndef SIMULATION ``), Yosys may report
  implicitly-declared bench wires; the smoke gate still passes.

---

## Next wave

Wave Loop 459 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W459_2026-07-01.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
