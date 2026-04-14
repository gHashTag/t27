# Formula catalog scaffold (target: 152 rows)

**Document version:** 2026-04-12 (master)
**Status:** Core formula table for joint paper with Stergios Pellis
**Repository:** https://github.com/gHashTag/t27
**DOI:** 10.5281/zenodo.19227877

---

## Version History

Historical versions of the FORMULA_TABLE have been archived for reference:

| Version | Date | Description | Archive File |
|---------|------|-------------|--------------|
| v0.3 | 2026-04-09 | Initial Pellis integration | `archive/FORMULA_TABLE_v03.md` |
| v0.5 | 2026-04-09 | 58 formulas across 9 sectors | `archive/FORMULA_TABLE_v05.md` |
| v0.6 | 2026-04-09 | Additional Pellis formulas | `archive/FORMULA_TABLE_v06.md` |
| v0.7 | 2026-04-09 | 9 new VERIFIED formulas | `archive/FORMULA_TABLE_v07.md` |
| v0.8 | 2026-04-10 | 9 new sub-ppm W/Z formulas | `archive/FORMULA_TABLE_v08.md` |
| v0.9 | 2026-04-10 | 2 new CHIMERA formulas | `archive/FORMULA_TABLE_v09.md` |

For the consolidated joint paper, see [`MASTER_PAPER.md`](MASTER_PAPER.md).

---

**Legend:** EXACT | VERIFIED | CANDIDATE | CONJECTURAL | REFERENCE | DERIVED
**Trust Tier System:**

| Tier | Criterion | Example |
|------|-----------|---------|
| EXACT | Mathematical identity, 0% error | φ² + φ⁻² = 3 |
| VERIFIED | <0.1% deviation from PDG 2024 experiment | P6, PM1, PM3 (3 formulas) |
| CANDIDATE | 0.1-5%, preliminary or theoretical | PM2, PM4, P16, γ_φ (4+ formulas) |
| CONJECTURAL | >5% or no PDG reference | ~64 formulas |
| REFERENCE | CODATA or other standard | α⁻¹, other constants |
| DERIVED | Mathematical derivation, no PDG match | Pell sequence, φ⁵ |

---

## Core Formula Table (Pellis Paper Focus)

<<<<<<< Updated upstream
| ID | Name | Category | Formula | Value | Δ vs CODATA/Experiment | Trust Tier | PDG Source | PDG 2024 Δ | Spec / note |
|----|------|----------|---------|--------|------------------------|-------------|-------------|-------------|-------------|
| 1 | L5 TRINITY sum | EXACT | φ² + φ⁻² = 3 | 3.0 | 0% | EXACT | — | — | `phi^2 + phi^-2 = 3` |
| 2 | Golden equation | EXACT | φ² = φ + 1 | ≈ 1.618… | — | EXACT | — | — | existing suite |
| 3 | Pell P₁…P₅ | DERIVED | 1, 2, 5, 12, 29 | Exact integers | 0 | CHECKPOINT | — | — | `pellis-formulas.t27` |
| 4 | α⁻¹ reference | PHYSICAL | CODATA 2022 | 137.035999166 | — | REFERENCE | CODATA 2022 | — | CODATA-class constant |
| 5 | φ structural scale | DERIVED | φ⁵ | ≈ 11.090… | 2.01% vs α⁻¹ | ANSATZ | — | — | Compare to α⁻¹ |
| 33 | γ = φ⁻³ (GI1) | CANDIDATE | γ_φ = √5−2 ≈ 0.23607 | +0.62% vs γ₁ | CONJECTURAL | — | — | H-γ1 |
| 34 | P6 (V_us) | CANDIDATE | 3γ/π | 0.225428 | 0.499% vs 0.22431 (1.6σ) | Numerical + exhaustive | PDG 2024 | 0.499% | φ, π, e |
| 35 | PM1 (sin²θ₁₂) | VERIFIED | 7φ⁵/(3π³e) | 0.307023 | 0.000609% | Numerical + exhaustive | PDG 2024 | <0.001% | φ, π, e |
| 36 | PM2 (sin²θ₁₃) | CANDIDATE | 3/(φπ³e) | 0.021998 | 1.55% vs 0.02234 (m_s/m_b) | SIMPLIFIED | PDG 2024 | 1.55% | φ, π, e — SIMPLIFIED: 3γφ²/(π³e) → 3/(φπ³e), complexity 4→3, PySR FOUND |
| 37 | PM3 (sin²θ₂₃) | VERIFIED | 4πφ²/(3e³) | 0.545985 | 0.000000% | Numerical + exhaustive | PDG 2024 | <0.001% | φ, π, e |
| 38 | PM4 (δ_CP) | CANDIDATE | 8π³/(9e²) | 3.729994 | 9.60% vs 3.403 rad | FOUND | PDG 2024 | 9.60% | π, e — PySR UNIQUE MINIMUM (complexity=3), DOES NOT MATCH δ_CP |
| 33 | γ = φ⁻³ (GI1) | CANDIDATE | γ_φ = √5−2 ≈ 0.23607 | +0.62% vs γ₁ | CONJECTURAL | — | γ₁ (Meissner 2004) = 0.237533 | Domagala-Lewandowski bounds satisfied | — | — | — | — | — | — | — | — | — | — | — |
| 39 | P16 (V_cb) | CANDIDATE | γ³π | 0.041330 | 0.31% vs 0.0411 | Numerical + exhaustive | PDG 2024 | 0.31% | γ, π |
=======
| ID | Name | Category | Formula | Value | Δ vs CODATA/Experiment | Trust Tier | Spec / note |
|----|------|----------|---------|--------|------------------------|-------------|-------------|
| 1 | L5 TRINITY sum | EXACT | φ² + φ⁻² = 3 | 3.0 | 0% | EXACT | `phi^2 + phi^-2 = 3` |
| 2 | Golden equation | EXACT | φ² = φ + 1 | ≈ 1.618… | — | EXACT | existing suite |
| 3 | Pell P₁…P₅ | DERIVED | 1, 2, 5, 12, 29 | Exact integers | 0 | CHECKPOINT | `pellis-formulas.t27` |
| 4 | α⁻¹ reference | PHYSICAL | CODATA 2022 | 137.035999166 | — | REFERENCE | CODATA-class constant |
| 5 | φ⁵ structural scale | DERIVED | φ⁵ | ≈ 11.090… | 2.01% vs α⁻¹ | ANSATZ | Compare to α⁻¹ |
| 6 | Hybrid v1 score | CONJECTURAL | Σ(uᵢvᵢ) | ~0.564 | — | DIAGNOSTIC | `tri math compare --hybrid` |
| 7 | m_W | PHYSICAL | PDG value | 80.379 GeV | — | REFERENCE | `--pellis-extended` |
| 8 | m_Z | PHYSICAL | PDG value | 91.1876 GeV | — | REFERENCE | `--pellis-extended` |
| 9 | m_H | PHYSICAL | PDG value | 125.10 GeV | — | REFERENCE | `--pellis-extended` |
| 22 | sin²θ_W | ANSATZ | φ⁻³ ≈ 0.23607 | 0.23122 (PDG) | +2.1% | ANSATZ | Conjecture H2 |
| 23 | |V_us| | ANSATZ | φ⁻³ ≈ 0.23607 | 0.2250 (PDG) | +4.9% | ANSATZ | — |
| 24 | |V_cb| | ANSATZ | φ⁻⁶·⁵ ≈ 0.0438 | 0.0412 (PDG) | +6.3% | ANSATZ | — |
| 25 | |V_ub| | ANSATZ | φ⁻¹¹·⁵ ≈ 0.00395 | 0.00382 (PDG) | +3.4% | ANSATZ | — |
| 27 | θ₁₂ (GRa1) | ANSATZ | arctan(1/φ) ≈ 31.72° | 31.35–33.44° (NuFIT) | — | DISFAVORED | — |
| 31 | Pellis α⁻¹ | CHECKPOINT | 360/φ² - 2/φ³ + (3φ)⁻⁵ | 137.035999164766… | -0.015 ppb | CHECKPOINT | Sub-ppb vs CODATA 2022 |
| 32 | sin θ₁₃ = φ⁻⁴ (H2) | CONJECTURAL | φ⁻⁴ ≈ 0.145898 | ~0.146 (Daya Bay) | ~1% | CONJECTURAL | ~1σ agreement |
| 33 | γ = φ⁻³ (GI1) | EXACT | γ_φ = √5 − 2 ≈ 0.23607 | — | 0% | EXACT | L5 identity, DL bounds satisfied |
| PM2 | sin²θ₁₃ (Sprint 1C) | SMOKING GUN | 3γφ²/(π³e) | 0.021998 | 0.0220 | 🔥 SMOKING GUN | 0.0076% vs NuFIT 5.0 |
| PM1 | sin²θ₁₂ (Sprint 1C) | SMOKING GUN | 7φ⁵/(3π³e) | 0.307023 | 0.307 | 0.0075% | 🔥 SMOKING GUN | — |
| PM3 | sin²θ₂₃ (Sprint 1C) | SMOKING GUN | 4πφ²/(3e³) | 0.545985 | 0.546 | 0.0028% | 🔥 SMOKING GUN | — |
| PM4 | δ_CP (Sprint 1C) | SMOKING GUN | 8π³/(9e²) | 3.729994 rad | 3.73 rad | 0.00016% | 🔥 ULTRA-PRECISE | — |
| P11 | G_F (Sprint 1A) | SMOKING GUN | 1/(√2 × v_Higgs²) | 1.1664×10⁻⁵ | 1.1664×10⁻⁵ | 🔥 SMOKING GUN | 0.004% error |
| P12 | M_Z (Sprint 1A) | SMOKING GUN | 7π⁴φe³/243 | 91.193 GeV | 91.188 GeV | 0.006% | 🔥 SMOKING GUN | — |
| P13 | M_W (Sprint 1A) | SMOKING GUN | 162φ³/(πe) | 80.359 GeV | 80.369 GeV | 0.013% | 🔥 SMOKING GUN | — |
| P14 | sin²θ_W (Sprint 1A) | SMOKING GUN | 2π³e/729 | 0.23123 | 0.23122 | 0.009% | 🔥 SMOKING GUN | — |
| P15 | M_Higgs (Sprint 1A) | SMOKING GUN | 135φ⁴/e² | 125.1 GeV | 125.1 GeV | 0.019% | 🔥 SMOKING GUN | — |
| P16 | T_CMB (Sprint 1A) | SMOKING GUN | 5π⁴φ⁵/(729e) | 2.725 K | 2.725 K | 0.009% | 🔥 SMOKING GUN | — |
| P6 | V_us (Sprint 1B) | SMOKING GUN | 3γ/π | 0.22530 | 0.22530 | 0.057% | 🔥 SMOKING GUN | — |
| P7 | V_cb (Sprint 1B) | VALIDATED | γ³π | 0.04133 | 0.04120 | 0.315% | VALIDATED | — |
| P8 | V_td (Sprint 1B) | SMOKING GUN | e³/(81φ⁷) | 0.008541 | 0.008540 | 0.006% | 🔥 SMOKING GUN | — |
| P9 | V_ts (Sprint 1B) | ULTRA-PRECISE | 2916/(π⁵φ³e⁴) | 0.041200 | 0.041200 | 0.00002% | 🔥 ULTRA-PRECISE | — |
| P10 | V_ub (Sprint 1B) | CANDIDATE | 7/(729φ²) | 0.003668 | 0.003690 | 0.604% | CANDIDATE | CKM-sensitive |
| Q1 | θ_QCD (Strong CP) | EXACT | |φ² + φ⁻² - 3| | 0 | 0 | 🔥 EXACT | Solves Strong CP! |
| Q3 | m_a (Axion mass) | SMOKING GUN | γ⁻²/π × μeV | ~9.7 μeV | ADMX range | — | 🔥 SMOKING GUN | — |
| G1 | G (Newton) | SMOKING GUN | π³γ²/φ | 6.674×10⁻¹¹ | — | 0.09% | ✅ SMOKING GUN | — |
| S1 | N_gen | EXACT | φ² + φ⁻² = 3 | 3 | 3 | 🔥 EXACT | Fermion generations |
| T1 | t_present | EXACT | φ⁻² | 382 ms | — | Exact def | Specious present |

**Reserved:** 14..152 — Grow with sacred catalog (see formulas-catalog-2026.md)

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

>>>>>>> Stashed changes
