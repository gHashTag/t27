# NOW -- Existing H4Lagrangian bounds are proved (2026-10-04)

## Proof repair (Closes #5927)

- Closes #5927; proof-only child of #3142. The unchanged baseline fails two H4Lagrangian goals in Lean 4.31.0 with the existing pinned Mathlib.
- Prove the existing order-of-magnitude interval with verified bounds on exp(1)/pi. Prove the existing Koide relative-error bound with sqrt(239) >= 15 and sqrt(549) >= 23.
- Every formula, definition and theorem statement remains unchanged. The old one-percent comment is corrected: the actual proposition is relative error below one. This is a coarse consistency bound, not a one-percent result or a physical derivation.
- The complete H4Lagrangian file compiles with the exact pinned toolchain and unchanged dependencies; no sorry or custom axioms are added. The separate TernaryFPGABoot and whole-library CI failures remain visible; no hardware or complete inference claim.
