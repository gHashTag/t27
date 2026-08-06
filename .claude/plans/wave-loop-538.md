# Wave Loop 538 Plan — Extend cocotb reference model with VCD signal comparison

**Issue:** #1509  
**Branch:** `wave-loop-538`  
**Date:** 2026-07-15  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Literature review

1. **SyoSil, "An implementation of a Python-based verification environment using
   PyUVM and cocotb"** — shows the canonical cocotb reference-model pattern:
   a Python/C reference model receives the same transactions as the DUT and a
   scoreboard compares outputs.  This wave adapts the pattern to a static
   source-level reference model that compares waveform values.
2. **JUSST, "A Python based Design Verification Methodology"** — argues that
   Python/cocotb plus a golden C/Python reference model is a viable alternative
   to SystemVerilog/UVM for SoC verification and supports co-simulation with
   simulators such as Icarus.
3. **cocotb simulator support documentation** — Icarus produces VCD/FST when
   `WAVES=1` or when `-vcd <file>` is passed to `vvp`; this wave uses the
   explicit `-vcd` path so the Python side can parse the dump with a standard
   VCD library.
4. **cirosantilli/vcdvcd** — mature Python VCD parser that maps hierarchical
   signal names to time/value dictionaries, enabling direct programmatic
   comparison against a reference value.

Sources:
- <https://www.syosil.com/images/resources/osv_whitepaper-1.0.3.0.pdf>
- <https://jusst.org/wp-content/uploads/2021/06/A-Python-based-Design-Verification-Methodology.pdf>
- <https://docs.cocotb.org/en/development/simulator_support.html>
- <https://github.com/cirosantilli/vcdvcd>

---

## Weak points addressed

- The W536 cocotb gate only parses the simulation log (`[TEST] ... PASSED/FAILED`).
  This checks that the self-checking testbench agrees with itself, but it does
  **not** cross-check the actual expression value against an independent source
  of truth.
- Icarus/Verilog bugs that silently corrupt a value but still pass the
  self-checking comparison would not be caught unless the self-checking logic
  itself is wrong — exactly the bug class we want a reference model to find.
- The existing Python model already evaluates a tiny constant-expression subset
  (`_eval_simple_const`); extending it to variables, function calls, struct
  fields, and array indexing gives a real independent reference evaluator.

---

## Variant A (recommended) — VCD probe + independent expression evaluator

### Subtasks

1. **Emit probe signals in the generated Verilog testbench.**
   - In `VerilogCodegen::gen_verilog_test_stmt`, for every `assert_eq(actual, expected)`
     inside a test block, emit a width-tolerant probe register:
     `reg [63:0] _t27_probe_<N>;` and assign it with the actual expression value
     before the comparison.
   - Tag the assignment with a `$display("[PROBE] <N> = %0d", _t27_probe_<N>);`
     so the log parser can also sanity-check the probe.
   - Guard the new emission with `emit_test_assertions` so synthesis output is
     unchanged.

2. **Capture VCD from the Icarus simulation.**
   - Extend `cocotb_ref_model.py::_run_iverilog_vvp` to pass `-vcd <work>/dump.vcd`
     to `vvp` when VCD comparison is enabled.
   - Extend the cocotb runner path to set `WAVES=1` and locate the produced FST,
     or fall back to the direct `-vcd` path when cocotb is unavailable.
   - The standalone mode must remain the authoritative path because cocotb
     availability is environment-dependent (W536 learning).

3. **Extend the Python reference evaluator.**
   - Promote `_eval_simple_const` into `_eval_expr(env, node)` that supports:
     literals, binary/unary/cast, variable reads from `env`, function calls
     (recursively interpret the called function body), struct field access,
     and scalar array indexing for the Icarus-lowerable subset.
   - Keep the evaluator conservative: if an expression cannot be interpreted,
     skip it with a clear note instead of silently assuming the DUT is correct.

4. **Parse VCD and compare probe values.**
   - Add a lightweight VCD parser fallback and use `vcdvcd` when installed.
   - Read `_t27_probe_<N>` values at the final simulation time and compare them
     with the independently evaluated expected expression.
   - Report mismatches with the block name, probe index, expected value, and
     actual waveform value.

5. **Seed with W5xx/W3xx witnesses.**
   - Run the gate on the existing lowerable regression set.
   - Fix any probe-width or signed-value issues that appear.
   - Record no new baseline changes for the log-based gate; the VCD check is an
     additional layer.

6. **Validation.**
   - `cargo build --release -p t27c` green.
   - `cargo test -p t27c --bin t27c` 1494/0/2.
   - `cargo test -p tri` 78/0.
   - `cargo test -p t27c --test icarus_lowerable` 4/0.
   - `./scripts/tri test --icarus-lowerable --cocotb --fast` 35/35 Icarus PASS,
     35/35 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baselines
     unchanged.
   - `lake build Trinity.IcarusLowerable.Soundness` green / 0 `sorry`.

---

## Variant B — Lean module-level procedural initialization

Extend the Lean 4 formal semantics to cover module-level `const`/`var`
initialization and whole-struct assignment, then add a non-scratch corpus
witness in `specs/igla/` and a value-preservation theorem.

**Why deferred:** This is a large proof-engineering task and does not immediately
address the cocotb gate's missing independent reference value.  It is a natural
follow-up after Variant A is in place.

---

## Variant C — Module-level packed-struct assignment from calls

Add compiler lowering and classifier support for module-level packed-struct
variables initialized from function calls or struct literals, with whole-struct
assignment between module variables.  Add a positive scratch witness and a Lean
lowerability theorem.

**Why deferred:** This extends the Icarus-lowerable subset but is backend work,
whereas the most urgent process debt is the reference-model gap identified in
Variant A.

---

## Decision

Proceed with **Variant A**.  It closes the most concrete weak point in the
existing verification flow and stays within the W538 charter.  The other variants
are recorded as follow-up seeds.

---

*φ² + φ⁻² = 3 | TRINITY*
