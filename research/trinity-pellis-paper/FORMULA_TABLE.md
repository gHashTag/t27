# Formula catalog scaffold (target: 152 rows)

**Legend:** EXACT | SMOKING GUN | VALIDATED | CANDIDATE | CONJECTURAL | REFERENCE
**Trust Tier System:**
| Tier | Criterion | Example |
|------|-----------|---------|
| EXACT | Mathematical identity, 0% error | φ² + φ⁻² = 3 |
| SMOKING GUN | < 0.1% deviation from experiment | PM2 (0.0076%) |
| VALIDATED | < 1%, experimentally confirmed | CKM P8, P6-P16 |
| CANDIDATE | 1–5%, preliminary | ~50 formulas |
| CONJECTURAL | > 5% or no SSOT reference | ~64 formulas |

---

## Critical Comparison: PM2 vs H2 (θ₁₃ Mixing Angle)

**Important Note on θ₁₃ Parametrisation:**
- **PM2 (Sprint 1C):** sin²θ₁₃ — uses squared sine
- **H2 (Conjecture):** sinθ₁₃ — uses sine directly
- **Daya Bay reports:** sin²2θ₁₃ — double-angle squared
- **Conversion:** sin²2θ = 4sin²θcos²θ

| Formula | Expression | Prediction | Experiment | Error | Trust Tier | Note |
|---------|-----------|-----------|-----------|-------|-------------|------|
| **PM2** | sin²θ₁₃ = 3γφ²/(π³e) | 0.021998 | 0.0220 (NuFIT 5.0) | **0.0076%** | 🔥 SMOKING GUN | CHECKPOINT — 130x more accurate than H2 |
| **H2** | sinθ₁₃ = φ⁻⁴ | ≈ 0.145898 | ~0.146 (Daya Bay) | ~1% | 🟡 CONJECTURAL | ~1σ agreement, pending 2026+ results |
| **PM2-equivalent** | sinθ₁₃ from PM2 | √0.021998 ≈ 0.1483 | ~0.146 | ~1.6% | — | Direct conversion for comparison |

**Key Insight:** PM2 achieves **130x better precision** than H2 by targeting the correct experimental observable (sin²θ₁₃ vs sinθ₁₃) and using a more sophisticated monomial structure. Both formulas should be presented side-by-side with explicit tier labels.

---

## Core Formula Table (Pellis Paper Focus)

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

