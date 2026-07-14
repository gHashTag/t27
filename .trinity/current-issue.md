# Wave Loop 534 — Harden the Icarus lowerability boundary

**Issue:** #1505 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-534`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 533's packed scalar-struct work and advance the
recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W534_2026-07-07.md`.

**Variant A (recommended):**
- Add adversarial negative witnesses for non-lowerable constructs:
  enum/string/float fields, unresolved imports, host-only helpers, casts to
  non-lowerable types, unbounded dynamic loops, and whole-struct assignment of
  non-lowerable structs at module scope.
- State `¬ Module.isLowerable env m` theorems in Lean 4 and discharge them with
  `native_decide` or directly from the classifier predicate.
- Add a Rust integration test that asserts the classifier rejects exactly the
  specs the Lean predicate rejects (and accepts every spec with a
  value-preservation theorem).
- Document the lowerability boundary in `docs/ICARUS_LOWERABLE_BOUNDARY.md` so
  future compiler changes cannot silently expand the subset.
- Keep `./scripts/tri test --icarus-simulate --icarus-lowerable` at 0 simulation
  failures and maintain the 24 documented yosys smoke baselines flat.

**Variant B:** Add cocotb reference-model cosimulation on top of the existing
Icarus gate to catch value-level semantic drift with an independent Python model.

**Variant C:** Extend the Lean 4 formal semantics to cover module-level procedural
initialization and whole-struct assignment, with a non-scratch corpus witness in
`specs/igla/`.

---

## Residual boundaries from W533

- `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` is green:
  36 Icarus simulations passed, 0 failed, 0 seal mismatches.
- 24 pre-existing yosys smoke failures remain documented and unchanged.
- Module-level packed scalar structs with fixed-size scalar array fields are now
  fully lowered; the remaining risk is the informal lowerability boundary.

---

*φ² + φ⁻² = 3 | TRINITY*
