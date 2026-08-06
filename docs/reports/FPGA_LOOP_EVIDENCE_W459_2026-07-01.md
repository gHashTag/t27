# FPGA Loop Evidence — Wave Loop 459 (2026-07-01)

**Issue:** #1431
**Branch:** `wave-loop-459`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was proven this wave

Wave Loop 459 selected **Variant B**: compiler-backend hardening for the
`gen-verilog` path. The evidence is the generated Verilog, the green regression
gates, and the empty yosys smoke baseline, because the physical FPGA bench is
still blocked.

### 1. Array parameters can be resolved from test/invariant/bench blocks

- A function declared with a fixed-size array parameter (`arr: [4]u16`) and
  called from a `test` block with a module-level array (`set(mem, 1, 0xABCD)`)
  is no longer skipped. The binding analysis collects the test-block call site
  and the emitted Verilog function body references the module array `mem` by
  sanitized name, omitting the bound argument from the scalar port list.
- The regression spec `specs/scratch/w459_array_param_test_call.t27` emits real
  `set`/`get` calls and a real `if (!(...))` assertion inside its test block.

### 2. Test and bench blocks are skipped during yosys synthesis

- `bootstrap/src/suite.rs` now invokes yosys with `read_verilog -sv -DSIMULATION`.
- The existing `` `ifndef SIMULATION `` / `` `endif `` guards around test and
  bench bodies are therefore effective: yosys never sees the procedural
  assertions or function calls that would otherwise trigger constant-function
  evaluation errors.
- The result is an empty `gen_verilog_smoke_baseline.json`: all 63 smoke specs
  pass, including the previously problematic ROM/local-array specs.

### 3. ROM style pragma is synthesizable

- `specs/scratch/w459_rom_style_block.t27` lowers to a module whose ROM
  declaration is preceded by `(* rom_style = "block" *)`.
- The generated module passes `yosys read_verilog -sv -DSIMULATION` without
  syntax errors or unrecognized warnings.

### 4. Regression suite is green on the fast path

- `./scripts/tri test --fast` reports:
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    **583/583 PASS**.
  - Gen Verilog Yosys Smoke: **63/63 PASS** with 0 baseline failures.
  - FPGA Board-Less Smoke Gate: **OK**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `schema_version: "1.0"`, `passed: true`.
  - Fixed Point: 0 divergences.
  - `TOTAL FAILURES: 0`, `ACCEPTABLE: yes`.

### 5. Full-path caveat

The default `./scripts/tri test` could not be completed because Phase 3c-standalone
(`lake build` of the standalone theorem package) stalls while downloading the
`batteries` dependency from `reservoir.lean-lang.org`. The phase-3c smoke-gate
report itself is generated successfully and reports `passed: true`; the hang is
an external network/dependency issue, not a regression in the compiler or FPGA
logic.

---

## Artifacts

- `docs/reports/WAVE_LOOP_459_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W459_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W460_2026-07-01.md`
- `specs/scratch/w459_array_param_test_call.t27`
- `specs/scratch/w459_rom_style_block.t27`
- `.trinity/seals/scratch_w459_array_param_test_call.json`
- `.trinity/seals/scratch_w459_rom_style_block.json`
- `docs/reports/gen_verilog_smoke_baseline.json` (empty)

---

*φ² + φ⁻² = 3 | TRINITY*
