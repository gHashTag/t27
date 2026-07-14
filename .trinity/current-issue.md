# Wave Loop 535 — Align the Lean 4 lowerability predicate with the Rust structural classifier

**Issue:** #1506 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-535`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 534's structural Icarus lowerability boundary and
advance the recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W535_2026-07-07.md`.

**Variant A (recommended):**
- Tighten `Trinity.IcarusLowerable.Predicate.lean` to match the Rust structural
  classifier:
  - reject `while (true)` loops;
  - reject scalar structs whose fields are not lowerable (e.g. `f32`).
- Add `¬ Module.isLowerable env m` theorems in `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  for the W534 negative witnesses and discharge them with `native_decide`.
- Import a non-scratch corpus witness with a bounded `while` loop into
  `Completeness.lean` to show the tightened predicate still accepts the lowerable
  subset.
- Keep `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` at 0
  simulation failures and 0 seal mismatches.
- Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Variant B:** Add a cocotb reference-model cosimulation gate that compares the
simulated Verilog output against an independent Python reference model for a
subset of lowerable specs.

**Variant C:** Extend the Lean 4 formal semantics to cover module-level procedural
initialization and whole-struct assignment, with a non-scratch corpus witness in
`specs/igla/`.

---

## Residual boundaries from W534

- `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` is green:
  35 Icarus simulations passed, 0 failed, 0 seal mismatches.
- 24 pre-existing yosys smoke failures remain documented and unchanged.
- The Rust structural classifier is now the authoritative lowerability gate;
  the remaining gap is formal-model alignment in Lean 4.

---

*φ² + φ⁻² = 3 | TRINITY*
