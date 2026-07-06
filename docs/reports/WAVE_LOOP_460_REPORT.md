# Wave Loop 460 Report

**Date:** 2026-07-06
**Issue:** #1433
**PR:** (to open)
**Branch:** `wave-loop-460`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 460 selected **Variant B** from the W460 cooperation plan: with the
physical bench still blocked, continue `gen-verilog` compiler-backend hardening.
The wave closes three related debt items:

1. **Preserve `let` bindings through optimization.** The parser already
   accepted `let` but lowered it through the same path as `const`, so copy
   propagation and constant propagation eliminated `let y = x; return y;`
   entirely. W460 records the original keyword (`let`/`const`/`var`) on every
   `StmtLocal` and skips `let` bindings in both optimization passes, restoring
   the three pre-existing cargo-test failures to passing status.
2. **Hoist bench-block local variables to module scope.** Verilog-2005 does not
   allow variable declarations inside procedural `initial begin … end` blocks.
   W460 recursively collects `StmtLocal` nodes inside each bench body, emits
   module-scope `reg` declarations (with per-element arrays) inside the same
   `` `ifndef SIMULATION `` guard as the bench counter, and emits only
   assignments inside the `initial` block. Hoisted names are prefixed with the
   sanitized bench name to avoid collisions across benches.
3. **Multi-site array-parameter scratch spec.** W459 required every call site
   to agree on the same module-level array identifier, but had no regression
   spec exercising more than one binding site. W460 adds a scratch spec where
   a function with a fixed-size array parameter is called from multiple
   `test`/`bench` blocks passing the same module-level array, with `assert_eq`
   checks validating the result.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - `parse_local_decl` now stores `decl.extra_kind = self.current.lexeme.clone()`
    so `StmtLocal` nodes remember whether they were declared with `let`,
    `const`, or `var`.
  - `copy_propagate` skips `StmtLocal` nodes where `extra_kind == "let"`.
  - `const_propagate` skips `StmtLocal` nodes where `extra_kind == "let"`.
  - `gen_verilog_module` now hoists bench-local scalar and array variables to
    module scope before emitting each bench `initial` block, using a
    `_bench_<name>_<var>` prefix for hoisted identifiers.
  - `gen_verilog_test_stmt` takes a new `hoist_locals` flag. For bench blocks it
    emits assignments only; for test blocks it preserves the previous
    comment-only behavior.
  - `gen_verilog_expr` resolves bench-local identifiers through
    `verilog_local_name` so references inside the bench body use the prefixed
    hoisted name.
  - `ExprIndex` rewrites variable-indexed bench-local arrays to the prefixed
    per-element register names.

- `specs/scratch/w460_bench_local_var.t27`
  - Regression spec with a bench block that declares three local `u32`
    variables (`x`, `y`, `z`) and uses them in a function call plus an
    `assert_eq`. Also contains a `test` block exercising the same module-level
    function.

- `specs/scratch/w460_array_param_multi_site.t27`
  - Regression spec with a module-level `const [4]u16` ROM and a `sum_pair`
    function taking that array as a parameter. The function is called from two
    separate `test` blocks, both binding the same array, with `assert_eq`
    checks.

- `.trinity/seals/scratch_w460_bench_local_var.json`
- `.trinity/seals/scratch_w460_array_param_multi_site.json`
  - Seals for the two new regression specs.

- All 585 `.trinity/seals/*.json` files re-sealed to account for the intentional
  gen-verilog output changes (`let` preservation and bench-local hoisting).

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - W460 competitor boundary section added.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_460_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W460_2026-07-06.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W461_2026-07-06.md`.

---

## Verification

- `cargo test -p t27c --bin t27c let_binding`: **PASS** (3/3).
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `t27c gen-verilog specs/scratch/w460_bench_local_var.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**, declarations
  hoisted to module scope.
- `t27c gen-verilog specs/scratch/w460_array_param_multi_site.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**.
- `./scripts/tri test --fast --json /tmp/tri_test_w460_fast.json`: **ALL TESTS PASSED**
  - Parse: 585 passed, 0 failed
  - Typecheck: 585 passed, 0 failed
  - Gen Zig: 585 passed, 0 failed
  - Gen Rust: 585 passed, 0 failed
  - Gen Verilog: 585 passed, 0 failed
  - Gen Verilog Yosys Smoke: **65 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - Gen C: 585 passed, 0 failed
  - Seal Verify: 585 passed, 0 failed
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
- GitHub CLI (`gh`) is not authenticated in this environment, so issue #1435 and
  the `wave-loop-461` branch cannot be created automatically. They must be
  created manually or after `gh auth login`.

---

## Known limitations

- Array-parameter binding still requires **all** call sites to pass the **same**
  module-level array identifier. A function with an array parameter cannot yet
  be called with a literal array or from sites that disagree on the bound array.
- Module-level bare function calls that ignore the return value are still illegal
  in generated Verilog. The new scratch spec places all binding sites inside
  `test` blocks instead.
- Bench-local variable hoisting uses a flat name-prefix scheme. If a user
  manually names a module-level identifier `_bench_<bench>_<local>`, a
  collision is possible; this is not currently detected.
- The `YOSYS_ALLOWED_WARNINGS` allow-list in `bootstrap/src/suite.rs` is kept
  unchanged. The bench-local hoisting removes the procedural-wire warnings for
  bench blocks, but function-local array variable-index selects may still
  produce the same warning class on some specs, so removing the entries now
  could cause regressions in a future wave.

---

## Next wave

Wave Loop 461 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W461_2026-07-06.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
