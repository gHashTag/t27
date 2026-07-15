# Wave Loop 539 — Typed 64-bit VCD probe + full Python expression evaluator

**Issue:** #1510 (placeholder — to create when GitHub token is available)
**Branch:** `wave-loop-539`
**Status:** planned
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 538's VCD probe cross-check and advance the
recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W539_2026-07-15.md`.

**Variant A (recommended):**
- Add type/width inference to `scripts/cocotb_ref_model.py` so the VCD probe
  comparison knows the expected bit width and signedness of each assertion.
- Extend the Python reference evaluator to handle variable reads,
  parameterless function calls, struct field access, and scalar array indexing
  for the Icarus-lowerable subset.
- Emit width-typed probes (`reg [W-1:0]`) when the expression width is
  statically known, and skip only genuinely non-scalar assertions.
- Seed with W5xx/W3xx witnesses that currently skip due to non-literal
  expecteds and verify they now get an independent VCD cross-check.
- Keep `./scripts/tri test --icarus-lowerable --cocotb --fast` at 0 cocotb
  failures and 0 seal mismatches.
- Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Variant B:** Formalize VCD-time value preservation in Lean by defining a
source-level expression denotation and relating it to the emitted Verilog
signal value.

**Variant C:** Emit multi-signal VCD probes for wide packed structs and arrays,
reconstruct the full value in Python, and compare against a bit-vector
reference value.

---

## Residual boundaries from W538

- `./scripts/tri test --icarus-lowerable --cocotb --fast` is green:
  35 Icarus simulations passed, 0 failed; 35 cocotb reference-model checks
  passed, 0 failed; 0 seal mismatches.
- 24 pre-existing yosys smoke failures remain documented and unchanged.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, and
  `cargo test -p tri` are green.
- The VCD probe is currently fixed at 64 bits and skips wide/non-scalar
  assertions; the Python evaluator currently handles literals and simple
  constant expressions only.

---

*φ² + φ⁻² = 3 | TRINITY*
