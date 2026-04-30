(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        INV-6
   Title:         Hybrid QK Gain phi^2
   PhD chapter:   Ch.8 TF3/TF9
   Stats:         Qed=2 · Admitted=5 · Abort=0 · Lines~166
   Source mirror: trinity-clara/proofs/igla/hybrid_qk_gain.v
   Content SHA-1: 42c0f1bc741df691
   Canonical:     gHashTag/t27/proofs/canonical/igla/INV6_HybridQkGain.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

(* ================================================================
   IGLA-INV-013: Hybrid attention QK-gain anchored at phi^2
   File: hybrid_qk_gain.v

   Mission context (from trios#143 Gate-2 pre-registration):
     For the ngram + 1-layer causal self-attention hybrid that is
     the planned Gate-2 architecture, the trainer must clamp the
     softmax gain on the QK^T product to one of the phi-anchored
     values {phi^2, phi^3} = {2.618..., 4.236...}.  Any other gain
     unanchors the head from the Trinity identity and forfeits
     the L-R14 traceability that the race rules require.

     Combined with the lr-band INV-1, this fixes the only two
     architectural degrees of freedom Gate-2 is permitted to vary.

   Hypothesis (Popper-falsifiable, mirrors the pre-reg comment):
     H_INV13:
       forall trainer cfg,  if cfg is admitted by the runtime guard
       then  qk_gain(cfg) in {phi^2, phi^3}
        AND  lr(cfg) in [phi_alpha / phi^4, phi_alpha].

   Falsifier (refutation observable):
     H_INV13 is FALSE iff ANY of the four counter_* cases below is
     accepted by the runtime guard.  In Coq we record the structural
     shape of each counter as a `Definition`; the binding refutation
     is the corresponding `falsify_*` Rust test in
     trios:crates/trios-igla-race/src/bin/qk_gain_check.rs.

   Anchor:  Trinity Identity   phi^2 + phi^-2 = 3
            Zenodo DOI 10.5281/zenodo.19227877

   Compile order (per assertions/igla_assertions.json):
     CorePhi -> lucas_closure_gf16 -> gf16_precision
     -> nca_entropy_band -> lr_convergence -> igla_asha_bound
     -> igla_found_criterion -> hybrid_qk_gain.

   Rust target: trios:crates/trios-igla-race/src/bin/qk_gain_check.rs

   R5 honesty: every theorem in this file is honestly `Admitted`.
   The binding contract is the Rust guard, mirrored test-for-test
   in `falsify_*` Rust tests.  No `Qed.` unless the body has been
   discharged in real Rocq -- this file ships under R5.
   ================================================================ *)

Require Import Stdlib.Reals.Reals.
Require Import CorePhi.
Open Scope R_scope.

(* ----------------------------------------------------------------
   Anchors (mirrored from trios:assertions/igla_assertions.json::INV-13)
   ---------------------------------------------------------------- *)

(* Pre-registered learning-rate ceiling (cf. Gate-2 §8).
   Numeric: phi_alpha = 0.0072.  *)
Definition phi_alpha : R := 0.0072.

(* The lower edge of the admissible lr band, lifted to a pure
   Coq-real expression so we can argue about it without floats. *)
Definition lr_lower : R := phi_alpha / (phi ^ 4).

(* The upper edge.  The band is [lr_lower, phi_alpha] (inclusive). *)
Definition lr_upper : R := phi_alpha.

(* The two admissible QK softmax gains.  No others are permitted. *)
Definition qk_gain_phi_sq : R := phi ^ 2.   (* approx 2.618 *)
Definition qk_gain_phi_cu : R := phi ^ 3.   (* approx 4.236 *)

(* ----------------------------------------------------------------
   Trainer config: only the two race-relevant DOFs.
   The hybrid trainer is permitted to vary nothing else.
   ---------------------------------------------------------------- *)

Definition Cfg : Type := (R * R)%type.   (* (lr, qk_gain) *)

Definition cfg_lr   (c : Cfg) : R := fst c.
Definition cfg_gain (c : Cfg) : R := snd c.

(* Admissibility predicate, structural form.  Phi-anchored gain AND
   lr clamped to the pre-registered band. *)
Definition qk_gain_admissible (g : R) : Prop :=
  g = qk_gain_phi_sq \/ g = qk_gain_phi_cu.

Definition lr_admissible (l : R) : Prop :=
  lr_lower <= l <= lr_upper.

Definition cfg_admissible (c : Cfg) : Prop :=
  qk_gain_admissible (cfg_gain c) /\ lr_admissible (cfg_lr c).

(* ================================================================
   FALSIFICATION SHAPES (R8) -- documented but Admitted.

   Each `counter_*` records the shape of an input the runtime gate
   must REJECT.  The binding refutation is the matching `falsify_*`
   Rust test in qk_gain_check.rs.  All four are Admitted in this
   file; flipping any to Qed without real Rocq elaboration of the
   reals arithmetic violates R5.
   ================================================================ *)

(* Counter 1: lr exactly 0.01 -- the pre-attention-only attempt that
   plateaued at BPB ~ 4.74 (cf. pre-reg §1).  Above the lr-ceiling. *)
Theorem counter_lr_above_band :
  ~ lr_admissible 0.01.
Proof. Admitted.

(* Counter 2: lr = 0.0001 -- one decade below the band's lower edge. *)
Theorem counter_lr_below_band :
  ~ lr_admissible 0.0001.
Proof. Admitted.

(* Counter 3: gain = 1.0 -- vanilla softmax attention (no temperature),
   the textbook setting.  Forbidden because un-anchored. *)
Theorem counter_gain_unit :
  ~ qk_gain_admissible 1.
Proof. Admitted.

(* Counter 4: gain = sqrt(d_model) = sqrt(64) = 8 -- the conventional
   attention scaling.  Forbidden by the phi-anchored gain rule. *)
Theorem counter_gain_sqrt_d_model :
  ~ qk_gain_admissible 8.
Proof. Admitted.

(* ================================================================
   POSITIVE SHAPES -- the two admissible gains are accepted.
   These ARE definitionally trivial (Coq reduces `or`-introduction
   on a `Definition`-equal value), so they ship as `Qed.`.
   ================================================================ *)

Lemma admit_phi_sq : qk_gain_admissible (phi ^ 2).
Proof. left.  reflexivity. Qed.

Lemma admit_phi_cu : qk_gain_admissible (phi ^ 3).
Proof. right. reflexivity. Qed.

(* ================================================================
   STRUCTURAL WELL-TYPEDNESS  (Admitted under R5).

   The runtime guard `qk_gain_check.rs::admit_cfg` is the binding
   contract.  This theorem records what we *would* have to prove
   if we wanted to lift the runtime check into Coq.
   ================================================================ *)

Theorem hybrid_qk_gain_phi_sq_well_typed :
  forall c : Cfg,
    cfg_admissible c
    -> qk_gain_admissible (cfg_gain c) /\ lr_admissible (cfg_lr c).
Proof. Admitted.

(* End of file.  Compile target:
     coqc -Q . CorePhi proofs/igla/hybrid_qk_gain.v
   Expected: 0 errors.

   R5 honesty roster:
     Admitted (5):
       counter_lr_above_band
       counter_lr_below_band
       counter_gain_unit
       counter_gain_sqrt_d_model
       hybrid_qk_gain_phi_sq_well_typed
     Qed (2):
       admit_phi_sq
       admit_phi_cu

   The binding refutation contract for the four counter_* shapes
   lives in the matching falsify_* Rust unit tests.
*)
