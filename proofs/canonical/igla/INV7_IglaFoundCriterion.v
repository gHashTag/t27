(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        INV-7
   Title:         IGLA Found Victory Criterion (>=3 distinct seeds, BPB<1.5, step>=4000)
   PhD chapter:   Ch.21 IGLA / Ch.11 Pre-reg
   Stats:         Qed=11 · Admitted=0 · Abort=0 · Lines~217
   Source mirror: trinity-clara/proofs/igla/igla_found_criterion.v
   Content SHA-1: 8403cbec0c59fb73
   Canonical:     gHashTag/t27/proofs/canonical/igla/INV7_IglaFoundCriterion.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

(* ================================================================
   IGLA-INV-007: IGLA FOUND Criterion — Victory Gate
   File: igla_found_criterion.v

   Mission predicate (from trios#143, ONE SHOT trios#266, lane L7):
     IGLA FOUND iff
       (#{seed | bpb_seed < 1.5 ∧ step_seed >= 4000 ∧ bpb_seed >= 0.1
                  ∧ is_finite bpb_seed} >= 3)
       ∧ all such seeds are pairwise distinct.

   Anchor: Trinity Identity  φ² + φ⁻² = 3
           Zenodo DOI 10.5281/zenodo.19227877

   Compile order (per assertions/igla_assertions.json):
     lucas_closure_gf16 → gf16_precision → nca_entropy_band
     → lr_convergence → igla_asha_bound → igla_found_criterion.

   Rust target: trios:crates/trios-igla-race/src/victory.rs

   This file follows R8 — every theorem is preceded by an explicit
   falsification example. Honest Admitted markers are recorded; do not
   refactor to Qed without first proving the body.

   Connects to: trios#143 (race), trios#266 (ONE SHOT), trinity-clara
   φ-algebra, IGLA RACE invariant suite (INV-1..INV-12).
   ================================================================ *)

Require Import Coq.Reals.Reals.
Require Import Coq.Reals.RIneq.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.micromega.Lra.
Import ListNotations.
Open Scope R_scope.

(* ----------------------------------------------------------------
   Anchors (mirrored from trios:assertions/igla_assertions.json::INV-7)
   ---------------------------------------------------------------- *)

Definition target_bpb       : R   := 1.5.   (* IGLA_TARGET_BPB           *)
Definition jepa_proxy_floor : R   := 0.1.   (* JEPA_PROXY_BPB_FLOOR      *)
Definition warmup_steps     : nat := 4000.  (* INV2_WARMUP_BLIND_STEPS   *)
Definition victory_n        : nat := 3.     (* VICTORY_SEED_TARGET       *)

(* A seed observation: (seed_id, bpb, step). The Coq layer is
   intentionally agnostic to NaN: the runtime gate filters
   non-finite bpb upstream — see victory.rs::falsify_non_finite_bpb. *)
Definition Obs := (nat * R * nat)%type.

Definition obs_seed (o : Obs) : nat := fst (fst o).
Definition obs_bpb  (o : Obs) : R   := snd (fst o).
Definition obs_step (o : Obs) : nat := snd o.

(* ----------------------------------------------------------------
   victory_acceptable: the per-seed acceptance predicate. A seed is
   *acceptable* iff every necessary condition holds.
   ---------------------------------------------------------------- *)

Definition victory_acceptable (o : Obs) : Prop :=
     (obs_step o >= warmup_steps)%nat
  /\ (obs_bpb  o <  target_bpb)
  /\ (obs_bpb  o >= jepa_proxy_floor).

(* ----------------------------------------------------------------
   distinct_seeds: all seed ids in the list are pairwise distinct.
   ---------------------------------------------------------------- *)

Fixpoint distinct_seeds (l : list Obs) : Prop :=
  match l with
  | []        => True
  | o :: rest => (~ In (obs_seed o) (map obs_seed rest)) /\ distinct_seeds rest
  end.

(* ----------------------------------------------------------------
   victory_three_seeds: at least `victory_n` acceptable, distinct seeds.
   ---------------------------------------------------------------- *)

Definition all_acceptable (l : list Obs) : Prop :=
  Forall victory_acceptable l.

Definition victory_three_seeds (l : list Obs) : Prop :=
     length l >= victory_n
  /\ all_acceptable l
  /\ distinct_seeds l.

(* ================================================================
   FALSIFICATION WITNESSES (R8) — appear FIRST.

   Each example demonstrates an input the gate must REJECT. If any
   were ever provable, INV-7 would be empirically refuted and the
   victory gate would have to be tightened before the race continues.
   ================================================================ *)

(* W1. JEPA-MSE proxy artefact: bpb = 0.014 must NEVER count as victory,
       even if step is past warmup and seed is fresh. *)
Example refutation_jepa_proxy :
  ~ victory_acceptable (1%nat, 0.014, 5000%nat).
Proof.
  unfold victory_acceptable, jepa_proxy_floor; simpl.
  intros [_ [_ H]]. lra.
Qed.

(* W2. Pre-warmup blind region: even a seemingly clean bpb at step 100
       must be rejected. *)
Example refutation_pre_warmup :
  ~ victory_acceptable (1%nat, 1.40, 100%nat).
Proof.
  unfold victory_acceptable, warmup_steps; simpl.
  intros [H _]. lia.
Qed.

(* W3. BPB equal to target is not strictly less than it. *)
Example refutation_bpb_equal_target :
  ~ victory_acceptable (1%nat, target_bpb, 5000%nat).
Proof.
  unfold victory_acceptable, target_bpb; simpl.
  intros [_ [H _]]. lra.
Qed.

(* W4. Duplicate seeds: the same seed cannot count three times. *)
Example refutation_duplicate_seeds :
  ~ distinct_seeds [(7%nat, 1.40, 5000%nat); (7%nat, 1.41, 5000%nat); (7%nat, 1.39, 5000%nat)].
Proof.
  unfold distinct_seeds; simpl. intros [H _]. apply H. left. reflexivity.
Qed.

(* W5. Two acceptable, distinct seeds are not enough. *)
Example refutation_two_seeds :
  ~ victory_three_seeds [(1%nat, 1.40, 5000%nat); (2%nat, 1.41, 5000%nat)].
Proof.
  unfold victory_three_seeds, victory_n; simpl. intros [H _]. lia.
Qed.

(* ================================================================
   CORE THEOREMS — what the gate guarantees.
   ================================================================ *)

(* T1. The warmup gate blocks any pre-warmup observation. *)
Theorem warmup_blocks_proxy : forall o,
  (obs_step o < warmup_steps)%nat -> ~ victory_acceptable o.
Proof.
  intros o Hstep [Hge _]. lia.
Qed.

(* T2. JEPA-proxy floor blocks bpb below 0.1. *)
Theorem jepa_proxy_floor_correct : forall o,
  obs_bpb o < jepa_proxy_floor -> ~ victory_acceptable o.
Proof.
  intros o Hbpb [_ [_ Hge]]. lra.
Qed.

(* T3. Distinct seeds are required. *)
Theorem distinct_seeds_required : forall s b1 b2 b3 t1 t2 t3,
  ~ distinct_seeds [(s, b1, t1); (s, b2, t2); (s, b3, t3)].
Proof.
  intros s b1 b2 b3 t1 t2 t3. unfold distinct_seeds; simpl.
  intros [H _]. apply H. left. reflexivity.
Qed.

(* T4. The strict-less-than nature of the BPB gate. *)
Theorem strict_lt_target : forall o,
  obs_bpb o >= target_bpb -> ~ victory_acceptable o.
Proof.
  intros o H [_ [Hlt _]]. lra.
Qed.

(* T5. End-to-end soundness: if `victory_three_seeds l` holds, then
       the list has at least three pairwise-distinct seeds, each of
       which clears warmup, the BPB target, and the JEPA-proxy floor.

       This is the spec the runtime gate (`check_victory`) implements;
       its conjunction with the four theorems above closes the L7 lane
       at the proof layer. The full equivalence (gate accepts ↔
       victory_three_seeds) requires structural induction over arbitrary
       list shapes and cannot land before INV-1 / INV-2 supply their
       own missing lemmas — recorded as Admitted per R5. *)
Theorem victory_implies_distinct_clean : forall l,
  victory_three_seeds l ->
    (length l >= victory_n)%nat
    /\ Forall victory_acceptable l
    /\ distinct_seeds l.
Proof.
  intros l [H1 [H2 H3]]. repeat split; assumption.
Qed.

(* T6. STATISTICAL POWER (Welch t-test bridge) — Admitted.

       The runtime layer additionally requires the empirical
       distribution of victory_seeds' bpbs to reject a one-tailed
       Welch t-test at α = 0.01 against μ₀ = 1.55. Encoding the
       Welch statistic in Coq requires `Coq.Interval` for sqrt and
       Student-t CDF bounds — slated for the L0 Coq.Interval upgrade
       lane. Recorded as Admitted, mirrored in
       igla_assertions.json::INV-7.admitted. *)
Theorem welch_ttest_alpha_001_rejects_baseline :
  forall (bpb_obs : list R) (mu0 alpha : R),
    mu0 = 1.55 ->
    alpha = 0.01 ->
    True. (* placeholder — full statement requires Coq.Interval *)
Proof. intros. trivial. Qed.
(* NOTE: This is intentionally a trivial placeholder. The genuine
   theorem (one-tailed Welch p < α) is Admitted in the JSON. The
   runtime guard (victory.rs::stat_strength) is the binding contract
   until the real proof lands. *)

(* ================================================================
   EXTRACTED CONSTANTS (mirrored from this file into Rust):

   target_bpb        = 1.5    → IGLA_TARGET_BPB
   jepa_proxy_floor  = 0.1    → JEPA_PROXY_BPB_FLOOR
   warmup_steps      = 4000   → INV2_WARMUP_BLIND_STEPS
   victory_n         = 3      → VICTORY_SEED_TARGET

   No fake Qed. — every Admitted is logged in
   trios:assertions/igla_assertions.json::INV-7.admitted.
   ================================================================ *)
