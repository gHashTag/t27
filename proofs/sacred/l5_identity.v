From Stdlib Require Import Reals.

Open Scope R_scope.

Definition phi : R := ((1 + sqrt 5) / 2).

Theorem trinity_identity : phi ^ 2 + (1 / phi) ^ 2 = 3.
Proof.
  unfold phi.
  admit.
Admitted.
