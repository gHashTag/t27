/- SPDX-License-Identifier: Apache-2.0
   t27/proofs/lean4/Trinity/TernaryGemm.lean
   Auto-generated from specs/igla/race/ternary_gemm.t27 via tri-lean backend.
   Do NOT hand-edit — regenerate via ./scripts/tri-lean.
   phi^2 + 1/phi^2 = 3 | TRINITY -/

import Trinity.TernaryMac

/-- Flattened 2x2 ternary GEMM result -/
structure TernaryGemmResult where
  data : Array Int
deriving Repr, DecidableEq

/-- 2x2 ternary GEMM: C = A * W where A is i8 activations and W is ternary weights.
    Arrays are flattened row-major: [a00, a01, a10, a11] and [w00, w01, w10, w11].
    Uses ternary_mac (no * operator) -- R-SI-1 compliant. -/
def ternaryGemm2x2 (a : Array Int) (w : Array TernaryWeight) : Array Int :=
  let c00 := ternaryMac (ternaryMac 0 (a[0]!) (w[0]!)) (a[1]!) (w[2]!)
  let c01 := ternaryMac (ternaryMac 0 (a[0]!) (w[1]!)) (a[1]!) (w[3]!)
  let c10 := ternaryMac (ternaryMac 0 (a[2]!) (w[0]!)) (a[3]!) (w[2]!)
  let c11 := ternaryMac (ternaryMac 0 (a[2]!) (w[1]!)) (a[3]!) (w[3]!)
  #[c00, c01, c10, c11]

/-- Reference scalar multiply-add for verifying ternary GEMM correctness -/
def referenceMulAdd (a : Int) (w : TernaryWeight) (acc : Int) : Int :=
  acc + a * (ternaryDecode w)

/-- Reference 2x2 GEMM using standard integer arithmetic.
    Nesting matches ternaryMac left-to-right accumulation:
    c00 = ((0 + a00*w00) + a01*w10) etc. -/
def referenceGemm2x2 (a : Array Int) (w : Array TernaryWeight) : Array Int :=
  let c00 := referenceMulAdd (a[1]!) (w[2]!) (referenceMulAdd (a[0]!) (w[0]!) 0)
  let c01 := referenceMulAdd (a[1]!) (w[3]!) (referenceMulAdd (a[0]!) (w[1]!) 0)
  let c10 := referenceMulAdd (a[3]!) (w[2]!) (referenceMulAdd (a[2]!) (w[0]!) 0)
  let c11 := referenceMulAdd (a[3]!) (w[3]!) (referenceMulAdd (a[2]!) (w[1]!) 0)
  #[c00, c01, c10, c11]

-- ============================================================================
-- Helper: prove ternaryMul equals a * decode(w)
-- ============================================================================

/-- ternaryMul computes the same result as a * decode(w) -/
theorem ternaryMul_eq_mul_decode (a : Int) (w : TernaryWeight) :
  ternaryMul a w = a * (ternaryDecode w) := by
  rcases w with ⟨code⟩
  cases code <;> simp [ternaryMul, ternaryDecode]

/-- ternaryMac computes the same result as referenceMulAdd -/
theorem ternaryMac_eq_referenceMulAdd (acc : Int) (a : Int) (w : TernaryWeight) :
  ternaryMac acc a w = referenceMulAdd a w acc := by
  simp [ternaryMac, referenceMulAdd, ternaryMul_eq_mul_decode]

-- ============================================================================
-- Formal Theorems
-- ============================================================================

/-- ternary_gemm_2x2 produces exactly 4 elements -/
theorem ternaryGemm2x2_length (a : Array Int) (w : Array TernaryWeight) :
    (ternaryGemm2x2 a w).size = 4 := by
  simp [ternaryGemm2x2]

/-- ternary_gemm_2x2 is equivalent to reference GEMM for all valid inputs.
    This is the key correctness theorem: the ternary-MAC-based implementation
    (no '*' operator) computes the same result as standard integer arithmetic. -/
theorem ternaryGemm2x2_equiv_reference (a : Array Int) (w : Array TernaryWeight)
    (_ha : a.size = 4) (_hw : w.size = 4) :
    ternaryGemm2x2 a w = referenceGemm2x2 a w := by
  simp [ternaryGemm2x2, referenceGemm2x2, ternaryMac_eq_referenceMulAdd]
