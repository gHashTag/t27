(** HoloRMarker4Slot.v — 4-slot R-marker hyper-vector spec for HOLOGRAPHIC tile.
    Issue: https://github.com/gHashTag/trinity-fpga/issues/99
    Lane: Z · codename holo-r-marker-4slot-spec · L-DPC24
    Anchor: φ²+φ⁻²=3
    R15: forbids mutating R-marker cell value in RTL — this file proves the invariant.
    Author: admin@t27.ai
    Part of: gHashTag/t27 Coq SoT (83 .v files / 73 _CoqProject paths — honest count per R5)
*)

Require Import T27.Kernel.Trit.

(** *** Ternary XOR (Kleene-flavoured: trit-wise complement-annihilation) *)
(** trit_xor a a = Zero for all a — used to prove r_marker_xor_self_is_zero *)
Definition trit_xor (a b : trit) : trit :=
  match a, b with
  | Zero, _ => b
  | _, Zero => a
  | Pos,  Pos  => Zero
  | Neg,  Neg  => Zero
  | Pos,  Neg  => Pos
  | Neg,  Pos  => Neg
  end.

Lemma trit_xor_self_zero : forall t : trit, trit_xor t t = Zero.
Proof. destruct t; reflexivity. Qed.

Lemma trit_xor_zero_r : forall t : trit, trit_xor t Zero = t.
Proof. destruct t; reflexivity. Qed.

Lemma trit_xor_comm : forall a b : trit, trit_xor a b = trit_xor b a.
Proof. destruct a, b; reflexivity. Qed.

(** *** 4-slot R-marker hyper-vector *)
Record RMarker4 : Set := mk_rmarker {
  slot0 : trit;
  slot1 : trit;
  slot2 : trit;
  slot3 : trit
}.

(** Zero R-marker (all slots = Zero) *)
Definition rmarker_zero : RMarker4 := mk_rmarker Zero Zero Zero Zero.

(** Slot-wise equality *)
Definition r_marker_eq (a b : RMarker4) : bool :=
  match a, b with
  | mk_rmarker s0a s1a s2a s3a, mk_rmarker s0b s1b s2b s3b =>
      (match s0a, s0b with | Neg, Neg | Zero, Zero | Pos, Pos => true | _, _ => false end) &&
      (match s1a, s1b with | Neg, Neg | Zero, Zero | Pos, Pos => true | _, _ => false end) &&
      (match s2a, s2b with | Neg, Neg | Zero, Zero | Pos, Pos => true | _, _ => false end) &&
      (match s3a, s3b with | Neg, Neg | Zero, Zero | Pos, Pos => true | _, _ => false end)
  end.

(** Slot-wise XOR *)
Definition r_marker_xor (a b : RMarker4) : RMarker4 :=
  match a, b with
  | mk_rmarker s0a s1a s2a s3a, mk_rmarker s0b s1b s2b s3b =>
      mk_rmarker (trit_xor s0a s0b)
                 (trit_xor s1a s1b)
                 (trit_xor s2a s2b)
                 (trit_xor s3a s3b)
  end.

(** Helper: slot-wise trit equality is reflexive *)
Lemma trit_eq_refl : forall t : trit,
  (match t, t with | Neg, Neg | Zero, Zero | Pos, Pos => true | _, _ => false end) = true.
Proof. destruct t; reflexivity. Qed.

(** *** Lemma 1 (R15 XOR-self invariant): XOR of any R-marker with itself is zero. *)
Lemma r_marker_xor_self_is_zero :
  forall x : RMarker4, r_marker_eq (r_marker_xor x x) rmarker_zero = true.
Proof.
  intro x.
  destruct x as [s0 s1 s2 s3].
  unfold r_marker_xor, r_marker_eq, rmarker_zero.
  repeat rewrite trit_xor_self_zero.
  reflexivity.
Qed.

(** *** Lemma 2: XOR is commutative. *)
Lemma r_marker_xor_commutative :
  forall a b : RMarker4, r_marker_xor a b = r_marker_xor b a.
Proof.
  intros a b.
  destruct a as [s0a s1a s2a s3a].
  destruct b as [s0b s1b s2b s3b].
  unfold r_marker_xor.
  repeat rewrite trit_xor_comm.
  reflexivity.
Qed.

(** *** Lemma 3 (R15 immutability invariant): XOR with zero is identity. *)
(** This is the core R15 invariant: an R-marker cell XOR'd with the zero vector
    is unchanged — i.e., XOR with zero does not mutate the R-marker. *)
Lemma r_marker_immutable_under_xor_zero :
  forall x : RMarker4, r_marker_xor x rmarker_zero = x.
Proof.
  intro x.
  destruct x as [s0 s1 s2 s3].
  unfold r_marker_xor, rmarker_zero.
  repeat rewrite trit_xor_zero_r.
  reflexivity.
Qed.
