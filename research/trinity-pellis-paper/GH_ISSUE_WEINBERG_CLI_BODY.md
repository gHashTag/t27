## Summary

Add `tri math compare --weinberg` (or equivalent `t27c` flag) to evaluate **row 22**: φ⁻³ vs PDG **sin²θ_W** with an explicit tolerance and JSONL fields for reproducibility.

## Motivation

- `FORMULA_TABLE.md` row **22** and `TRINITY_VS_SM_FORMULAS.md` §4 document the conjectural ansatz **sin²θ_W ≈ φ⁻³** (~**0.236068** vs PDG central ~**0.23122**, **~2.1%**).
- Falsifiability path: **P2@MESA** / **DUNE ND** (see `FORMULA_TABLE.md` Mechanism note).
- Today there is no dedicated CLI path; golden test would lock **Δ** under a declared bound (e.g. **< 0.005** on |φ⁻³ − sin²θ_W| only after PDG reference + scheme are frozen in spec).

## Acceptance criteria

1. CLI flag documented next to `--hybrid-v2` / `--pellis-extended` in `bootstrap/src/math_compare.rs` (or factored module).
2. Reference **sin²θ_W** from a **single** documented source (PDG / preset constant in `pellis-formulas.t27` or sibling spec) — no silent hard-coded magic without spec pointer.
3. Print φ⁻³, reference sin²θ_W, absolute and relative Δ; append JSONL keys e.g. `weinberg_enabled`, `phi_inv_cubed`, `sin2_theta_w_ref`, `delta_abs`, `delta_rel`.
4. **`#[cfg(test)]`**: golden test |φ⁻³ − sin²θ_W_ref| < **0.005** (adjust if scheme requires).
5. English docs: one paragraph in `FORMULA_TABLE.md` row 22 status or `hybrid-conjecture.md` pointing to the flag.

## Non-goals

- Claiming H1 or electroweak “proof”; this is a **diagnostic / conjecture** hook only.
- Replacing PDG MS-bar running with a full renormalization story (document scheme in spec).

## References

- `FORMULA_TABLE.md` row 22, Falsifiability marker (P2@MESA, DUNE).
- `TRINITY_VS_SM_FORMULAS.md` §4.
