(** THEOREM-K2 direction — numeric format ordering via phi-distance; stub until formats are formalized. *)

Require Import Reals.
Open Scope R_scope.

(** Placeholder distance on reals; replace with format-indexed definitions from specs/numeric. *)
Definition phi_distance_stub (x y : R) : R := Rabs (x - y).

Lemma phi_distance_nonneg (x y : R) : 0 <= phi_distance_stub x y.
Proof. apply Rabs_pos. Qed.
