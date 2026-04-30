(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   Trinity Coq — Canonical Index (Single Source of Truth)
   Anchor: phi^2 + phi^-2 = 3
   38 bundles · 438 theorem-like declarations · 376 Qed (audited 2026-04-30)
   Census: github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

(* --- Sacred core (phi-mathematics + Standard Model bounds) --- *)
From Trinity.Canonical.Sacred Require Import CorePhi.
From Trinity.Canonical.Sacred Require Import AlphaPhi.
From Trinity.Canonical.Sacred Require Import ExactIdentities.
From Trinity.Canonical.Sacred Require Import DerivationLevels.
From Trinity.Canonical.Sacred Require Import Catalog42.
From Trinity.Canonical.Sacred Require Import FormulaEval.
From Trinity.Canonical.Sacred Require Import ConsistencyChecks.
From Trinity.Canonical.Sacred Require Import Unitarity.
From Trinity.Canonical.Sacred Require Import BoundsGauge.
From Trinity.Canonical.Sacred Require Import BoundsLeptonMasses.
From Trinity.Canonical.Sacred Require Import BoundsMasses.
From Trinity.Canonical.Sacred Require Import BoundsMixing.
From Trinity.Canonical.Sacred Require Import BoundsQuarkMasses.
From Trinity.Canonical.Sacred Require Import GammaPhi3.
From Trinity.Canonical.Sacred Require Import L5Identity.
From Trinity.Canonical.Sacred Require Import StrongCP.
From Trinity.Canonical.Sacred Require Import DLBounds.

(* --- Kernel (phi/Trit/E8 building blocks) --- *)
From Trinity.Canonical.Kernel Require Import Phi.
From Trinity.Canonical.Kernel Require Import PhiAttractor.
From Trinity.Canonical.Kernel Require Import PhiFloat.
From Trinity.Canonical.Kernel Require Import PhiDistance.
From Trinity.Canonical.Kernel Require Import FlowerE8Embedding.
From Trinity.Canonical.Kernel Require Import Trit.
From Trinity.Canonical.Kernel Require Import Semantics.
From Trinity.Canonical.Kernel Require Import GenIdempotency.
From Trinity.Canonical.Kernel Require Import TernarySufficiency.

(* --- IGLA invariants INV-1..INV-9 (runtime-mirror contract) --- *)
From Trinity.Canonical.Igla Require Import INV1_BpbMonotoneBackward.
From Trinity.Canonical.Igla Require Import INV1b_LrPhiOptimality.
From Trinity.Canonical.Igla Require Import INV2_IglaAshaBound.
From Trinity.Canonical.Igla Require Import INV3_Gf16Precision.
From Trinity.Canonical.Igla Require Import INV4_NcaEntropyBand.
From Trinity.Canonical.Igla Require Import INV5_LucasClosureGf16.
From Trinity.Canonical.Igla Require Import INV6_HybridQkGain.
From Trinity.Canonical.Igla Require Import INV7_IglaFoundCriterion.
From Trinity.Canonical.Igla Require Import INV7b_RainbowBridgeConsistency.
From Trinity.Canonical.Igla Require Import INV8_WorkerPoolComposite.
From Trinity.Canonical.Igla Require Import INV9_EmaDecayValid.
From Trinity.Canonical.Igla Require Import IglaCatalog.

(* End of canonical index. All 38 bundles loaded. *)
