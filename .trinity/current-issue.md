# Wave Loop 538 — Extend cocotb reference model with independent expression evaluator + VCD comparison

**Issue:** #1509 (placeholder — to create when GitHub token is available)
**Branch:** `wave-loop-538`
**Status:** planned
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 537's Rust/Lean lowerability alignment and advance the
recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W538_2026-07-07.md`.

**Variant A (recommended):**
- Extend `scripts/cocotb_ref_model.py` with a recursive interpreter for the
  Icarus-lowerable expression subset (literals, arithmetic, function calls,
  scalar array/struct indexing).
- Drive the generated Verilog as a DUT from cocotb, force inputs and clock, and
  capture a VCD trace.
- Compare the VCD trace against the independently computed reference values.
- Seed with W5xx witnesses that already have Lean value-preservation theorems.
- Keep `./scripts/tri test --icarus-lowerable --cocotb --fast` at 0 cocotb
  failures and 0 seal mismatches.
- Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Variant B:** Extend the Lean 4 formal semantics to cover module-level
procedural initialization and whole-struct assignment, with a non-scratch
corpus witness in `specs/igla/`.

**Variant C:** Add module-level packed-struct assignment from function calls
and struct literals, and prove lowerability/value-preservation for the new
pattern.

---

## Residual boundaries from W537

- `./scripts/tri test --icarus-lowerable --cocotb --fast` is green:
  35 Icarus simulations passed, 0 failed; 35 cocotb reference-model checks
  passed, 0 failed; 0 seal mismatches.
- 24 pre-existing yosys smoke failures remain documented and unchanged.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, and
  `cargo test -p tri` are green.
- W537 closed the undefined-struct leniency in `Trinity.IcarusLowerable.Predicate`
  and repaired all 249 corpus envs in `Completeness.lean`; Rust and Lean
  verdicts now agree.

---

*φ² + φ⁻² = 3 | TRINITY*
