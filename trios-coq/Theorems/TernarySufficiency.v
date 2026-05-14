(** THEOREM-K1 direction — HSLM / linear layer over trit; to be refined with matrix ops. *)

Require Import T27.Kernel.Trit.

Lemma trit_mul_zero_l (a : trit) : trit_mul Zero a = Zero.
Proof. destruct a; reflexivity. Qed.

Lemma trit_mul_zero_r (a : trit) : trit_mul a Zero = Zero.
Proof. destruct a; reflexivity. Qed.
