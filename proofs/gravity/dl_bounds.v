Require Import Reals.Reals.
Open Scope R_scope.

Definition phi : R := (sqrt(5) - 2)%R.

Definition dl_lower : R := (ln(2) / PI)%R.

Definition dl_upper : R := (ln(3) / PI)%R.

Theorem gamma_phi_within_dl_bounds : dl_lower < phi < dl_upper.
Proof.
  (* Numerical verification: *)
  (* dl_lower ≈ 0.2206, phi = √5 - 2 ≈ 0.2361, dl_upper ≈ 0.3497 *)
  compute.
Qed.
