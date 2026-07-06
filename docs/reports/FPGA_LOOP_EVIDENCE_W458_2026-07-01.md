# FPGA Loop Evidence — Wave Loop 458 (2026-07-01)

**Issue:** #1429
**Branch:** `wave-loop-458`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was proven this wave

Wave Loop 458 selected **Variant B**: compiler-backend hardening for the
`gen-verilog` path. The evidence is the generated Verilog and the green
regression gates, because the physical FPGA bench is still blocked.

### 1. Warning-hygiene fixes are synthesizable

- `t27c gen-verilog specs/scratch/w458_array_param_read.t27` produces a module
  whose test block is guarded by `` `ifndef SIMULATION `` / `` `endif `` and
  contains no `// synthesis translate_off/on` comments.
- `t27c gen-verilog` on `specs/igla/coder/arch.t27` and
  `specs/igla/coder/training.t27` now emits `parameter real` for `f32` scalar
  constants such as `ROPE_THETA = 10000.0` and `WEIGHT_DECAY = 0.1`.
- Multi-line string literals in generated Verilog are escaped (`\n`) before
  emission, eliminating the yosys "unterminated string" warning class.

### 2. Module-level array access from functions is synthesizable

- `w458_array_param_read.t27` lowers to a module with a `reg [15:0] rom [0:3]`
  initialized by `initial begin ... end`, and a Verilog `function lookup`
  whose body reads `rom[i]` directly.
- `w458_array_param_write.t27` lowers to a module with a `var [15:0] mem [0:3]`
  and functions `set` / `get` that assign to and read from `mem[i]` directly.
- Both specs pass `yosys read_verilog -sv; synth -top <module>` without
  syntax errors.

### 3. Regression suite is green on the fast path

- `./scripts/tri test --fast` reports:
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    **581/581 PASS**.
  - Gen Verilog Yosys Smoke: **61/61 PASS**.
  - FPGA Board-Less Smoke Gate: **OK**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `schema_version: "1.0"`, `passed: true`.
  - Fixed Point: 0 divergences.
  - `TOTAL FAILURES: 0`, `ACCEPTABLE: yes`.

### 4. Full-path caveat

The default `./scripts/tri test` could not be completed because Phase 3c-standalone
(`lake build` of the standalone theorem package) stalls while downloading the
`batteries` dependency from `reservoir.lean-lang.org`. The phase-3c smoke-gate
report itself is generated successfully and reports `passed: true`; the hang is
an external network/dependency issue, not a regression in the compiler or FPGA
logic.

---

## Artifacts

- `docs/reports/WAVE_LOOP_458_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W458_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W459_2026-07-01.md`
- `specs/scratch/w458_array_param_read.t27`
- `specs/scratch/w458_array_param_write.t27`
- `.trinity/seals/scratch_w458_array_param_read.json`
- `.trinity/seals/scratch_w458_array_param_write.json`

---

*φ² + φ⁻² = 3 | TRINITY*
