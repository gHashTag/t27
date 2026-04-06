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
