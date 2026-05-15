(** * RMarker — 4-slot Holographic Working-Memory Register File
    L-DPC24 Lane Z · holo-r-marker-4slot-spec
    Reference: https://github.com/gHashTag/trinity-fpga/issues/100 *)

Set Warnings "-stdlib-vector".

Require Import Coq.Init.Datatypes.
Require Import Coq.Init.Nat.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Fin.

Import ListNotations.

(** ** 1. Slot type: four R-marker slots *)

Definition Slot := Fin.t 4.

(** ** 2. RMarker record *)

Record RMarker : Set := mkRMarker
  { slot  : Slot
  ; value : nat
  ; phase : nat
  }.

(** ** 3. Lemmas *)

(** *** 3.1  slot_bounded *)

Lemma slot_bounded : forall (m : RMarker), proj1_sig (Fin.to_nat (slot m)) < 4.
Proof.
  intro m.
  exact (proj2_sig (Fin.to_nat (slot m))).
Qed.

(** *** 3.2  marker_eq_dec — decidable equality *)

Lemma fin_eq_dec : forall (n : nat) (a b : Fin.t n), {a = b} + {a <> b}.
Proof.
  intros n a.
  induction a as [| n' a' IH].
  - intro b. destruct b using Fin.caseS'.
    + left. reflexivity.
    + right. discriminate.
  - intro b. destruct b using Fin.caseS'.
    + right. discriminate.
    + destruct (IH b) as [Heq | Hneq].
      * left. subst. reflexivity.
      * right. intro H. apply Hneq. apply Fin.FS_inj in H. exact H.
Defined.

Lemma marker_eq_dec : forall (a b : RMarker), {a = b} + {a <> b}.
Proof.
  intros [sa va pa] [sb vb pb].
  destruct (fin_eq_dec 4 sa sb) as [Hs | Hs].
  - destruct (Nat.eq_dec va vb) as [Hv | Hv].
    + destruct (Nat.eq_dec pa pb) as [Hp | Hp].
      * left. subst. reflexivity.
      * right. intro H. inversion H. contradiction.
    + right. intro H. inversion H. contradiction.
  - right. intro H. inversion H. contradiction.
Defined.

(** *** 3.3  four_distinct_slots_exist *)

Lemma four_distinct_slots_exist :
  exists s0 s1 s2 s3 : Slot,
    s0 <> s1 /\ s0 <> s2 /\ s0 <> s3 /\
    s1 <> s2 /\ s1 <> s3 /\
    s2 <> s3.
Proof.
  exists Fin.F1, (Fin.FS Fin.F1), (Fin.FS (Fin.FS Fin.F1)), (Fin.FS (Fin.FS (Fin.FS Fin.F1))).
  repeat split; discriminate.
Qed.

(** ** 4. Holographic invariant *)

Definition holographic_invariant (mks : list RMarker) : Prop :=
  length mks <= 4 /\ NoDup (map slot mks).

Lemma empty_satisfies : holographic_invariant [].
Proof.
  unfold holographic_invariant.
  split.
  - simpl. apply Nat.le_0_l.
  - simpl. apply NoDup_nil.
Qed.

(* phi^2 + phi^-2 = 3 *)
(* DOI 10.5281/zenodo.19227877 *)
(* Vasilev Dmitrii <admin@t27.ai> *)
(* ORCID 0009-0008-4294-6159 *)
