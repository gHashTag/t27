# Wave Loop 505 Close-Out Report

**Issue:** #1474 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-505`  
**Variant:** A — adversarial sequential witnesses  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 505 executes **Variant A** from the W505 cooperation plan: five adversarial scratch witnesses that stress the sequential `ifThenElse` / `forLoop` boundary introduced by W503 and generalized by W504. Every witness is classified lowerable, survives yosys and Icarus Verilog simulation, has a saved seal, and is backed by a Lean 4 lowerability/sequentiality/value-preservation theorem. At least one witness (`w505_nested_if`) is proved directly by the generic sequential theorem `module_value_equiv_proved_sequential`; the remaining witnesses reuse the same generic theorem, demonstrating that the sequential equivalence boundary now holds for nested conditionals, conditional updates inside loops, parameter-bound loops, return-value loops, and loops with local body variables.

The Icarus smoke gate stays at **0 documented baseline failures**.

---

## Weak-point analysis

- **Nested `ifThenElse` was not explicitly covered.** W503 added `ifThenElse` semantics and proved a single conditional-return witness, but the generic sequential theorem had not been applied to a deeply nested cascade.
- **Conditional updates inside `forLoop` were not exercised.** A loop body containing an `ifThenElse` that mutates a local variable is a different stress point than a conditional return.
- **Parameter-bound and return-value loops needed separate coverage.** `w504_for_sum` covered `0..n` accumulation; W505 adds loops whose range is a binary expression (`n + 1`) and loops whose result is returned directly.
- **Local variable declarations inside loop bodies were unproven.** A `varDecl` inside a `forLoop` body must satisfy the sequential predicate and preserve valuation equivalence.

---

## Scientific / engineering anchors

- **CompCert Clight / Cminor** — fuel-based big-step loop semantics and forward simulation; the W505 witnesses map the same structural induction pattern to nested `if`/`for` control flow. ([Leroy et al., *CompCert*](https://compcert.org/))
- **Csmith / YARPGen** — adversarial compiler-fuzzing methodology: small hand-written programs that target a single boundary, static UB avoidance, and oracle-based value comparison. ([Yang et al., PLDI 2011](https://doi.org/10.1145/1993316.1993532))
- **Icarus Verilog / SystemVerilog LRM** — procedural `if` and `for` semantics; the shallow Verilog model in `IcarusLowerable` keeps unrolled behavior bit-exact.
- **Kami / Bluespec** — Coq-embedded HDL trace refinement; the `all_equiv` invariant in `Equivalence.lean` plays the same forward-simulation role for the t27 → Verilog lowering.

---

## What changed

### t27 specs and seals

- `specs/scratch/w505_nested_if.t27` — nested `if` with four return arms (`classify`).
- `specs/scratch/w505_if_in_for.t27` — conditional accumulation inside a bounded `for` (`conditional_sum`).
- `specs/scratch/w505_for_var_range.t27` — bounded `for` whose range is a parameter (`sum_range`).
- `specs/scratch/w505_for_return.t27` — return value computed by a bounded `for` (`factorial`).
- `specs/scratch/w505_for_local_var_init.t27` — local variable declared and used inside the loop body (`fill_init`).
- `.trinity/seals/scratch_w505_*.json` — deterministic seals for all five specs.

### Lean 4 model

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W505 witness environments and modules:
    - `w505NestedIfEnv` / `w505NestedIfModule` / `w505NestedIfClassify`
    - `w505IfInForEnv` / `w505IfInForModule` / `w505IfInForConditionalSum`
    - `w505ForVarRangeEnv` / `w505ForVarRangeModule` / `w505ForVarRangeSumRange`
    - `w505ForReturnEnv` / `w505ForReturnModule` / `w505ForReturnFactorial`
    - `w505ForLocalVarInitEnv` / `w505ForLocalVarInitModule` / `w505ForLocalVarInitFillInit`

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Added lowerability theorems:
    `w505_nested_if_lowerable`, `w505_if_in_for_lowerable`, `w505_for_var_range_lowerable`, `w505_for_return_lowerable`, `w505_for_local_var_init_lowerable`.
  - Added sequentiality theorems:
    `w505_nested_if_sequential`, `w505_if_in_for_sequential`, `w505_for_var_range_sequential`, `w505_for_return_sequential`, `w505_for_local_var_init_sequential`.
  - Added value-preservation theorems for representative inputs, each applying
    `module_value_equiv_proved_sequential`:
    - `w505_nested_if_value_equiv` — `classify(9)`
    - `w505_if_in_for_value_equiv` — `conditional_sum(3, 2, 5)`
    - `w505_for_var_range_value_equiv` — `sum_range(5)`
    - `w505_for_return_value_equiv` — `factorial(5)`
    - `w505_for_local_var_init_value_equiv` — `fill_init(4)`

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0 disagreements.
- `./scripts/tri test`:
  - 711 / 711 non-smoke PASS
  - 191 / 191 yosys smoke PASS, 0 baseline failures
  - 191 / 191 Icarus smoke PASS, 0 documented baseline failures
  - 711 / 711 seal matches
  - FPGA board-less smoke gate / replay: OK
  - Standalone lake-package build: OK
  - Gen C / Fixed Point: clean
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- `while` and `switch` remain outside the modeled operational semantics (targeted by W506 Variants B/C).
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

## Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W506_2026-07-07.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
