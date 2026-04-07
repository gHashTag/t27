# Formula catalog scaffold (target: 152 rows)

Legend: **EXACT** | **PHYSICAL** | **DERIVED** | **CONJECTURAL** — aligned with `specs/physics/sacred_verification.t27`.

The table is intentionally sparse at scaffold time; fill rows as each identity is sealed in `specs/` and referenced from `tri math compare`.

**Rule:** do not label any row as confirming **Conjecture H1** until the hybrid **v2** map is implemented, golden-tested, and reproducible via `tri` (see `hybrid-conjecture.md` and [#287](https://github.com/gHashTag/t27/issues/287)).

| ID | Name | Category | Spec / note | Status |
|----|------|----------|-------------|--------|
| 1 | L5 TRINITY sum | EXACT | `phi^2 + phi^-2 = 3` | wired |
| 2 | L5 phi quadratic | EXACT | `phi^2 = phi + 1` | existing suite |
| 3 | Pell P_1..P_5 block | DERIVED | `pellis-formulas.t27` | wired |
| 4 | alpha^-1 reference | PHYSICAL | CODATA-class constant in spec / CLI | reference only |
| 5 | phi^5 structural scale | DERIVED | Compare to alpha^-1 in CLI | diagnostic |
| 6 | Hybrid v1 H_5^(v1) | DERIVED | L1+max Pell map; `hybrid-conjecture.md` Hybrid v1; `tri math compare --hybrid` | diagnostic — **not** H1 |
| 7 | m_W | PHYSICAL | PDG GeV in CLI `--pellis-extended` | reference |
| 8 | m_Z | PHYSICAL | PDG GeV in CLI `--pellis-extended` | reference |
| 9 | m_H | PHYSICAL | PDG GeV in CLI `--pellis-extended` | reference |
| 10 | m_nu1/m_nu2 placeholder | CONJECTURAL | CLI placeholder only; not PDG masses | illustrative — **not** H1 |
| 11 | \|V_us\| | PHYSICAL | CKM modulus; CLI `--pellis-extended` | reference |
| 12 | \|V_cb\| | PHYSICAL | CKM modulus | reference |
| 13 | \|V_ub\| | PHYSICAL | CKM modulus | reference |
| 14 | Delta m^2_21 (nu) | PHYSICAL | PDG oscillation eV^2; table row for paper — not wired to `tri` yet | reference — **not** H1 |
| 15 | Delta m^2_31 or 32 (nu) | PHYSICAL | PDG / ordering convention TBD | reference — **not** H1 |
| 16 | \|V_ud\| | PHYSICAL | CKM unitarity row | reference |
| 17 | \|V_cs\| | PHYSICAL | CKM | reference |
| 18 | \|V_tb\| | PHYSICAL | CKM; ~1 | reference |
| 19 | delta_CP (CKM) | PHYSICAL | PDG phase; if used in Trinity map, cite convention | reference |
| 20 | Hybrid v2 H_N^(v2) | CONJECTURAL | L2 cosine map; [#287](https://github.com/gHashTag/t27/issues/287); **not in CLI** | planned — **no** H1 text until goldens pass |
| 21 | theta_N (hybrid v2) | CONJECTURAL | degrees from H_N^(v2); same gate as row 20 | planned |
| 22..152 | *Reserved* | — | Grow with `sacred_verification.t27` / catalog | TBD |

## Next steps

1. **SSOT for 152 rows (this repo):** derive rows from `specs/physics/sacred_verification.t27` and linked conformance/docs — there is **no** `src/particle_physics/formulas.zig` in t27. When a single JSON catalog for all 152 IDs exists, generate or sync table rows from that file under `tri` (no Python on the verification critical path per AGENTS).
2. Mirror each **EXACT** row with a `test` / `invariant` in the owning `.t27` file.
3. Add columns **Pellis equivalent** (if known) and **delta_ppm** vs experiment once definitions are frozen.
4. Use `tri math compare --sensitivity` to track numeric stability of the hybrid proxy under phi perturbations.

## Outreach snippet (Pellis / collaborators)

After merge to `master`:

```text
PR #280 is merged (#277 closed). Repro on a clean checkout:

  ./scripts/tri math compare --pellis --hybrid --sensitivity

P1..P5 = {1,2,5,12,29} are in specs/physics/pellis-formulas.t27.
Hybrid v1 scalar H_5^(v1) ~ 0.5638 — see explicit formula in hybrid-conjecture.md (not L2 cosine).
Hybrid v2 (~0.9617 / theta ~15.9 deg) is **not** outreach-safe until issue #287 is implemented and CI-green.
```
