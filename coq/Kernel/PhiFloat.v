(** PHI-IDENTITY — Flocq IEEE 754 binary64 bridge (Phase B).
    Requires [coq-flocq] on COQPATH (CI: opam install coq-flocq; see [../README.md]).
    Mantissas/exponents must match [scripts/validate_phi_f64.py].

    NAME COLLISION — read this before touching the imports.

    Flocq declares TWO distinct inductives, both named [binary_float], with the
    same four constructor names and the same projection names:
      - [Flocq.IEEE754.Binary] — NaN carries a payload
        ([B754_nan (s : bool) (pl : positive) : nan_pl pl = true -> _]);
      - [Flocq.IEEE754.BinarySingleNaN] — one payload-free [B754_nan].

    [Flocq.IEEE754.Bits] closes its generic section BEFORE writing
    [Definition binary64 := binary_float 53 1024], so the [binary_float] there is
    the *Binary* one (Bits imports Core, then BinarySingleNaN, then Binary, and
    the last import wins). The neighbouring operations settle it independently:
    [b64_opp := Bopp 53 1024 unop_nan_pl64] passes a NaN-payload function, which
    only [Binary.Bopp] accepts.

    The old header ended with [Import Binary Bits BinarySingleNaN], so the bare
    names [B2R] and [B754_finite] resolved to BinarySingleNaN's — again, last
    import wins — while [binary64] stayed Binary's. Both print as
    "binary_float 53 1024", so the failure read like a transparent [Definition]
    refusing to delta-unfold, when in fact the two sides were different
    inductives. Every colliding Flocq name below is module-qualified; do not
    re-add that [Import].

    For this [binary64] literal of φ, [fl(phi*phi)] and [fl(phi+1)] coincide —
    bit-identical, both 5895288448088660 * 2^-51 = 0x4004F1BBCDCBFA54, so the
    residual is exactly zero, not merely small. [phi_identity_contract] is
    therefore [Rabs 0 < phi_tolerance], using [phi_tolerance_pos] from [Phi.v].
    A future ring can add [Bmult_correct] / [Bplus_correct] + error bounds for
    other formats (GF16, etc.). *)

From Coq Require Import Reals ZArith SpecFloat.

From Flocq Require Import IEEE754.Binary.
From Flocq Require Import IEEE754.Bits.
From Flocq Require Import IEEE754.BinarySingleNaN.

Require Import T27.Kernel.Phi.

(** [R_scope] is opened LAST, and [Z_scope] is not opened here at all. Every
    statement below whose head notation is [<] or [-] is on [R]; with
    [Open Scope Z_scope] following [Open Scope R_scope] those parsed as [Z.lt]
    and [Z.sub], so [Rabs _ < _] and [0 < phi_tolerance] were type errors
    independently of the float layer. Z-typed *arguments* still parse in
    [Z_scope] through [Bind Scope Z_scope with Z] (Coq.ZArith.BinInt); the
    explicit [%Z] marks below do not depend on that. *)
Open Scope R_scope.

Definition B2R64 : binary64 -> R := Binary.B2R 53%Z 1024%Z.

(** Nearest-round φ as [B754_finite] (see validation script). *)
Definition phi_mantissa : positive := 7286977268806824%positive.
Definition phi_exponent : Z := (-52)%Z.

(** [bounded] is a section-local [Notation] in both [Binary] and
    [BinarySingleNaN] and so is exported by neither; the constant both abbreviate
    is [Coq.Floats.SpecFloat.bounded prec emax]. Naming it directly is what makes
    this statement match the certificate [Binary.B754_finite] demands. *)
Lemma phi_f64_bounded :
  SpecFloat.bounded 53%Z 1024%Z phi_mantissa phi_exponent = true.
Proof. vm_compute. reflexivity. Qed.

Definition phi_f64 : binary64 :=
  Binary.B754_finite 53%Z 1024%Z false phi_mantissa phi_exponent phi_f64_bounded.

(** [1.0] in binary64. *)
Definition one_mantissa : positive := 4503599627370496%positive.
Definition one_exponent : Z := (-52)%Z.

Lemma one_f64_bounded :
  SpecFloat.bounded 53%Z 1024%Z one_mantissa one_exponent = true.
Proof. vm_compute. reflexivity. Qed.

Definition one_f64 : binary64 :=
  Binary.B754_finite 53%Z 1024%Z false one_mantissa one_exponent one_f64_bounded.

Definition phi_sq_f64 : binary64 :=
  b64_mult BinarySingleNaN.mode_NE phi_f64 phi_f64.

Definition phi_plus_one_f64 : binary64 :=
  b64_plus BinarySingleNaN.mode_NE phi_f64 one_f64.

(** Bit-identity, stated on [full_float]. [Binary.B2FF] keeps sign, mantissa and
    exponent and drops the [bounded _ _ = true] certificate that [B754_finite]
    carries, so both sides are closed [full_float]s and [vm_compute] decides it.

    Stating it directly as [phi_sq_f64 = phi_plus_one_f64] is NOT provable by
    [vm_compute; reflexivity]: Flocq derives the certificate of a product from
    [Bmult_correct_aux] and that of a sum from [binary_round_correct], both
    [Qed]-opaque, so the two terms carry identical data and non-convertible proof
    components — and [Prop] has no definitional proof irrelevance. Hence the
    detour through [Binary.B2FF_inj], whose entire content is exactly that
    certificate-erasure step ([eqbool_irrelevance]). *)
Lemma phi_sq_f64_ff_eq_phi_plus_one_f64_ff :
  Binary.B2FF 53%Z 1024%Z phi_sq_f64 = Binary.B2FF 53%Z 1024%Z phi_plus_one_f64.
Proof. vm_compute. reflexivity. Qed.

Lemma phi_sq_f64_eq_phi_plus_one_f64 : phi_sq_f64 = phi_plus_one_f64.
Proof.
  apply Binary.B2FF_inj.
  exact phi_sq_f64_ff_eq_phi_plus_one_f64_ff.
Qed.

(** Engineering bound (Layer A SSOT): [5 * 2^-53 * phi^2] on [R]. *)
Definition PHI_F64_TOLERANCE : R := phi_tolerance.

Theorem phi_identity_contract :
  Rabs (B2R64 phi_sq_f64 - B2R64 phi_plus_one_f64) < PHI_F64_TOLERANCE.
Proof.
  assert (Hbr : B2R64 phi_sq_f64 = B2R64 phi_plus_one_f64).
  {
    apply f_equal.
    exact phi_sq_f64_eq_phi_plus_one_f64.
  }
  unfold PHI_F64_TOLERANCE.
  replace (B2R64 phi_sq_f64 - B2R64 phi_plus_one_f64) with 0.
  - rewrite Rabs_R0.
    apply phi_tolerance_pos.
  - rewrite Hbr.
    ring.
Qed.

Lemma phi_tolerance_positive : 0 < phi_tolerance.
Proof. apply phi_tolerance_pos. Qed.

Lemma PHI_F64_TOLERANCE_pos : 0 < PHI_F64_TOLERANCE.
Proof. unfold PHI_F64_TOLERANCE; apply phi_tolerance_pos. Qed.
