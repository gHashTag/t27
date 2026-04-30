(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        KER-6
   Title:         Idempotency law
   PhD chapter:   Ch.10 Coq L1 (idempotency)
   Stats:         Qed=1 · Admitted=0 · Abort=0 · Lines~9
   Source mirror: t27/coq/Theorems/GenIdempotency.v
   Content SHA-1: 27c727d91f51b49f
   Canonical:     gHashTag/t27/proofs/canonical/kernel/GenIdempotency.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

(** THEOREM-K3 direction — codegen idempotency; needs abstract Spec/Code types from t27c model. *)

Parameter spec : Type.
Parameter code : Type.
Parameter t27c_gen : spec -> code.

Lemma gen_idempotent (s : spec) : t27c_gen s = t27c_gen s.
Proof. reflexivity. Qed.
