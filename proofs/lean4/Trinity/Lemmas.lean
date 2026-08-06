/- SPDX-License-Identifier: Apache-2.0
   Trinity/Lemmas.lean
   Custom lemma library for ternary MAC algebraic identities.
   Reduces simp expansion overhead for deep accumulation theorems.
   phi^2 + 1/phi^2 = 3 | TRINITY -/

import Trinity.TernaryMac

/-- Associativity lemma: two consecutive plus-weight MACs collapse to a single MAC
    with summed activation. Reduces simp expansion overhead for deep accumulation proofs.
    For any accumulator acc and activations a, b:
    mac(mac(acc, a, .plus), b, .plus) = mac(acc, a + b, .plus). -/
theorem ternaryMac_plus_assoc (acc a b : Int) :
    ternaryMac (ternaryMac acc a (TernaryWeight.mk .plus)) b (TernaryWeight.mk .plus) =
    ternaryMac acc (a + b) (TernaryWeight.mk .plus) := by
  simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]
  <;> try omega

/-- Associativity lemma: two consecutive minus-weight MACs collapse to a single MAC
    with summed activation (negated result).
    For any accumulator acc and activations a, b:
    mac(mac(acc, a, .minus), b, .minus) = mac(acc, a + b, .minus). -/
theorem ternaryMac_minus_assoc (acc a b : Int) :
    ternaryMac (ternaryMac acc a (TernaryWeight.mk .minus)) b (TernaryWeight.mk .minus) =
    ternaryMac acc (a + b) (TernaryWeight.mk .minus) := by
  simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]
  <;> try omega

/-- Collapse lemma: plus-weight followed by minus-weight MAC collapses to a single MAC
    with difference of activations.
    For any accumulator acc and activations a, b:
    mac(mac(acc, a, .plus), b, .minus) = mac(acc, a - b, .plus). -/
theorem ternaryMac_mixed_collapse (acc a b : Int) :
    ternaryMac (ternaryMac acc a (TernaryWeight.mk .plus)) b (TernaryWeight.mk .minus) =
    ternaryMac acc (a - b) (TernaryWeight.mk .plus) := by
  simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]
  <;> try omega
