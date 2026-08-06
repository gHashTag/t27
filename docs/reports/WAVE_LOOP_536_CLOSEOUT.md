# Wave Loop 536 Closeout — Cocotb reference-model cosimulation gate

**Issue:** #1507  
**Branch:** `wave-loop-536`  
**Closed:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What was delivered

### 1.1 AST JSON export

- Derived `serde::Serialize` on `NodeKind` and `Node` in `bootstrap/src/compiler.rs`.
- Added `--json` to `t27c parse` so external tools can consume the parsed AST
  without re-implementing the parser.
- Updated the `bootstrap/stage0/FROZEN_HASH` seal for the changed compiler
  surface.

### 1.2 Self-checking simulation Verilog dump

- Added `t27c gen-verilog-for-simulation` to emit the same self-checking
  Verilog testbench that `t27c icarus-simulate` runs internally.

### 1.3 Python reference model

- Created `scripts/cocotb_ref_model.py`:
  - Extracts expected literals from `assert_eq` calls inside `test` / `invariant`
    blocks of the t27 AST JSON.
  - Runs the generated Verilog through `iverilog` + `vvp`.
  - Verifies that every statically evaluable test block is reported as
    `[TEST] <name> : PASSED` and that no `FAILED` lines appear.
  - Uses `cocotb_tools.runner` when `cocotb` is importable; falls back to direct
    `iverilog`/`vvp` subprocess invocation otherwise, so the gate works in
    environments where PEP 668 / Python-version constraints prevent a full
    cocotb install.

### 1.4 `t27c icarus-cocotb` CLI gate

- Added `Commands::IcarusCocotb` and `run_icarus_cocotb` in `bootstrap/src/main.rs`.
- The command generates a temporary `DUT.v` and `ast.json`, resolves the actual
  top-level module name from the emitted Verilog (important for specs without an
  explicit `module` declaration), and drives the reference model script.
- Honors `T27_COCOTB_PYTHON` to select a Python interpreter with cocotb.

### 1.5 `tri test --cocotb` suite integration

- Added `cocotb: bool` to `SuiteOptions` and `--cocotb` to the `Suite` CLI in
  `bootstrap/src/main.rs`.
- Added Phase 3e in `bootstrap/src/suite.rs`:
  `Cocotb Reference-Model Cross-Check Gate`.
- The gate runs on the same `w5xx`/`w3xx` regression corpus as the Icarus
  simulation gate, optionally filtered by `--icarus-lowerable`.

### 1.6 Documentation

- Updated `docs/ICARUS_LOWERABLE_BOUNDARY.md` with a new section documenting the
  cocotb reference-model gate, AST JSON export, and simulation Verilog dump.

---

## 2. Validation gates

| Gate | Result |
|---|---|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | Icarus Simulation: 35 passed, 0 failed; Cocotb Reference Model: 35 passed, 0 failed; Seal Verify: 610 passed, 0 failed |
| `t27c icarus-cocotb` on seed witnesses (`w528_function_2d_struct_array_param`, `w529_module_2d_struct_array_const`, `w532_signed_struct_array_field_2d_read`, `w533_module_scalar_struct_read`, `w528_parse_const_2d`) | all OK |

Yosys smoke gate still reports 24 pre-existing baseline failures in legacy
`w3xx` scratch specs. Those specs are outside the Icarus-lowerable subset and
were not touched in this wave.

`cargo test -p t27c --tests` shows one pre-existing failure in
`bitnet_pipeline::sequencer_idle_arms_on_start`, unrelated to the cocotb work.

---

## 3. Residual risks / next-wave seeds

- The Python reference model currently only cross-checks expected literals from
  `assert_eq`. A future wave can extend it to independently evaluate the actual
  expression, making it a true independent reference model rather than a
  literal extractor.
- Cocotb dependency handling is environment-specific (PEP 668, Python 3.14
  compatibility). A future wave could pin a Docker/venv definition or add a
  `pyproject.toml` to make the cocotb path reproducible.
- The gate could be extended to capture VCD traces and compare signal values
  directly, moving beyond log-parsing.

---

*φ² + φ⁻² = 3 | TRINITY*
