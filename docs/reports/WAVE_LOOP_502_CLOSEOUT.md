# Wave Loop 502 Close-Out Report

**Issue:** #1471  
**Branch:** `wave-loop-502`  
**Variant:** B (defensive hardening)  
**Date:** 2026-07-13  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 501 generalized the Icarus structural-equivalence theorem to any
emitted (non-host-only) function name, removing the hard-coded `main` assumption.
Wave Loop 502 hardened that generalization by adding four adversarial scratch
witnesses that exercise non-`main` entry points under the classifier, the Icarus
smoke gate, and the Lean soundness contract.  As a side effect, the theorem was
generalized once more: it now accepts an arbitrary list of argument values, so
value preservation can be proved for emitted functions that take parameters.

The Icarus smoke gate remains at **0 documented baseline failures**.

---

## Weak-point analysis

- **Thin witness coverage for non-`main` entry points.**  After W501 only one
  helper (`get_y`) had been proved with the generalized theorem.
- **Classifier/smoke boundary strength.**  The 294 intentionally skipped specs
  sit next to the 253 lowerable specs; future emitter changes could silently move
  a spec across the boundary.
- **No parameterized-function coverage.**  The W501 theorem still evaluated every
  entry point with empty arguments, leaving helpers that take parameters outside
  the generic contract.

---

## Scientific / engineering anchors

- **Csmith / YARPGen / CsmithEdge** — randomized compiler fuzzing shows that
  witness coverage is the best defense against silent wrong-code regressions.
  t27 applies the same idea with hand-crafted adversarial witnesses on the
  lowerability boundary.
  ([Yang et al., PLDI 2011](https://doi.org/10.1145/1993316.1993532);
  [Regehr et al., OOPSLA 2020](https://users.cs.utah.edu/~regehr/yarpgen-oopsla20.pdf);
  [Sun et al., EMSE 2022](https://doi.org/10.1007/s10664-022-10146-1))
- **Icarus Verilog packed-vector workaround** — the smoke gate stays green because
  the emitter keeps values in packed arrays / packed structs.
  ([steveicarus/iverilog#1134](https://github.com/steveicarus/iverilog/issues/1134),
  [steveicarus/iverilog#266](https://github.com/steveicarus/iverilog/issues/266))
- **CompCert `Unusedglobproof`** — entry-point-independent semantic preservation
  is the model for W499/W501's unconditional function emission.
  ([CompCert Unusedglobproof](https://compcert.org/doc/html/compcert.backend.Unusedglobproof.html))

---

## What changed

### Lean 4 model

- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - `evalVModuleTotal` now takes `args : List Value`.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - `module_value_equiv_proved` and `module_value_equiv_main` take `args`.
  - `evalVModuleTotal_bind` updated accordingly.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - `module_value_equiv_statement` / `module_value_equiv_main_statement` take
    `args`.
  - Added W502 lowerability and value-equivalence theorems for four non-`main`
    functions:
    - `caller` calling `helper`,
    - `leaf` at the end of a three-function chain,
    - `helper` taking a scalar `Pt` struct parameter,
    - `a` and `b` in a module with multiple non-`main` entries.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W502 witness environments and modules.

### t27 specs and seals

- `specs/scratch/w502_non_main_called_from_emitted.t27`
- `specs/scratch/w502_non_main_chain_leaf.t27`
- `specs/scratch/w502_non_main_helper_struct_param.t27`
- `specs/scratch/w502_multiple_non_main_entries.t27`
- `.trinity/seals/scratch_w502_*.json` (4 new seals)

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0
  disagreements.
- `./scripts/tri test`:
  - 703 / 703 non-smoke PASS
  - 183 / 183 yosys smoke PASS, 0 baseline failures
  - 183 / 183 Icarus smoke PASS, 0 documented baseline failures
  - 703 / 703 seal matches
  - FPGA board-less smoke gate / replay: OK
  - Standalone lake-package build: OK
  - Gen C / Fixed Point: clean
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- Conditionals and loops remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

## Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W503_2026-07-13.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
