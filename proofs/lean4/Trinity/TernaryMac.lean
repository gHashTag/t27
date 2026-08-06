/- SPDX-License-Identifier: Apache-2.0
   t27/gen/lean/T27Lean/TernaryMac.lean
   Auto-generated from specs/igla/race/ternary_mac.t27 via tri-lean backend.
   Do NOT hand-edit — regenerate via ./scripts/tri-lean.
   phi^2 + 1/phi^2 = 3 | TRINITY -/

/-- Ternary weight code: 0 = zero, 1 = +1, 2 = -1 -/
inductive TernaryWeightCode
  | zero
  | plus
  | minus
deriving Repr, DecidableEq, Inhabited

structure TernaryWeight where
  code : TernaryWeightCode
deriving Repr, DecidableEq, Inhabited

def ternaryDecode (w : TernaryWeight) : Int :=
  match w.code with
  | .plus  => 1
  | .minus => -1
  | .zero  => 0

def ternaryMul (a : Int) (w : TernaryWeight) : Int :=
  let decoded := ternaryDecode w
  match decoded with
  | 0  => 0
  | 1  => a
  | -1 => -a
  | _  => 0

def ternaryMac (acc : Int) (a : Int) (w : TernaryWeight) : Int :=
  let prod := ternaryMul a w
  acc + prod

-- ============================================================================
-- Formal Theorems (extracted from ternary_mac.t27 invariants)
-- ============================================================================

/-- ternary_decode produces only -1, 0, or +1 -/
theorem ternaryDecode_range (w : TernaryWeight) :
  ternaryDecode w = 0 ∨ ternaryDecode w = 1 ∨ ternaryDecode w = -1 := by
  rcases w with ⟨code⟩
  cases code <;> simp [ternaryDecode]

/-- ternary_mul produces only 0, a, or -a (R-SI-1: no '*' operator at RTL) -/
theorem ternaryMul_bounded (a : Int) (w : TernaryWeight) :
  ternaryMul a w = 0 ∨ ternaryMul a w = a ∨ ternaryMul a w = -a := by
  rcases w with ⟨code⟩
  cases code <;> simp [ternaryMul, ternaryDecode]

/-- ternary_mac is associative: mac(acc, a, w) = acc + mul(a, w) -/
theorem ternaryMac_eq_acc_plus_mul (acc : Int) (a : Int) (w : TernaryWeight) :
  ternaryMac acc a w = acc + ternaryMul a w := by
  rfl

/-- ternary_mac with zero weight leaves accumulator unchanged -/
theorem ternaryMac_zero_weight_identity (acc : Int) (a : Int) :
  ternaryMac acc a (TernaryWeight.mk .zero) = acc := by
  simp [ternaryMac, ternaryMul, ternaryDecode]

/-- ternary_mul with zero weight always returns 0 -/
theorem ternaryMul_zero_weight_identity (a : Int) :
  ternaryMul a (TernaryWeight.mk .zero) = 0 := by
  simp [ternaryMul, ternaryDecode]

/-- ternary_mul with +1 weight returns the activation unchanged -/
theorem ternaryMul_plus_weight_identity (a : Int) :
  ternaryMul a (TernaryWeight.mk .plus) = a := by
  simp [ternaryMul, ternaryDecode]

/-- ternary_mul with -1 weight returns the negated activation -/
theorem ternaryMul_minus_weight_identity (a : Int) :
  ternaryMul a (TernaryWeight.mk .minus) = -a := by
  simp [ternaryMul, ternaryDecode]
