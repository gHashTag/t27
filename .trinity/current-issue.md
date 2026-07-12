# Wave Loop 501 — Generalize `module_value_equiv` beyond `main`

**Issue:** #1470  
**Branch:** `wave-loop-501`  
**Status:** closed  
**Variant:** A (scoped) — remove the hard-coded `main` entry-point assumption
from the generic Icarus structural-equivalence theorem, so value preservation
holds for any emitted (non-host-only) function in a lowerable combinational
module.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Wave Loop 500 closed the last documented Icarus baseline and left the generic
structural-equivalence theorem with one remaining entry-point assumption:
`module_value_equiv_statement` was hard-coded to the function named `"main"` and
required `main` to be non-host-only.  Wave Loop 501 removed that restriction by
parameterizing the theorem over any emitted function name.

The change makes the theorem usable for generated host code or test harnesses
that call module helpers directly, without forcing every verification goal to be
wrapped in a `main` function.

---

## What changed

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - `module_value_equiv_proved` is now parameterized by `fnName : String` and
    `fn : Function` instead of hard-coding `"main"`.
  - The proof derives lookup of the emitted `VFunction` for `fnName` and
    applies the fuel/AST forward-simulation invariant to `fn.body`.
  - Added `module_value_equiv_main` as a convenience corollary.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - `module_value_equiv_statement` is now the generalized theorem.
  - `module_value_equiv_main_statement` is the `main` corollary.
  - Added `w501_non_main_entry_lowerable` and
    `w501_non_main_entry_value_equiv`, applying the generalized theorem to the
    non-`main` function `get_y`.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added the W501 witness environment/module: `w501NonMainEnv`,
    `w501NonMainModule`, `w501NonMainMakePt`, `w501NonMainGetY`,
    `w501NonMainMain`.

- `specs/scratch/w501_non_main_entry_function.t27`
  - Regression spec matching the Lean witness; test block checks both `get_y()`
    and `main()`.

- `.trinity/seals/scratch_w501_non_main_entry_function.json`
  - Seal for the new witness.

---

## Verification (final)

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 254 lowerable specs.
- `./scripts/tri test`:
  - 699 / 699 non-smoke PASS.
  - 179 / 179 yosys smoke PASS, 0 baseline failures.
  - 179 / 179 Icarus smoke PASS, 0 documented baseline failures.
  - 699 / 699 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
  - Gen C / Fixed Point: clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- Conditionals and loops remain outside the modeled operational semantics.
- The theorem still requires the chosen function to be emitted (non-host-only),
  which is exactly the `Module.emittedFunctions` contract.
- Array-typed direct fields continue to use memory-mode lowering.

---

## Close-out artifacts

- `docs/reports/WAVE_LOOP_501_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W502_2026-07-13.md`

---

*φ² + φ⁻² = 3 | TRINITY*
