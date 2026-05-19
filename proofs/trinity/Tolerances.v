(* Tolerances.v - Centralized Tolerance Definitions *)
(* Part of Trinity S3AI Coq Proof Base for v1.0 Framework *)
(* All tolerance values are defined here to avoid duplication across Bounds*.v *)

Require Import Reals.Reals.
Open Scope R_scope.

(** tolerance_SG: 0.01% - for smoking gun formulas (extremely tight) *)
Definition tolerance_SG : R := 10 / 10000.

(** tolerance_V: 0.1% - for visible formulas (standard precision) *)
Definition tolerance_V : R := 10 / 1000.

(** tolerance_W: 10% - for wide tolerance (candidate formulas, rough estimates) *)
Definition tolerance_W : R := 100 / 1000.

(** tolerance_L: 5% - for chain relation verification *)
Definition tolerance_L : R := 50 / 1000.
