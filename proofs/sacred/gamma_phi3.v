From Stdlib Require Import Reals.

Open Scope R_scope.

Definition phi : R := ((1 + sqrt 5) / 2).
Definition gamma_phi : R := 1 / (phi ^ 3).

Theorem gamma_phi_is_sqrt5_minus_2 : gamma_phi = sqrt 5 - 2.
Proof.
  unfold gamma_phi, phi.
  admit.
Admitted.
