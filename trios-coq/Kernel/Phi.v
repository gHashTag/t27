(** Golden ratio as real; algebraic identities (AXIOM-K2 / PHI-IDENTITY).
    Layer A: pure [Coq.Reals] — no Flocq here. See [Kernel/PhiFloat.v] for Flocq link. *)

Require Import Reals.
Require Import Psatz.
Require Import ZArith.
Require Import RealField.

Open Scope R_scope.

Definition phi : R := (1 + sqrt 5) / 2.

(** Denominator 2^53 for IEEE binary64 relative unit u = 2^{-53}. *)
Definition coeff_53 : Z := 9007199254740992%Z.

Lemma coeff_53_pos : (0 < coeff_53)%Z.
Proof. unfold coeff_53; lia. Qed.

(** Engineering tolerance: 5 * u * phi^2 with u = 2^{-53} (see TZ §5.5). *)
Definition phi_tolerance : R := 5 * / IZR coeff_53 * (phi * phi).

Lemma sqrt5_sq : sqrt 5 * sqrt 5 = 5.
Proof.
  replace (sqrt 5 * sqrt 5) with (Rsqr (sqrt 5)).
  - rewrite Rsqr_sqrt; lra.
  - unfold Rsqr; ring.
Qed.

Lemma sqrt5_pos : 0 < sqrt 5.
Proof. apply sqrt_lt_R0; lra. Qed.

Lemma sqrt4 : sqrt 4 = 2.
Proof.
  replace 4 with (2 * 2) by ring.
  rewrite sqrt_square; lra.
Qed.

Lemma sqrt5_gt_2 : 2 < sqrt 5.
Proof.
  rewrite <- sqrt4.
  apply sqrt_lt_1; lra.
Qed.

Lemma phi_pos : 0 < phi.
Proof.
  unfold phi.
  assert (H1lt : 1 < 1 + sqrt 5).
  {
    rewrite <- (Rplus_0_r 1) at 1.
    apply (Rplus_lt_compat_l 1 0 (sqrt 5) sqrt5_pos).
  }
  assert (Hn : 0 < 1 + sqrt 5) by lra.
  replace ((1 + sqrt 5) / 2) with ((1 + sqrt 5) * / 2) by field.
  apply Rmult_lt_0_compat.
  - exact Hn.
  - apply Rinv_0_lt_compat; lra.
Qed.

Lemma phi_neq_0 : phi <> 0.
Proof. apply Rgt_not_eq, Rlt_gt; exact phi_pos. Qed.

Lemma phi_gt_1 : 1 < phi.
Proof.
  unfold phi.
  assert (H : 2 < 1 + sqrt 5) by (generalize sqrt5_gt_2; lra).
  lra.
Qed.

Lemma sqrt9 : sqrt 9 = 3.
Proof.
  replace 9 with (3 * 3) by ring.
  rewrite sqrt_square; lra.
Qed.

Lemma phi_lt_2 : phi < 2.
Proof.
  unfold phi.
  assert (H : 1 + sqrt 5 < 4).
  {
    assert (sqrt 5 < 3) by (rewrite <- sqrt9; apply sqrt_lt_1; lra).
    lra.
  }
  lra.
Qed.

Lemma phi_squared_identity : phi * phi = phi + 1.
Proof.
  unfold phi.
  assert (Hsq : (1 + sqrt 5) * (1 + sqrt 5) = 6 + 2 * sqrt 5).
  {
    replace ((1 + sqrt 5) * (1 + sqrt 5)) with (1 + 2 * sqrt 5 + (sqrt 5 * sqrt 5)) by ring.
    rewrite sqrt5_sq.
    ring.
  }
  assert (Hleft : ((1 + sqrt 5) / 2) * ((1 + sqrt 5) / 2) = (6 + 2 * sqrt 5) / 4).
  {
    replace (((1 + sqrt 5) / 2) * ((1 + sqrt 5) / 2)) with (((1 + sqrt 5) * (1 + sqrt 5)) / 4) by field.
    rewrite Hsq.
    reflexivity.
  }
  assert (Hright : (1 + sqrt 5) / 2 + 1 = (3 + sqrt 5) / 2) by field.
  assert (Hmid : (6 + 2 * sqrt 5) / 4 = (3 + sqrt 5) / 2) by field.
  rewrite Hleft, Hright.
  exact Hmid.
Qed.

Lemma phi_mul_phi_minus_one : phi * (phi - 1) = 1.
Proof.
  replace (phi * (phi - 1)) with (phi * phi - phi) by ring.
  rewrite phi_squared_identity.
  ring.
Qed.

Lemma phi_inv_is_phi_minus_one : / phi = phi - 1.
Proof.
  apply (Rmult_eq_reg_l phi).
  - rewrite Rinv_r; [| exact phi_neq_0 ].
    symmetry.
    exact phi_mul_phi_minus_one.
  - exact phi_neq_0.
Qed.

Lemma phi_inv_sq_sum_three : phi * phi + Rsqr (/ phi) = 3.
Proof.
  rewrite phi_inv_is_phi_minus_one.
  unfold Rsqr.
  assert (Hsq1 : (phi - 1) * (phi - 1) = 2 - phi).
  {
    replace ((phi - 1) * (phi - 1)) with (phi * phi - 2 * phi + 1) by ring.
    rewrite phi_squared_identity.
    ring.
  }
  rewrite Hsq1.
  rewrite phi_squared_identity.
  ring.
Qed.

Lemma phi_tolerance_pos : 0 < phi_tolerance.
Proof.
  unfold phi_tolerance.
  apply Rmult_lt_0_compat.
  - apply Rmult_lt_0_compat.
    + lra.
    + apply Rinv_0_lt_compat.
      apply IZR_lt.
      exact coeff_53_pos.
  - apply Rmult_lt_0_compat.
    + exact phi_pos.
    + exact phi_pos.
Qed.

(** [PHI-IDENTITY] on [R]: no float error — residual is zero. *)
Lemma phi_identity_residual_R : Rabs (phi * phi - (phi + 1)) = 0.
Proof.
  replace (phi * phi - (phi + 1)) with 0.
  - apply Rabs_R0.
  - unfold Rminus.
    rewrite phi_squared_identity.
    ring.
Qed.

Definition phi_approx_valid (x : R) : Prop :=
  Rabs (x - phi) <= phi_tolerance.
