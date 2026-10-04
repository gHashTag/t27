# NOW -- FPGA elaboration ratchet: three rows explained and fixed (2026-10-04)

## fpga-conformance is green again at 176 (baseline 176) (Closes #5908, Part of #5906)

### What was read

- The red step printed `elaboration errors: 178 (baseline 176)` with `NEW adapter 0`, `WORSE ternary_isa 6 -> 7` and `WORSE uart 0 -> 1`. Reproduced exactly with master's t27c and iverilog 12.0.
- The baseline header said iverilog 13.0. CI installs iverilog unpinned (`apt-get install -y iverilog`, ubuntu noble `12.0-2build2`). The last green master run, 35677509145 on 6b159942d, ran under 12.0 and printed `176 (baseline 176)`, so the version moved no row.

### Per row

- **ternary_isa 6 -> 7, commit 901ed4fcc (#4594).** That commit added a test with a test-local `given stages = [...]`. W459 binds a `[N]T` parameter to the identifier at its call sites and read that test-local name as a module array. So `pipeline_total_latency` lost `input [263:0] stages;` and indexed a name declared nowhere. Proof: master t27c on `git show 901ed4fcc^:specs/fpga/ternary_isa.t27` gives 6 errors, and on 901ed4fcc it gives 7. Fixed in `bootstrap/src/compiler.rs`: W459 binds only to a module-level const or var, and anything else is passed by value. The new test is `bootstrap/tests/verilog_array_param_test_local.rs`; it failed before the fix and passes after. FROZEN_HASH is updated.
- **uart 0 -> 1, commit 942a00132 (#4651).** That commit applied the reproduction patch of #2364 to the spec itself and replaced `fn on_comb(data: u8) -> bool` with an `on_clock` body. Error: `Could not find variable on_clock in ZeroDSP_UART`. The spec line is restored and gives 0 errors. Its ports `data`/`result` match `contrib/formal/uart_formal_props.v` again.
- **adapter NEW at 0, commit d4a71646f.** A new module with zero errors failed the step, because `if worse or new: return 1` counted a clean new row as a regression. `tools/check_elab_ratchet.py` now reports a zero-error new module as NEW-clean with an `--update-baseline` hint and exits 0. The self-check case `NEW0` was added first and shown failing. `adapter 0` is recorded.

### What changed beyond the rows

- `tools/elab_baseline.txt` was re-taken under iverilog 12.0. The total is unchanged at 176, and the only row added is adapter 0.
- The compiler fix changes the Verilog of 10 specs, with no zig/C/Rust change. Error counts: lexer 71->21, gft_generalize_demo 6->2, uuid 2->1, task_analysis 5->0, views 3->0, ternary_isa 7->6, and three specs unchanged. vsa/ops goes 13->14 because the old output summed one argument three times, and the fix unmasks it.
- Fifteen seals were resealed for the new Verilog hashes. `tools_GftGeneralizeDemo` was resealed with `--force`, because `softmax_picks_the_max_index` already fails with master's t27c. A `tests-fail` row in `tools/seal_baseline.txt` records it.

### What was verified

- The ratchet gives 176 (baseline 176) OK, and `--self-check` passes. Seal currency and seal coverage exit 0.
- The rest of fpga-conformance (gate preconditions, vector data, mac 18 cases, spi 3 cases, summary 34 CLEAN, schema 34 valid) passes locally. On master these steps were skipped after the ratchet failed.
- fpga-lint: yosys parse + hierarchy passes for 37/37 generated modules, and `synth-readiness` reports 38/38 READY. `fpga-build --docker false --synth-only`, with and without `--minimal`, completes under yosys 0.67.

### Not verified

- No board was touched and nothing was flashed. sby is not installed locally, and the uart formal harness stays `.blocked`.
- The #2364 compiler defect is still live although that issue is closed: an `on_clock` body assigns to an undeclared `on_clock`.
- The comment at `.github/workflows/fpga-build.yml` line 839, which says the ratchet fails on every master run, goes stale once this lands.
