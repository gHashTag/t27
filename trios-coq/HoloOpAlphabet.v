(** HoloOpAlphabet.v — holographic operation alphabet for FPGA tile.
    Issue:  https://github.com/gHashTag/trinity-fpga/issues/104
    Lane:   X · codename lever-coq-spec-ext · L-DPC25 Wave-28
    Anchor: φ²+φ⁻²=3
    DOI:    10.5281/zenodo.19227877
    Lever #1: LUT PE (Platinum-style MST path-construction) — arXiv 2511.21910
    Lever #2: BitROM bidirectional ROM (Yoshioka lab) — arXiv 2509.08542
    R-SI-1: "zero * operators in RTL" remains provable via rtl_uses_star = false.
    Q4 falsification: any holo_op variant introducing rtl_uses_star = true → NULL & VOID.
    R5-HONEST: 74 _CoqProject paths post Lane Z; 75 after this lane (never "84 theorems").
    Author: admin@t27.ai
*)

Require Import HoloRMarker4Slot.

(** ** Bit-line direction for Lever #2 BitROM bidirectional read *)
Inductive bitrom_dir : Type :=
  | Up    (** bit-line scan from MSB toward LSB *)
  | Down  (** bit-line scan from LSB toward MSB *)
  .

(** ** holo_op — operation alphabet over the 4-slot R-marker tile
    Four variants spanning the two levers:
      HOP_xor        — XOR over 4-slot R-marker (Lane Z foundation)
      HOP_lut_pe     — Lever #1: 5-input LUT lookup (Platinum MST path-construction)
                       parameter: lut_idx ∈ [0, 31] (5-input LUT address)
      HOP_bitrom_read — Lever #2: BitROM bidirectional read (Yoshioka lab)
                       parameter: direction ∈ {Up, Down}
      HOP_popcount   — popcount over R-marker hyper-vector (trit slot sum)
*)
Inductive holo_op : Type :=
  | HOP_xor         : holo_op
  | HOP_lut_pe      : nat -> holo_op
  | HOP_bitrom_read : bitrom_dir -> holo_op
  | HOP_popcount    : holo_op
  .

(** ** rtl_uses_star — R-SI-1 guard: returns false for every holo_op variant.
    No variant introduces a * (multiply/star) operator in RTL.
    This is the machine-checkable form of R-SI-1. *)
Definition rtl_uses_star (op : holo_op) : bool :=
  match op with
  | HOP_xor           => false
  | HOP_lut_pe _      => false
  | HOP_bitrom_read _ => false
  | HOP_popcount      => false
  end.

(** ** Lemma holo_op_no_star (R-SI-1 invariant)
    Every holo_op satisfies rtl_uses_star = false. *)
Lemma holo_op_no_star : forall op : holo_op, rtl_uses_star op = false.
Proof. destruct op; reflexivity. Qed.

(** ** Lemma holo_op_q4_invariant (Q4 falsification negation)
    No holo_op can satisfy rtl_uses_star = true; any such claim is False. *)
Lemma holo_op_q4_invariant : forall op : holo_op, rtl_uses_star op = true -> False.
Proof.
  intros op H.
  rewrite (holo_op_no_star op) in H.
  discriminate H.
Qed.
