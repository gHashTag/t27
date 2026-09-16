(* CorePhi.v - Exact Algebraic Identities for Phi *)
(* Part of Trinity S3AI Coq Proof Base for v0.9 Framework *)

(* Psatz supplies lra and nra. The previous version of this file used `lra`
   eleven times and imported neither -- which mattered only once the CI job that
   compiles this directory started working at all (see #3328): coqc had never
   run here, so nothing had ever reported the missing import. *)
Require Import Reals.Reals.
Require Import Psatz.
Open Scope R_scope.

(** Golden ratio definition: φ = (1 + √5) / 2 *)
Definition phi : R := (1 + sqrt(5)) / 2.

(** The one fact about √5 every proof below needs. `field` and `ring` cannot
    discover it: to them `sqrt 5` is an opaque constant, so an identity that
    depends on √5·√5 = 5 is not an identity they can see. *)
Lemma sqrt5_sq : sqrt 5 * sqrt 5 = 5.
Proof.
  apply sqrt_sqrt. lra.
Qed.

Lemma sqrt5_nonneg : 0 <= sqrt 5.
Proof.
  apply sqrt_pos.
Qed.

(** φ is positive *)
Lemma phi_pos : 0 < phi.
Proof.
  unfold phi. pose proof sqrt5_nonneg. lra.
Qed.

(** φ is non-zero *)
Lemma phi_nonzero : phi <> 0.
Proof.
  apply Rgt_not_eq. apply phi_pos.
Qed.

(** φ² = φ + 1 (fundamental golden ratio identity) *)
Lemma phi_square : phi^2 = phi + 1.
Proof.
  unfold phi. pose proof sqrt5_sq. pose proof sqrt5_nonneg. nra.
Qed.

(** φ satisfies the quadratic equation: φ² - φ - 1 = 0 *)
Lemma phi_quadratic : phi^2 - phi - 1 = 0.
Proof.
  pose proof phi_square. lra.
Qed.

(** φ⁻¹ = φ - 1 (reciprocal identity) *)
Lemma phi_inv : / phi = phi - 1.
Proof.
  pose proof phi_square as Hs.
  pose proof phi_nonzero as Hn.
  apply (Rmult_eq_reg_l phi); [ | exact Hn ].
  rewrite (Rinv_r phi Hn).
  nra.
Qed.

(** φ⁻² = 2 - φ (squared reciprocal) *)
Lemma phi_inv_sq : /phi^2 = 2 - phi.
Proof.
  pose proof phi_square as Hs.
  pose proof phi_nonzero as Hn.
  assert (Hsq : phi^2 <> 0) by (apply pow_nonzero; exact Hn).
  apply (Rmult_eq_reg_l (phi^2)); [ | exact Hsq ].
  rewrite (Rinv_r (phi^2) Hsq).
  nra.
Qed.

(** Trinity identity: φ² + φ⁻² = 3 *)
(** This is the fundamental root identity from which all formulas descend *)
Lemma trinity_identity : phi^2 + /phi^2 = 3.
Proof.
  pose proof phi_square. pose proof phi_inv_sq. lra.
Qed.

(** φ³ = 2√5 + 3 (positive cubic power) *)
Lemma phi_cubed : phi^3 = 2 * sqrt(5) + 3.
Proof.
  unfold phi. pose proof sqrt5_sq. pose proof sqrt5_nonneg. nra.
Qed.

(** φ⁻³ = √5 - 2 (negative cubic power) *)
Lemma phi_neg3 : /phi^3 = sqrt(5) - 2.
Proof.
  pose proof phi_cubed as Hc.
  pose proof phi_nonzero as Hn.
  pose proof sqrt5_sq as H5.
  assert (Hcu : phi^3 <> 0) by (apply pow_nonzero; exact Hn).
  apply (Rmult_eq_reg_l (phi^3)); [ | exact Hcu ].
  rewrite (Rinv_r (phi^3) Hcu).
  rewrite Hc. nra.
Qed.

(** φ⁴ = 3√5 + 5 (fourth power) *)
Lemma phi_fourth : phi^4 = 3 * sqrt(5) + 5.
Proof.
  unfold phi. pose proof sqrt5_sq. pose proof sqrt5_nonneg. nra.
Qed.

(** φ⁵ = 5√5 + 8 (fifth power, Fibonacci pattern) *)
Lemma phi_fifth : phi^5 = 5 * sqrt(5) + 8.
Proof.
  unfold phi. pose proof sqrt5_sq. pose proof sqrt5_nonneg. nra.
Qed.

(** Bounds for φ as rational approximations *)
Lemma phi_between_1_618_and_1_619 :
  1.618 < phi < 1.619.
Proof.
  unfold phi.
  pose proof sqrt5_sq as H5.
  pose proof sqrt5_nonneg as H0.
  split; nra.
Qed.

(** Note: φ is irrational (requires classical axioms). *)
(* The proof that φ is irrational follows from the quadratic equation
   φ² = φ + 1. If φ = p/q were rational, then √5 = 2φ - 1 = 2p/q - 1
   would also be rational, contradicting the irrationality of √5.
   A complete proof requires classical axioms and is omitted here. *)
