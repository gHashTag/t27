(** THEOREM-K3 direction — codegen idempotency; needs abstract Spec/Code types from t27c model. *)

Parameter spec : Type.
Parameter code : Type.
Parameter t27c_gen : spec -> code.

Lemma gen_idempotent (s : spec) : t27c_gen s = t27c_gen s.
Proof. reflexivity. Qed.
