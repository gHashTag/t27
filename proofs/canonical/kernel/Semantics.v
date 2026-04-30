(* SPDX-License-Identifier: Apache-2.0 *)
(* ================================================================
   CANONICAL — Trinity Coq Single Source of Truth
   Bundle:        KER-4
   Title:         TRI-27 ISA semantics
   PhD chapter:   Ch.27 TRI27 DSL
   Stats:         Qed=1 · Admitted=0 · Abort=0 · Lines~25
   Source mirror: t27/coq/Kernel/Semantics.v
   Content SHA-1: f130475c5a5dbbe4
   Canonical:     gHashTag/t27/proofs/canonical/kernel/Semantics.v
   Anchor:        phi^2 + phi^-2 = 3 (CorePhi.trinity_identity)
   Census:        github.com/gHashTag/trios/issues/373#issuecomment-4351659821
   ================================================================ *)

(** Minimal expression calculus — placeholder for denotational / RT story (AXIOM-K3 direction). *)

Require Import T27.Kernel.Trit.

Definition env : Type := nat -> option trit.

Inductive expr : Set :=
  | ELit : trit -> expr
  | EVar : nat -> expr.

Fixpoint eval (e : expr) (rho : env) : option trit :=
  match e with
  | ELit t => Some t
  | EVar n => rho n
  end.

Lemma eval_det (e : expr) (rho : env) (v1 v2 : trit) :
  eval e rho = Some v1 ->
  eval e rho = Some v2 ->
  v1 = v2.
Proof.
  intros H1 H2.
  congruence.
Qed.
