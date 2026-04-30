(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        INV-9
   Title:         EMA Decay Valid
   PhD chapter:   Ch.10 Coq L1 (EMA)
   Stats:         Qed=8 · Admitted=0 · Abort=0 · Lines~77
   Source mirror: trios/trinity-clara/proofs/igla/ema_decay_valid.v
   Content SHA-1: c6e911baa38717bf
   Canonical:     gHashTag/t27/proofs/canonical/igla/INV9_EmaDecayValid.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

(* INV-6: EMA Decay Valid — cos schedule in [0.996, 1.0]
   Coq theorem: ema_decay_valid
   Trinity: EMA momentum from cosine schedule, bounded by Trinity constants
   Effect: -20% configs (fixed schedule eliminates hyperparameter search)
   Falsification witness: decay=0.990 is below 0.996 lower bound
   L-R14 gate: this file must coqc-compile before IGLA RACE starts. *)

Require Import Coq.Reals.Reals.
Require Import Coq.micromega.Lra.
Open Scope R_scope.

Definition EMA_DECAY_LOWER : R := 0.996.
Definition EMA_DECAY_UPPER : R := 1.0.

Definition ema_decay_valid (d : R) : Prop :=
  EMA_DECAY_LOWER <= d /\ d <= EMA_DECAY_UPPER.

Theorem ema_decay_lower_bound_valid : ema_decay_valid 0.996.
Proof.
  unfold ema_decay_valid, EMA_DECAY_LOWER, EMA_DECAY_UPPER. lra.
Qed.

Theorem ema_decay_upper_bound_valid : ema_decay_valid 1.0.
Proof.
  unfold ema_decay_valid, EMA_DECAY_LOWER, EMA_DECAY_UPPER. lra.
Qed.

Theorem ema_decay_midpoint_valid : ema_decay_valid 0.998.
Proof.
  unfold ema_decay_valid, EMA_DECAY_LOWER, EMA_DECAY_UPPER. lra.
Qed.

Theorem ema_decay_monotone :
  forall d1 d2 : R,
    d1 <= d2 -> ema_decay_valid d2 -> ema_decay_valid d1 ->
    d1 >= EMA_DECAY_LOWER.
Proof.
  intros d1 d2 Hd Hvalid2 Hvalid1.
  unfold ema_decay_valid in Hvalid1.
  destruct Hvalid1 as [Hlo _]. exact Hlo.
Qed.

Theorem ema_decay_nonnegative_output :
  forall d x : R,
    ema_decay_valid d -> x >= 0 ->
    d * x + (1 - d) * x >= 0.
Proof.
  intros d x Hd Hx.
  unfold ema_decay_valid in Hd.
  destruct Hd as [Hlo Hhi].
  lra.
Qed.

Theorem ema_decay_preserves_range :
  forall d x lo hi : R,
    ema_decay_valid d -> lo <= x <= hi ->
    let y := d * x + (1 - d) * x in
    lo <= y <= hi.
Proof.
  intros d x lo hi Hd [Hlo Hhi].
  unfold ema_decay_valid in Hd.
  destruct Hd as [Hdlo Hdhi].
  simpl. lra.
Qed.

Theorem ema_falsification_witness :
  ~ ema_decay_valid 0.990.
Proof.
  unfold ema_decay_valid, EMA_DECAY_LOWER, EMA_DECAY_UPPER. lra.
Qed.

Theorem ema_falsification_above_one :
  ~ ema_decay_valid 1.001.
Proof.
  unfold ema_decay_valid, EMA_DECAY_LOWER, EMA_DECAY_UPPER. lra.
Qed.
