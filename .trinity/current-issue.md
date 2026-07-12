# Wave Loop 502 — Harden Icarus lowerability gate with adversarial non-main witnesses

**Issue:** #1471  
**Branch:** `wave-loop-502`  
**Status:** closed  
**Variant:** B (defensive) — grow the Icarus-lowerable witness corpus with
adversarial non-`main` entry points and prove each witness with a
`native_decide` equivalence theorem.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Wave Loop 501 removed the hard-coded `main` assumption from the generic
structural-equivalence theorem.  Wave Loop 502 stressed the resulting boundary
by adding four hand-crafted adversarial witnesses that exercise non-`main`
functions under the classifier, the smoke gate, and the Lean soundness contract.

The wave also generalized the theorem one step further: it now accepts an
arbitrary list of argument values, so value preservation can be stated for
emitted functions that take parameters (e.g., a scalar-struct helper).

---

## What changed

- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - `evalVModuleTotal` now takes an explicit `args : List Value` parameter and
    forwards it to the emitted function, matching `evalModuleFunctionTotal`.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - `module_value_equiv_proved` is now parameterized by `args : List Value`.
  - `module_value_equiv_main` carries the same `args` parameter.
  - `evalVModuleTotal_bind` is updated accordingly.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - `module_value_equiv_statement` and `module_value_equiv_main_statement` now
    accept `args`.
  - Added W502 lowerability and value-equivalence theorems:
    - `w502_non_main_called_from_emitted_value_equiv` for `caller`,
    - `w502_non_main_chain_leaf_value_equiv` for `leaf`,
    - `w502_non_main_helper_struct_param_value_equiv` for `helper` with a
      scalar struct argument,
    - `w502_multiple_non_main_entries_a_value_equiv` / `_b_value_equiv` for `a`
      and `b`.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W502 witness environments and modules.

- `specs/scratch/w502_*.t27`
  - Four regression `.t27` specs covering the adversarial shapes.

- `.trinity/seals/scratch_w502_*.json`
  - Seals for the four new witness specs.

- `docs/reports/WAVE_LOOP_502_CLOSEOUT.md`
  - Close-out report.

- `docs/reports/FPGA_LOOP_COOPERATION_W503_2026-07-13.md`
  - Three W503 cooperation variants.

---

## Verification (final)

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0
  disagreements.
- `./scripts/tri test`:
  - 703 / 703 non-smoke PASS.
  - 183 / 183 yosys smoke PASS, 0 baseline failures.
  - 183 / 183 Icarus smoke PASS, 0 documented baseline failures.
  - 703 / 703 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
  - Gen C / Fixed Point: clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- Conditionals and loops remain outside the modeled operational semantics.
- Array-typed direct fields continue to use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

## Close-out artifacts

- `docs/reports/WAVE_LOOP_502_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W503_2026-07-13.md`

---

*φ² + φ⁻² = 3 | TRINITY*
