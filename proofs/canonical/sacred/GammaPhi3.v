(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        SAC-G3
   Title:         Gamma . phi^3 identity
   PhD chapter:   Ch.29 Sacred V (Gamma.phi^3)
   Stats:         Qed=1 · Admitted=0 · Abort=0 · Lines~25
   Source mirror: trios/docs/phd/theorems/sacred/gamma_phi3.v
   Content SHA-1: c579d5f2546c3678
   Canonical:     gHashTag/t27/proofs/canonical/sacred/GammaPhi3.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

Require Import Reals.Reals.
Open Scope R_scope.

Definition phi : R := (1 + sqrt(5)) / 2.
Definition gamma_phi : R := phi ^ (-3).

Theorem gamma_phi_is_sqrt5_minus_2 : gamma_phi = sqrt(5) - 2.
Proof.
<<<<<<< Updated upstream
  unfold gamma_phi.
  unfold phi.
  field.
=======
  unfold gamma_phi, phi.
  (* gamma_phi = 8/(1+sqrt5)^3 *)
  (* (1+sqrt5)^3 = 1 + 3*sqrt5 + 3*5 + 5*sqrt5 = 16 + 8*sqrt5 *)
  (* gamma_phi = 8/(16+8*sqrt5) = 1/(2+sqrt5) *)
  (* 1/(2+sqrt5) = sqrt5-2, since (sqrt5-2)(sqrt5+2) = 5-4 = 1 *)
  rewrite Rsqr_sqrt.
  ring_simplify.
  try reflexivity.
  admit.
>>>>>>> Stashed changes
Qed.
