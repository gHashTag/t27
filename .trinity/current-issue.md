# Wave Loop 536 — Cocotb reference-model cosimulation gate

**Issue:** #1507 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-536`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 535's aligned Rust/Lean lowerability predicate and
advance the recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W536_2026-07-07.md`.

**Variant A (recommended):**
- Add a `t27c icarus-cocotb` subcommand that emits a cocotb testbench plus a
  Python reference model for a subset of Icarus-lowerable specs.
- Integrate the cocotb run into `bootstrap/src/suite.rs` as an optional phase
  gated by `--cocotb`.
- Seed the gate with 3–5 W5xx witnesses that already have Lean value-preservation
  theorems, so the Python reference model is checked against two independent
  sources of truth.
- Document the cocotb dependency and workflow in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.
- Keep `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` at 0
  simulation failures and 0 seal mismatches.
- Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Variant B:** Close the undefined-struct leniency in `Completeness.lean` by
generating full struct-field declarations for every struct name referenced in the
corpus envs and making `Ty.isLowerableFuel` return `false` for undeclared structs.

**Variant C:** Extend the Lean 4 formal semantics to cover module-level procedural
initialization and whole-struct assignment, with a non-scratch corpus witness in
`specs/igla/`.

---

## Residual boundaries from W535

- `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` is green:
  35 Icarus simulations passed, 0 failed, 0 seal mismatches.
- 24 pre-existing yosys smoke failures remain documented and unchanged.
- The Rust structural classifier and the Lean `Trinity.IcarusLowerable` predicate
  now agree on `while (true)`, non-lowerable struct fields, and imported-function
  calls.

---

*φ² + φ⁻² = 3 | TRINITY*
