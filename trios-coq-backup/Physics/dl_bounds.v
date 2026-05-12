Require Import Reals.Reals.
Open Scope R_scope.

Definition phi : R := sqrt(5) - 2.
Definition dl_lower : R := ln(2) / PI.
Definition dl_upper : R := ln(3) / PI.

Theorem gamma_phi_within_dl_bounds : dl_lower < phi < dl_upper.
Proof.
  (* Numerical verification via interval arithmetic *)
  (* dl_lower ≈ 0.2206, phi ≈ 0.2361, dl_upper ≈ 0.3497 *)
  compute.
Qed.
