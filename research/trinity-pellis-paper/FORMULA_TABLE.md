# Formula catalog scaffold (target: 152 rows)

Legend: **EXACT** | **PHYSICAL** | **DERIVED** | **CONJECTURAL** — aligned with `specs/physics/sacred_verification.t27`.

The table is intentionally sparse at scaffold time; fill rows as each identity is sealed in `specs/` and referenced from `tri math compare`.

| ID | Name | Category | Spec / note | Status |
|----|------|----------|-------------|--------|
| 1 | L5 TRINITY sum | EXACT | `phi^2 + phi^-2 = 3` | wired |
| 2 | L5 phi quadratic | EXACT | `phi^2 = phi + 1` | existing suite |
| 3 | Pell P_1..P_5 block | DERIVED | `pellis-formulas.t27` | wired |
| 4 | alpha^-1 reference | PHYSICAL | CODATA-class constant in spec / CLI | reference only |
| 5 | phi^5 structural scale | DERIVED | Compare to alpha^-1 in CLI | diagnostic |
| 6 | Hybrid inner product | CONJECTURAL | `tri math compare --hybrid` | diagnostic |
| 7 | m_W | PHYSICAL | PDG GeV in CLI `--pellis-extended` | reference |
| 8 | m_Z | PHYSICAL | PDG GeV in CLI `--pellis-extended` | reference |
| 9 | m_H | PHYSICAL | PDG GeV in CLI `--pellis-extended` | reference |
| 10 | m_nu1/m_nu2 placeholder | CONJECTURAL | normal-hierarchy placeholder | illustrative |
| 11 | \|V_us\| | PHYSICAL | CKM modulus | reference |
| 12 | \|V_cb\| | PHYSICAL | CKM modulus | reference |
| 13 | \|V_ub\| | PHYSICAL | CKM modulus | reference |
| 14..152 | *Reserved* | — | Grow with sacred catalog | TBD |

## Next steps

1. Import row metadata from the sacred formula JSON when it lands in-repo.
2. **SSOT for 152 rows (this repo):** derive rows from `specs/physics/sacred_verification.t27` and linked conformance/docs — there is **no** `src/particle_physics/formulas.zig` in t27. When a single JSON catalog for all 152 IDs exists, generate or sync table rows from that file under `tri` (no Python on the verification critical path per AGENTS).
3. Mirror each **EXACT** row with a `test` / `invariant` in the owning `.t27` file.
4. Add columns **Pellis equivalent** (if known) and **delta_ppm** vs experiment once definitions are frozen.
5. Use `tri math compare --sensitivity` to track numeric stability of the hybrid proxy under phi perturbations.

## Outreach snippet (Pellis / collaborators)

After merge to `master`:

```text
PR #280 is merged (#277 closed). Repro on a clean checkout:

  ./scripts/tri math compare --pellis --hybrid --sensitivity

P1..P5 = {1,2,5,12,29} are in specs/physics/pellis-formulas.t27.
Current hybrid inner product (diagnostic v1) ~ 0.5638 — first joint numeric handle;
see research/trinity-pellis-paper/hybrid-conjecture.md for Conjecture H1 and limits.
```

