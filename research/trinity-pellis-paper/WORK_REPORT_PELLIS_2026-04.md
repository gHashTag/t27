# Trinity × Pellis — Work Report April 2026

> **Scope:** Engineering and research work on the Pellis line (golden ratio φ, fine-structure constant, hybrid diagnostics, Standard Model comparison). Intended for collaborators and reviewers.

---

## Executive Summary

The Pellis program has moved from “beautiful coincidences” to a **verifiable catalog**: every claimed φ-formula in the repository either has a path to an executable check (`tri math compare`, `.t27` specs) or carries an explicit **CONJECTURAL** label with a numerical value, experimental reference, and deviation. **Three numerical errors** were found and corrected during audit (two CKM table entries + one NuFIT σ mishandling), making the picture simultaneously more honest and more defensible for peer review.

---

## Response to Collaborators / Reviewers

The main numerical asset remains the **Pellis closed form for α⁻¹**: a pre-registered 50-digit decimal seal (SSOT in [`specs/physics/pellis-formulas.t27`](../../specs/physics/pellis-formulas.t27) and high-precision scripts [`scripts/print_pellis_seal_decimal.py`](../../scripts/print_pellis_seal_decimal.py), [`scripts/verify_precision.py`](../../scripts/verify_precision.py)). It agrees with the CODATA 2022 recommended value for the **direct** α⁻¹ (central value ending **…166**), with a separate note about the **…177** string from the back-calculation of α — documented explicitly to avoid confusing reviewers.

The document audit found and corrected **three numerical errors** that were distorting conclusions:

1. **CKM ansätze |V_cb| and |V_ub|:** wrong magnitudes and percentages at fixed φ-exponents (values were swapped vs the stated powers).
2. **Neutrino θ₁₂ vs NuFIT:** incorrect σ scaling — with NuFIT 6.0–style **±0.75°** at 1σ, the gap **~1.69°** is **~2.3σ**, not ~1.2σ when a wider illustrative band (e.g. ±1.4°) is mixed in without citing the exact table column. See [NuFIT 6.0 (arXiv:2410.05380)](https://arxiv.org/abs/2410.05380).

After corrections the picture is **honest**: some ansätze look better, some harder than before the audit.

**New Conjecture H2** (**sin θ₁₃ = φ⁻⁴**): \(\varphi^{-4} \approx 0.145898\) → \(\arcsin(\varphi^{-4}) \approx 8.39^\circ\). Compared to a reactor-style central value ~8.54° with ±0.15° uncertainty this is ~1σ — **provided** the parametrization and error source are stated explicitly. The table must not conflate **sin θ₁₃** and **sin²2θ₁₃**.

---

## 1. GitHub integration timeline

| Artifact | Status | Summary |
|----------|--------|---------|
| [**PR #294**](https://github.com/gHashTag/t27/pull/294) | **Merged** | `.trinity` seals; specs (ternary, Pellis precision, GF competitive); [`benchmarks/language_tests/`](../../benchmarks/language_tests/); [`bootstrap/src/math_compare.rs`](../../bootstrap/src/math_compare.rs) + [`research/trinity-pellis-paper/`](./); merge to `master` with conflicts resolved. |
| [**PR #299**](https://github.com/gHashTag/t27/pull/299) | **Merged** | P0 Sprint 1: Zig → `.t27` — [`PackedTrit`](../../specs/ternary/packed_trit.t27), [`SacredConstants`](../../specs/sacred/constants.t27), [`HybridArithmetic`](../../specs/ternary/hybrid_arithmetic.t27); see §3. |
| [**PR #321**](https://github.com/gHashTag/t27/pull/321) | **Merged** | Removed nested `trinity/.git`; [`trinity/README.md`](../../trinity/README.md) tracked as normal files. |
| [**PR #325**](https://github.com/gHashTag/t27/pull/325) | **Merged** | This report, [`FORMULA_TABLE.md`](./FORMULA_TABLE.md) rows **31–32**, [`TECHNOLOGY_MAP.md`](./TECHNOLOGY_MAP.md), [`TRINITY_VS_SM_FORMULAS.md`](./TRINITY_VS_SM_FORMULAS.md) §13. |
| [**PR #328**](https://github.com/gHashTag/t27/pull/328) | **Merged** | Revert-merge **[#327](https://github.com/gHashTag/t27/pull/327)** (wrong files), re-apply **only** this report + `TECHNOLOGY_MAP.md`. |
| [**PR #297**](https://github.com/gHashTag/t27/pull/297) | **Open** | Whitepaper + benchmarks — **blocked:** merge conflicts + **`check-linked-issue`** until PR links a qualifying issue (GitHub **Checks** tab). |
| [**Issue #295**](https://github.com/gHashTag/t27/issues/295) | **Open** | `tri math compare --weinberg` — [`GH_ISSUE_WEINBERG_CLI_BODY.md`](./GH_ISSUE_WEINBERG_CLI_BODY.md). |
| [**Issue #296**](https://github.com/gHashTag/t27/issues/296) | **Open** | Hybrid v2 + golden **N = 5, 10, 15, 20, 50, 152** — [`GH_ISSUE_HYBRID_V2_BODY.md`](./GH_ISSUE_HYBRID_V2_BODY.md). |

*[**#302**](https://github.com/gHashTag/t27/issues/302) / [**#303**](https://github.com/gHashTag/t27/issues/303) **closed** as duplicates of **#295** / **#296**.*

*Confirm numbers in [GitHub](https://github.com/gHashTag/t27) before external citations.*

**Merge policy:** **#294**, **#299**, **#321**, **#325**, **#328** are on `master`. **#297** needs manual fix. No bulk-merge of unrelated Ring PRs.

---

## 2. Numerical Audit and Corrections

### 2.1 CKM matrix (documents)

- **|V_cb|** at ansatz **φ⁻⁶·⁵**: correct value ≈ **0.0438**, relative deviation from PDG-class ~**6%**, not the erroneous 0.0344 / 16.5%.
- **|V_ub|** at **φ⁻¹¹·⁵**: ≈ **0.00395**, ~**3.4%**, not 0.00218 / 43%.
- **φ⁻⁷ ≈ 0.0344** gives **worse** agreement with |V_cb| (~16%).

### 2.2 Neutrino / NuFIT (θ₁₂)

- Gap |33.41° − arctan(1/φ)| ≈ **1.69°** at 1σ = **±0.75°** → **~2.3σ**, not ~1.2σ.
- Do not mix interval width **±1.4°** without citing the exact NuFIT column — [arXiv:2410.05380](https://arxiv.org/abs/2410.05380).

### 2.3 CODATA / α⁻¹ (code)

- [`bootstrap/src/math_compare.rs`](../../bootstrap/src/math_compare.rs): `ALPHA_INV_REF = 137.035999166` (CODATA 2022, direct α⁻¹), replacing the outdated 2018-class **…084**.

### 2.4 Conjecture H2 (θ₁₃)

\[\sin\theta_{13} = \varphi^{-4}, \quad \varphi^{-4} \approx 0.14589803375\ldots\]

\[\arcsin(\varphi^{-4}) \approx 8.39^\circ\]

Compared to reactor central value ~8.54° with ±0.15° → **~1σ** (conjecture in the catalog; main long-horizon check: next-generation reactor / JUNO-era fits).

---

## 3. P0 Sprint 1 — New `.t27` specs ([PR #299](https://github.com/gHashTag/t27/pull/299), merged)

| Spec | File | Purpose | Lines (Apr 2026, `wc -l`) |
|------|------|---------|---------------------------|
| **PackedTrit** | [`specs/ternary/packed_trit.t27`](../../specs/ternary/packed_trit.t27) | 5-trit/byte encoding, arithmetic | 428 |
| **SacredConstants** | [`specs/sacred/constants.t27`](../../specs/sacred/constants.t27) | φ, π, e, Trinity identities | 384 |
| **HybridArithmetic** | [`specs/ternary/hybrid_arithmetic.t27`](../../specs/ternary/hybrid_arithmetic.t27) | Dual storage, SIMD-oriented layout | 490 |

**Verification (local, `master`):**

- **Parse:** `./scripts/tri parse <file.t27>` succeeds for all three paths above (bootstrap `t27c` may panic on duplicate CLI alias in some builds — use `./scripts/tri` as the portable check).
- **Tests:** each file defines **many** `test` blocks (21 / 29 / 24 respectively — counts may drift slightly with edits).
- **Imports:** modules use `use base::types`, `use base::ops`, `use numeric::gf16` as appropriate (see file headers).
- **L3 / ASCII:** English identifiers in these specs; follow project purity rules in CI.

Traceability: tie future edits to the **P0 sprint / PR #299** and active GitHub issues; add explicit `// issue #NNN` headers where ISSUE-GATE requires it.

### L5 identity — canonical invariant

The relation \(\varphi^2 + \varphi^{-2} = 3\) is enforced as a **machine-checkable** anchor in [`specs/sacred/constants.t27`](../../specs/sacred/constants.t27) (and row 1 in [`FORMULA_TABLE.md`](FORMULA_TABLE.md)), not merely as prose. This distinguishes the t27 approach from informal φ-literature where the same identity appears without a spec-linked test harness.

---

## 4. Formula catalog ([`FORMULA_TABLE.md`](FORMULA_TABLE.md))

**On `master` (this report commit):**

| Row | Formula | Status | Notes |
|-----|---------|--------|-------|
| **31** | 50-digit Pellis α⁻¹ seal | DERIVED / sealed | Pre-registered; scripts + [`pellis-formulas.t27`](../../specs/physics/pellis-formulas.t27) |
| **32** | sin θ₁₃ = φ⁻⁴ (H2) | CONJECTURAL | arcsin(φ⁻⁴) ≈ 8.39°; reactor ~8.54° ± 0.15° → ~1σ if convention matches; sin vs sin²2θ₁₃ |

The §13 index in [`TRINITY_VS_SM_FORMULAS.md`](TRINITY_VS_SM_FORMULAS.md) includes row **32**.

---

## 5. Supporting documentation

| File | Purpose |
|------|---------|
| [`TRINITY_VS_SM_FORMULAS.md`](TRINITY_VS_SM_FORMULAS.md) | Trinity/Pellis vs SM definitions; row index; CODATA 166 vs 177; trust tiers |
| [`hybrid-conjecture.md`](hybrid-conjecture.md) | Hybrid ansatz derivation and status |
| [`GMP_MPFR_ROADMAP.md`](GMP_MPFR_ROADMAP.md) | High-precision arithmetic expansion plan |
| [`competitors.md`](competitors.md) | Competitor / context sketch |
| [`TECHNOLOGY_MAP.md`](TECHNOLOGY_MAP.md) | In-repo vs external claims (product roadmap hygiene) |
| [`GH_ISSUE_WEINBERG_CLI_BODY.md`](GH_ISSUE_WEINBERG_CLI_BODY.md) | Template body for Weinberg CLI issues |
| [`GH_ISSUE_HYBRID_V2_BODY.md`](GH_ISSUE_HYBRID_V2_BODY.md) | Template body for hybrid v2 issues |

All under `research/trinity-pellis-paper/`.

---

## 6. Infrastructure changes

- [`bootstrap/src/math_compare.rs`](../../bootstrap/src/math_compare.rs): α⁻¹ reference → CODATA 2022 (**…166**).
- `compiler.rs.orig` removed; `*.orig` in [`.gitignore`](../../.gitignore).
- `trinity/` nested `.git` removed — [**PR #321**](https://github.com/gHashTag/t27/pull/321).

---

## 7. Open steps (recommendations)

1. **Resolve and merge [PR #297](https://github.com/gHashTag/t27/pull/297)** after fixing conflicts and satisfying **issue-gate** (link a qualifying issue in PR description).
2. **Implement and close** Weinberg CLI on **[#295](https://github.com/gHashTag/t27/issues/295)** (duplicate **#302** closed).
3. **Implement and close** hybrid v2 golden tests on **[#296](https://github.com/gHashTag/t27/issues/296)** (duplicate **#303** closed).
4. **Sprint / stash “seven files”** — land as a separate commit when the sprint branch is ready (do not mix with whitepaper PR until conflicts are resolved).
5. **GMP/MPFR** + `verify_precision.py` expansion — separate PRs, lower priority.
6. **JUNO-era / reactor fits:** monitor as the long-horizon experimental discriminator for **H2**.

---

## 8. How to cite this report

```text
File:       research/trinity-pellis-paper/WORK_REPORT_PELLIS_2026-04.md
Repository: https://github.com/gHashTag/t27
```

*Re-verify all experimental numbers against primary sources before publication.*
