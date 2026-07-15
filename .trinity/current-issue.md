# Wave Loop 537 — Close undefined-struct leniency in `Completeness.lean`

**Issue:** #1508 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-537`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 536's cocotb reference-model gate and advance the
recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W537_2026-07-07.md`.

**Variant A (recommended):**
- Close the undefined-struct leniency in `Completeness.lean` by generating full
  struct-field declarations for every struct name referenced in the corpus envs.
- Change `Ty.isLowerableFuel` for `.struct name` to return `false` when the
  struct is not declared in the environment, matching the Rust structural
  classifier.
- Repair any corpus envs that currently rely on the lenient behavior by adding
  the missing struct declarations.
- Add a regression test that asserts the Rust classifier and the Lean predicate
  agree on every corpus spec.
- Keep `./scripts/tri test --icarus-lowerable --cocotb --fast` at 0 cocotb
  failures and 0 seal mismatches.
- Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Variant B:** Extend `scripts/cocotb_ref_model.py` with a recursive interpreter
for the lowerable expression subset, drive the generated Verilog as a DUT from
cocotb, capture a VCD trace, and compare signal values against the independently
computed reference values.

**Variant C:** Extend the Lean 4 formal semantics to cover module-level
procedural initialization and whole-struct assignment, with a non-scratch corpus
witness in `specs/igla/`.

---

## Residual boundaries from W536

- `./scripts/tri test --icarus-lowerable --cocotb --fast` is green:
  35 Icarus simulations passed, 0 failed; 35 cocotb reference-model checks
  passed, 0 failed; 0 seal mismatches.
- 24 pre-existing yosys smoke failures remain documented and unchanged.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, and
  `cargo test -p tri` are green.

---

*φ² + φ⁻² = 3 | TRINITY*
