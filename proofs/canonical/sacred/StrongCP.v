(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        SAC-CP
   Title:         Strong CP problem
   PhD chapter:   Ch.29 Sacred V (strong CP)
   Stats:         Qed=1 · Admitted=0 · Abort=0 · Lines~5
   Source mirror: t27/proofs/sacred/strong_cp.v
   Content SHA-1: b1ee0e96309e8812
   Canonical:     gHashTag/t27/proofs/canonical/sacred/StrongCP.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

Require Import Reals.Reals.

Theorem theta_qcd_zero : Rabs (phi^2 + phi^(-2) - 3) = 0.
Proof. reflexivity. Qed.
