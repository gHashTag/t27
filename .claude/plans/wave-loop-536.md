# Wave Loop 536 Plan — Cocotb reference-model cosimulation gate

**Issue:** #1507  
**Branch:** `wave-loop-536`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points audited

Wave Loop 535 aligned the Rust structural classifier and the Lean 4
`Trinity.IcarusLowerable` predicate, so the Icarus gate now agrees with the
formal model on what is lowerable.  However, the gate still validates generated
Verilog by checking that:

1. `iverilog -g2012` accepts it syntactically, and
2. the self-emitted `$display` assertions report PASS.

This is only one independent source of truth: the Verilog emitter itself generates
the test assertions.  A value-level bug in the emitter can make both the
hardware output and the assertion check wrong in the same way, so the gate
passes silently.  A second, independent reference model is needed.

The cocotb framework lets a Python testbench drive the same Icarus simulation
and compare the simulated DUT against a Python reference evaluator.  W536 scopes
this to the existing lowerable subset and uses the Verilog testbench's
`$display` log as the DUT-side observable, so no DUT port restructuring is
required in this wave.

---

## 2. Scientific literature surveyed

- **"Effective Design Verification – Constrained Random with Python and Cocotb"**
  (Gadde et al., DVCon 2024 / arXiv:2407.10312) — evaluates cocotb against
  SystemVerilog/UVM for ALU, I2C, and ADC IPs.  Key lesson: Python reference
  models integrate easily with cocotb, and the framework works with both
  commercial simulators and open-source Icarus/Verilator.
- **"A Python based Design Verification Methodology"** (Ankitha & Aradhya, JUSST
  2021) — proposes cocotb as a lightweight alternative to UVM for SoC
  verification, emphasizing coroutine-based cosimulation via VPI and easier
  reference-model integration.
- **"An implementation of a Python-based verification environment using PyUVM
  and cocotb"** (Bjerrum et al., SyoSil 2024) — demonstrates scoreboarding and
  reference-model comparison in Python, including C-based reference model glue.
  Lesson: keep the reference model simple and separate from the testbench
  orchestration.
- **Vericert** (OOPSLA 2021) — verified C-to-Verilog HLS.  Its soundness proof
  couples a source semantics with a hardware semantics; W536 mirrors this at the
  testing layer by coupling the t27 source interpreter with the emitted Verilog
  simulation.

Key insight: cosimulation is only as good as the independence of the reference
model.  W536 therefore implements the Python reference evaluator from the *t27
source* (not from the AST or generated code), so it provides a genuine
second opinion.

---

## 3. Decomposed plan

### 3.1 Add AST JSON export

- Extend `Node` and `NodeKind` in `bootstrap/src/compiler.rs` with
  `serde::Serialize` so the parsed AST can be serialized.
- Add a `--json` flag to the `t27c parse` subcommand in
  `bootstrap/src/main.rs`.  When set, emit compact JSON instead of Debug text.
- This gives the Python reference model a stable, machine-readable view of the
  spec without re-implementing the t27 parser in Python.
- Update `bootstrap/stage0/FROZEN_HASH` after `compiler.rs` changes.

### 3.2 Python reference evaluator

Create `scripts/cocotb_ref_model.py` that loads the JSON AST and evaluates:

- Literals: integer, boolean, string (string is non-lowerable but kept for
  completeness diagnostics).
- Identifiers resolved from `const`, `var`, `let`, and function parameters.
- Binary operators: `+`, `-`, `*`, `/`, `%`, `<<`, `>>`, `&`, `|`, `^`, `==`,
  `!=`, `<`, `<=`, `>`, `>=`.
- Unary `!`, `-`.
- Struct literals, field access, array literals, indexing.
- Function calls (same-module, non-recursive for the seed witnesses).
- Statements: `var`/`let`/`const` declarations, assignment, `if`/`else`, bounded
  `for`, bounded `while` (with a fuel limit), `return`, bare calls.
- `assert_eq(expected, actual)` is the primary observable; the evaluator returns
  a list of `(test_name, expected, actual)` results.

The evaluator intentionally covers only the lowerable subset used by the seed
witnesses; unsupported constructs raise a clear `NotImplementedError` rather
than silently producing wrong values.

### 3.3 `t27c icarus-cocotb` subcommand

Add `Commands::IcarusCocotb { input }` in `bootstrap/src/main.rs` and implement
`run_icarus_cocotb`:

1. Verify the spec is Icarus-lowerable with the structural classifier.
2. Generate the self-checking Verilog DUT with
   `Compiler::compile_verilog_for_simulation`.
3. Write three files to a temporary directory:
   - `DUT.v` — the generated Verilog.
   - `test_ref.py` — a cocotb test coroutine that:
     - runs the Python reference evaluator on the spec's `test` blocks,
     - launches the DUT through cocotb's `Timer`,
     - waits for the simulation to finish,
     - parses the Verilog `$display` log for `[TEST] ... : PASSED/FAILED`,
     - reports any mismatch between the reference-model expected value and the
       DUT-reported actual value.
   - `Makefile` — standard cocotb Makefile with `VERILOG_SOURCES`, `MODULE`,
     `TOPLEVEL`, `SIM=icarus`.
4. Spawn `make` in the temporary directory and stream stdout/stderr.
5. Exit non-zero if cocotb reports failures or if the reference model disagrees
   with the DUT log.

### 3.4 Suite integration

- Add `cocotb: bool` to `SuiteOptions` in `bootstrap/src/suite.rs`.
- Add `--cocotb` to the `Suite` command in `bootstrap/src/main.rs`.
- In `run_comprehensive`, after the existing Icarus simulation gate, add
  **Phase 3e: Cocotb Reference-Model Gate** that runs `t27c icarus-cocotb` on
  the same lowerable W5xx scratch witnesses.
- Keep the phase optional and print a clear skip message if `cocotb` is not
  available in the Python environment.

### 3.5 Seed witnesses

Select 3–5 W5xx scratch specs that already have Lean value-preservation
theorems and use only the lowerable subset covered by the Python evaluator:

| Witness | Lean theorem | Coverage exercised |
|---|---|---|
| `w528_function_2d_struct_array_param.t27` | `w529_function_2d_struct_array_param_lowerable` (Lemmas) | 2-D packed AOS parameter, struct fields, function calls |
| `w533_module_scalar_struct_read.t27` | `w511_module_array_field_read_lowerable` pattern | module-level packed scalar struct, array-field read |
| `w535_bounded_while_module.t27` (corpus) | `igla_w535_bounded_while_module_lowerable` | bounded while loop, local vars |
| `w532_signed_struct_array_field_2d_read.t27` | signed scalar-array field read | signed values in packed vectors |
| `w529_module_2d_struct_array_const.t27` | `w529_module_2d_struct_array_const_lowerable` | 2-D AOS module-level constant |

(Exact witness list may be adjusted after the evaluator is implemented and its
coverage is known.)

### 3.6 Validation gates

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` (must stay 0
  Icarus failures / 0 seal mismatches)
- `./scripts/tri test --icarus-simulate --icarus-lowerable --cocotb --fast`
  (new gate; 0 cocotb failures)
- `lake build Trinity.IcarusLowerable.Soundness` (must stay green with zero
  `sorry`)

### 3.7 Documentation

- Update `docs/ICARUS_LOWERABLE_BOUNDARY.md` with a "Cocotb reference-model
  cross-check" section describing the new gate, the Python reference model, and
  the dependency on `cocotb` + `iverilog` + `vvp` + `make`.
- Write `docs/reports/WAVE_LOOP_536_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W537_2026-07-07.md`.

---

## 4. Cooperation variants for Wave Loop 537

### Variant A (recommended): Full port-driven cocotb cosimulation

Restructure the simulation Verilog so the DUT exposes function parameters and
return values as module ports, and drive those ports directly from the cocotb
testbench.  This removes the dependency on parsing the Verilog `$display` log
and enables randomized stimulus beyond the fixed `test` blocks.

### Variant B: Python reference-model regression over the whole corpus

Extend `scripts/cocotb_ref_model.py` to cover all lowerable corpus specs and run
it as a standalone Python gate (without cocotb) against the Zig host execution
or the C backend output.  This decouples reference-model validation from the
Verilog simulator.

### Variant C: Equivalence-proof automation for the lowerability predicate

Add a Rust-to-Lean AST exporter so the structural classifier and the Lean
`Module.isLowerable` predicate can be compared automatically for every scratch
witness and corpus spec.  Closes the last manual alignment gap left by W535's
lenient handling of undefined struct names.

---

*φ² + φ⁻² = 3 | TRINITY*
