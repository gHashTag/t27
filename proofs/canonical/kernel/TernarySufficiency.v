(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        KER-8
   Title:         Ternary sufficiency
   PhD chapter:   Ch.4 Sacred / Ch.27 TRI27
   Stats:         Qed=2 · Admitted=0 · Abort=0 · Lines~10
   Source mirror: t27/coq/Theorems/TernarySufficiency.v
   Content SHA-1: f0528fe18a79ac8f
   Canonical:     gHashTag/t27/proofs/canonical/kernel/TernarySufficiency.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

(** THEOREM-K1 direction — HSLM / linear layer over trit; to be refined with matrix ops. *)

Require Import T27.Kernel.Trit.

Lemma trit_mul_zero_l (a : trit) : trit_mul Zero a = Zero.
Proof. destruct a; reflexivity. Qed.

Lemma trit_mul_zero_r (a : trit) : trit_mul a Zero = Zero.
Proof. destruct a; reflexivity. Qed.
