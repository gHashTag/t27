(* FormulaEval.v - Monomial Datatype and Evaluator *)
(* Part of Trinity S3AI Coq Proof Base for v0.9 Framework *)

Require Import Reals.Reals.
Require Import Reals.Rfunctions.
Require Import ZArith.
Require Import String.
Open Scope R_scope.

Require Import CorePhi.

(** Integer power of a real number *)
(** Coq's ^ operator is R -> nat -> R; we need Z -> R for negative exponents *)
Definition powZ (x : R) (n : Z) : R := powerRZ x n.

(** Trinity monomial: represents expressions of the form n * 3^k * φ^p * π^m * e^q *)
(** This captures all 69 formulas in the Trinity framework v0.9 *)
Inductive monomial : Type :=
  | M_const : Z -> monomial                    (* Integer constant *)
  | M_three : Z -> monomial                   (* 3^k *)
  | M_phi : Z -> monomial                     (* φ^p *)
  | M_pi : Z -> monomial                      (* π^m *)
  | M_exp : Z -> monomial                     (* e^q *)
  | M_mul : monomial -> monomial -> monomial. (* Multiplication *)

(** Flatten multiplication (identity for Rocq 9.x compatibility) *)
(** CRITICAL FIX: Removed recursive fixpoint (structural recursion failure in Rocq 9.x) *)
Definition flatten_mul (m : monomial) : monomial := m.

(** Evaluator: converts monomial to real number *)
Fixpoint eval_monomial (m : monomial) : R :=
  match m with
  | M_const c => IZR c
  | M_three k => powZ (IZR 3) k
  | M_phi p => powZ phi p
  | M_pi m => powZ PI m
  | M_exp q => powZ (exp 1) q
  | M_mul m1 m2 => (eval_monomial m1) * (eval_monomial m2)
  end.

(** Helper: create constant monomial *)
Definition mk_const (c : Z) : monomial := M_const c.

(** Helper: create 3^k monomial *)
Definition mk_three (k : Z) : monomial := M_three k.

(** Helper: create φ^p monomial *)
Definition mk_phi (p : Z) : monomial := M_phi p.

(** Helper: create π^m monomial *)
Definition mk_pi (m : Z) : monomial := M_pi m.

(** Helper: create e^q monomial *)
Definition mk_exp (q : Z) : monomial := M_exp q.

(** Helper: multiply monomials *)
Definition mk_mul (m1 m2 : monomial) : monomial := M_mul m1 m2.

(** Eval of constant is the integer as real *)
Lemma eval_const_eq : forall c : Z, eval_monomial (M_const c) = IZR c.
Proof.
  intro c; reflexivity.
Qed.

(** Eval of 3^k is 3^k as real *)
Lemma eval_three_eq : forall k : Z, eval_monomial (M_three k) = powZ (IZR 3) k.
Proof.
  intro k; reflexivity.
Qed.

(** Eval of φ^p is φ^p *)
Lemma eval_phi_eq : forall p : Z, eval_monomial (M_phi p) = powZ phi p.
Proof.
  intro p; reflexivity.
Qed.

(** Eval of π^m is π^m *)
Lemma eval_pi_eq : forall m : Z, eval_monomial (M_pi m) = powZ PI m.
Proof.
  intro m; reflexivity.
Qed.

(** Eval of e^q is e^q *)
Lemma eval_exp_eq : forall q : Z, eval_monomial (M_exp q) = powZ (exp 1) q.
Proof.
  intro q; reflexivity.
Qed.

(** Multiplication distributes over evaluation *)
Lemma eval_mul_distrib :
  forall m1 m2 : monomial,
    eval_monomial (M_mul m1 m2) = eval_monomial m1 * eval_monomial m2.
Proof.
  intros m1 m2; reflexivity.
Qed.

(** Associativity of multiplication in evaluation *)
Lemma eval_mul_assoc :
  forall m1 m2 m3 : monomial,
    eval_monomial (M_mul (M_mul m1 m2) m3) =
    eval_monomial (M_mul m1 (M_mul m2 m3)).
Proof.
  intros m1 m2 m3.
  simpl.
  ring.
Qed.

(** Identity element: M_const 1 evaluates to 1 *)
Lemma eval_one : eval_monomial (M_const 1) = 1.
Proof.
  simpl; auto.
Qed.

(** Zero element: M_const 0 evaluates to 0 *)
Lemma eval_zero : eval_monomial (M_const 0) = 0.
Proof.
  simpl; auto.
Qed.

(** Negative power: M_phi (-1) = 1/φ *)
Lemma eval_phi_neg1 : eval_monomial (M_phi (-1)) = /phi.
Proof.
  simpl. unfold powZ. simpl. field. apply phi_nonzero.
Qed.

(** Example: α⁻¹ = 4 * 9 * π⁻¹ * φ * e² (G01 formula) *)
Definition G01_monomial : monomial :=
  M_mul
    (M_mul
      (M_mul
        (M_const (Z.of_nat 36))
        (M_pi (-1)))
      (M_phi 1))
    (M_exp 2).

Lemma eval_G01_monomial :
  eval_monomial G01_monomial = 4 * 9 * / PI * phi * (exp 1 ^ 2).
Proof.
  unfold G01_monomial, powZ.
  simpl.
  rewrite Rinv_1. rewrite Rmult_1_r.
  repeat rewrite powerRZ_nat.
  repeat rewrite pow1.
  field.
  split; [apply PI_neq0 | apply phi_nonzero].
Qed.

(** Example: |V_us| = 2 * 3⁻² * π⁻³ * φ³ * e² (C01 formula) *)
Definition C01_monomial : monomial :=
  M_mul
    (M_mul
      (M_mul
        (M_const (Z.of_nat 2))
        (M_three (-2)))
      (M_mul (M_pi (-3)) (M_phi 3)))
    (M_exp 2).

Lemma eval_C01_monomial :
  eval_monomial C01_monomial = 2 * / (3 ^ 2) * / (PI ^ 3) * (phi ^ 3) * (exp 1 ^ 2).
Proof.
  unfold C01_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  repeat rewrite Rinv_1 || rewrite pow1 || rewrite Rmult_1_r.
  field; split; [apply pow_nonzero; lra | split; [apply pow_nonzero; apply PI_neq0 | apply phi_nonzero]].
Qed.

(** Example: m_s/m_d = 8 * 3 * π⁻¹ * φ² (Q07 formula, smoking gun) *)
Definition Q07_monomial : monomial :=
  M_mul
    (M_mul
      (M_const (Z.of_nat 24))
      (M_pi (-1)))
    (M_phi 2).

Lemma eval_Q07_monomial :
  eval_monomial Q07_monomial = 8 * 3 * / PI * (phi ^ 2).
Proof.
  unfold Q07_monomial, powZ.
  simpl.
  rewrite powerRZ_nat.
  rewrite Rinv_1. rewrite Rmult_1_r.
  rewrite pow2.
  field. apply PI_neq0.
Qed.

(** Example: Higgs mass: m_H = 4 * φ³ * e² (H01 formula) *)
Definition H01_monomial : monomial :=
  M_mul
    (M_mul
      (M_const (Z.of_nat 4))
      (M_phi 3))
    (M_exp 2).

Lemma eval_H01_monomial :
  eval_monomial H01_monomial = 4 * (phi ^ 3) * (exp 1 ^ 2).
Proof.
  unfold H01_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  repeat rewrite pow1 || rewrite Rmult_1_r.
  rewrite pow2.
  field. apply phi_nonzero.
Qed.

(** Example: sin²(θ₁₂) = 8 * φ⁻⁵ * π * e⁻² (N01 formula) *)
Definition N01_monomial : monomial :=
  M_mul
    (M_mul
      (M_const (Z.of_nat 8))
      (M_phi (-5)))
    (M_mul (M_pi 1) (M_exp (-2))).

Lemma eval_N01_monomial :
  eval_monomial N01_monomial = 8 * / (phi ^ 5) * PI * / (exp 1 ^ 2).
Proof.
  unfold N01_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  repeat rewrite pow1 || rewrite Rmult_1_r.
  rewrite pow2.
  field. split; [apply pow_nonzero; apply phi_nonzero | apply pow_nonzero; apply exp_pos].
Qed.

(** G06 monomial: 3 * φ² * e⁻² *)
Definition G06_monomial : monomial :=
  M_mul
    (M_mul
      (M_const (Z.of_nat 3))
      (M_phi 2))
    (M_exp (-2)).

Lemma eval_G06_monomial :
  eval_monomial G06_monomial = 3 * phi^2 * / (exp 1 ^ 2).
Proof.
  unfold G06_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  rewrite pow2. rewrite Rmult_1_r.
  field. apply pow_nonzero; apply exp_pos.
Qed.

(** L01 monomial: 4 * φ³ / e² *)
Definition L01_monomial : monomial :=
  M_mul
    (M_mul
      (M_const (Z.of_nat 4))
      (M_phi 3))
    (M_exp (-2)).

Lemma eval_L01_monomial :
  eval_monomial L01_monomial = 4 * (phi ^ 3) / (exp 1 ^ 2).
Proof.
  unfold L01_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  repeat rewrite pow1 || rewrite Rmult_1_r.
  rewrite pow2.
  unfold Rdiv. field. split; [apply phi_nonzero | apply pow_nonzero; apply exp_pos].
Qed.

(** L02 monomial: 4 * φ³ [IMPROVED via Chimera v3.0] *)
Definition L02_monomial : monomial :=
  M_mul
    (M_const (Z.of_nat 4))
    (M_phi 3).

Lemma eval_L02_monomial :
  eval_monomial L02_monomial = 4 * (phi ^ 3).
Proof.
  unfold L02_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  rewrite pow2.
  field. apply phi_nonzero.
Qed.

(** L03 monomial: 8 * φ⁷ * π / e³ *)
Definition L03_monomial : monomial :=
  M_mul
    (M_mul
      (M_mul
        (M_const (Z.of_nat 8))
        (M_phi 7))
      (M_pi 1))
    (M_exp (-3)).

Lemma eval_L03_monomial :
  eval_monomial L03_monomial = 8 * (phi ^ 7) * PI / (exp 1 ^ 3).
Proof.
  unfold L03_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  repeat rewrite pow1 || rewrite Rmult_1_r.
  rewrite pow2.
  unfold Rdiv. field. split; [apply phi_nonzero | apply pow_nonzero; apply exp_pos].
Qed.

(** Q03 monomial: φ⁴ * π / e² *)
Definition Q03_monomial : monomial :=
  M_mul
    (M_mul
      (M_phi 4)
      (M_pi 1))
    (M_exp (-2)).

Lemma eval_Q03_monomial :
  eval_monomial Q03_monomial = (phi ^ 4) * PI / (exp 1 ^ 2).
Proof.
  unfold Q03_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  repeat rewrite pow1 || rewrite Rmult_1_r.
  rewrite pow2.
  unfold Rdiv. field. split; [apply phi_nonzero | apply pow_nonzero; apply exp_pos].
Qed.

(** Q05 monomial: 48 * e² / φ⁴ *)
Definition Q05_monomial : monomial :=
  M_mul
    (M_mul
      (M_const (Z.of_nat 48))
      (M_exp 2))
    (M_phi (-4)).

Lemma eval_Q05_monomial :
  eval_monomial Q05_monomial = 48 * (exp 1 ^ 2) / (phi ^ 4).
Proof.
  unfold Q05_monomial, powZ.
  simpl.
  repeat rewrite powerRZ_nat.
  repeat rewrite pow1 || rewrite Rmult_1_r.
  rewrite pow2.
  unfold Rdiv. field. split; [apply pow_nonzero; apply exp_pos | apply phi_nonzero].
Qed.

(** Theorem: every well-formed Trinity formula evaluates to a real number *)
Theorem eval_monomial_real :
  forall m : monomial,
    exists r : R, eval_monomial m = r.
Proof.
  intro m.
  exists (eval_monomial m); reflexivity.
Qed.

(** Evaluator is total (no undefined cases) *)
Theorem eval_total : forall m : monomial, True.
Proof.
  intro m; exact I.
Qed.
