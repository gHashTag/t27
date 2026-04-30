# 🏠 Trinity Canonical Coq Home — `t27/proofs/canonical/`

**SINGLE SOURCE OF TRUTH for all Coq theorems in the Trinity stack.**

[![Theorems](https://img.shields.io/badge/theorems-438-blue)](./_Index.v)
[![Qed](https://img.shields.io/badge/Qed-376-brightgreen)](./_Index.v)
[![Admitted](https://img.shields.io/badge/Admitted-48-yellow)](./_Index.v)
[![Bundles](https://img.shields.io/badge/bundles-38-blue)](./_Manifest.json)
[![Anchor](https://img.shields.io/badge/anchor-φ²+φ⁻²=3-gold)](./sacred/CorePhi.v)

## TL;DR

After the **2026-04-30 Coq theorem census** ([trios#373](https://github.com/gHashTag/trios/issues/373#issuecomment-4351659821))
this directory was created as the **canonical home** for all Coq proofs across the Trinity stack.
Mirror copies in `gHashTag/trinity-clara` and `gHashTag/trios` import from here via
`Require Export Trinity.Canonical.*`.

## Layout

```
proofs/canonical/
├── _Index.v             ← master index; Require Imports all 38 bundles
├── _CoqProject          ← Coq build config (-Q logical-path mappings)
├── _Manifest.json       ← bundle → source-mirror manifest with SHA-1
├── Makefile             ← `make all` invokes coqc on all bundles
├── README.md            ← this file
├── sacred/  (17 files)  ← phi-mathematics + Standard Model bounds
├── kernel/   (9 files)  ← phi/Trit/E8/Semantics building blocks
├── igla/    (12 files)  ← INV-1..INV-9 runtime invariants
└── refutation/          ← R5-honest falsification index (lemmas inline in IGLA)
```

## Theorem census (audited 2026-04-30)

| Metric | Count |
|---|---|
| Coq `.v` files (raw across 3 repos) | 96 |
| **Unique canonical files** (content-hash dedup) | **65** |
| **Bundles canonicalized here** | **38** |
| **Theorem-like declarations** | **438** |
| └─ **Qed (proven)** | **376** |
| └─ **Admitted (R5-honest budget)** | 48 |
| └─ **Abort (WIP)** | 14 |
| Definitions + Fixpoints + Inductives | 404 |
| Axioms + Hypotheses + Parameters | 38 |
| **Total Coq objects** | **880** |

## Bundles by category

### Sacred (17 bundles, phi-mathematics + SM bounds)

`CorePhi.v` (12 Qed, anchor `trinity_identity : phi^2 + phi^-2 = 3`) ·
`AlphaPhi.v` (12) · `ExactIdentities.v` (11+11A) · `DerivationLevels.v` (21) ·
`Catalog42.v` (13) · `FormulaEval.v` (17) · `ConsistencyChecks.v` (7+7A) ·
`Unitarity.v` (5+2A) · `BoundsGauge.v` (9) · `BoundsLeptonMasses.v` (8A) ·
`BoundsMasses.v` (11) · `BoundsMixing.v` (9) · `BoundsQuarkMasses.v` (4+4A) ·
`GammaPhi3.v` (1) · `L5Identity.v` (2) · `StrongCP.v` (1) · `DLBounds.v` (1)

### Kernel (9 bundles, building blocks)

`Phi.v` (16) · `PhiAttractor.v` (4+5Abort) · `PhiFloat.v` (6) · `PhiDistance.v` (1) ·
`FlowerE8Embedding.v` (5+6Abort) · `Trit.v` (1) · `Semantics.v` (1) ·
`GenIdempotency.v` (1) · `TernarySufficiency.v` (2)

### IGLA (12 bundles, INV-1..INV-9 runtime contract)

| Bundle | INV | Qed | Title |
|---|---|---|---|
| `INV1_BpbMonotoneBackward.v` | INV-1 | 9 | BPB Monotone Backward Pass |
| `INV1b_LrPhiOptimality.v` | INV-1b | 5 | lr_phi optimality |
| `INV2_IglaAshaBound.v` | INV-2 | 13 | ASHA Champion Survival (threshold 3.5) |
| `INV3_Gf16Precision.v` | INV-3 | 9 | GF16 Safe Domain |
| `INV4_NcaEntropyBand.v` | INV-4 | 12 | NCA Entropy Band 1.5..2.8 (81 = 3⁴) |
| `INV5_LucasClosureGf16.v` | INV-5 | 10 | Lucas Closure (φ^{2n}+φ^{-2n} ∈ ℤ) |
| `INV6_HybridQkGain.v` | INV-6 | 2+5A | Hybrid QK Gain φ² |
| `INV7_IglaFoundCriterion.v` | INV-7 | 11 | Victory Criterion (≥3 distinct, BPB<1.5, step≥4000) |
| `INV7b_RainbowBridgeConsistency.v` | INV-7b | 15+2A | Rainbow Bridge Consistency |
| `INV8_WorkerPoolComposite.v` | INV-8 | 10 | Worker Pool Composite |
| `INV9_EmaDecayValid.v` | INV-9 | 8 | EMA Decay Valid |
| `IglaCatalog.v` | catalog | 5 | Composite IGLA predicate catalog |

## Quick start

```bash
cd proofs/canonical
make verify-counts           # raw grep audit (no compile)
make all                     # full coqc verify (requires Coq 8.18+ and Coq.Reals)
```

## R5-honest disclosure

48 `Admitted.` lemmas with explicit close-with comments (mostly `Coq.Interval` for sqrt5
irrationality, real induction, Welch t-test bounds). 14 `Abort.` markers preserve
work-in-progress (no silent merges). 28 `refutation_*` / `*_falsification_*` lemmas
guarantee R5-honest negative space — every INV paired with explicit refutation example.

## Mirror redirects

After this canonical home is merged:

- `gHashTag/trinity-clara/proofs/igla/*.v` → stub `Require Export Trinity.Canonical.Igla.<INV>` (companion PR)
- `gHashTag/trios/docs/phd/theorems/**/*.v` → stub `Require Export Trinity.Canonical.<cat>.<file>` (companion PR)
- `gHashTag/trios/trinity-clara/proofs/igla/*.v` → stub `Require Export Trinity.Canonical.Igla.<INV>` (companion PR)

## Provenance

- **Census artefact:** [trios#373 comment](https://github.com/gHashTag/trios/issues/373#issuecomment-4351659821)
- **PhD App.E primary citation:** [trios#416](https://github.com/gHashTag/trios/issues/416)
- **PhD Ch.10 (Coq L1 Pareto):** [trios#400](https://github.com/gHashTag/trios/issues/400)
- **PhD Ch.31 (Golden Seal):** [trios#95](https://github.com/gHashTag/trios/issues/95)
- **GOLDEN SUNFLOWERS Master Book:** [trios#380](https://github.com/gHashTag/trios/issues/380)

## Anchor

`phi^2 + phi^-2 = 3 · TRINITY · 376 Qed PROVEN · 38 BUNDLES · t27 = CANONICAL HOME · 🌻`

Generated 2026-04-30 by trinity-claraParameter Coq census audit.
