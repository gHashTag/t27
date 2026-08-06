/-
  Trinity.CorePhi - Exact Algebraic Identities for Phi
  Lean 4 / Mathlib translation of proofs/trinity/CorePhi.v
  Part of Trinity S³AI Lean 4 Bridge (Wave Loop 53)
  Status: MANUAL TRANSLATION - awaiting verification with `lake build`

  Tactic mapping (Coq → Lean 4):
    ring        → ring
    field       → field_simp
    lra         → linarith
    nra         → nlinarith
    rewrite     → rw
    reflexivity → rfl
    unfold pow  → simp [pow_succ, pow_zero]
    field_simplify_eq → field_simp [ne_of_gt phi_pos]
-/

import Mathlib

noncomputable def phi : ℝ := (1 + Real.sqrt 5) / 2

lemma phi_pos : phi > 0 := by
  have h1 : Real.sqrt 5 > 0 := Real.sqrt_pos.mpr (show (5 : ℝ) > 0 by norm_num)
  have h2 : 1 + Real.sqrt 5 > 0 := by linarith
  rw [phi]
  linarith

lemma phi_nonzero : phi ≠ 0 := by
  linarith [phi_pos]

lemma sqrt5_sq : (Real.sqrt 5 : ℝ) ^ 2 = 5 := Real.sq_sqrt (by norm_num)

lemma sqrt5_cubed : (Real.sqrt 5 : ℝ) ^ 3 = 5 * Real.sqrt 5 := by
  calc
    (Real.sqrt 5 : ℝ) ^ 3 = (Real.sqrt 5) ^ 2 * Real.sqrt 5 := by ring
    _ = 5 * Real.sqrt 5 := by rw [sqrt5_sq]

lemma phi_quadratic : phi ^ 2 = phi + 1 := by
  rw [phi]
  have h : Real.sqrt 5 ^ 2 = 5 := sqrt5_sq
  field_simp
  nlinarith [Real.sqrt_pos.mpr (show (5 : ℝ) > 0 by norm_num)]

lemma phi_square : phi ^ 2 = phi + 1 := phi_quadratic

lemma phi_inv : 1 / phi = phi - 1 := by
  have h1 : phi ^ 2 = phi + 1 := phi_quadratic
  have h2 : phi ≠ 0 := phi_nonzero
  field_simp
  nlinarith

lemma phi_inv_sq : 1 / phi ^ 2 = 2 - phi := by
  have h1 : phi ^ 2 = phi + 1 := phi_quadratic
  have h2 : phi ≠ 0 := phi_nonzero
  have h3 : phi ^ 2 ≠ 0 := by
    apply pow_ne_zero 2
    exact phi_nonzero
  field_simp
  nlinarith

lemma trinity_identity : phi ^ 2 + 1 / phi ^ 2 = 3 := by
  rw [phi_inv_sq]
  have h1 : phi ^ 2 = phi + 1 := phi_quadratic
  nlinarith

lemma phi_cubed : phi ^ 3 = 2 * phi + 1 := by
  calc
    phi ^ 3 = phi * phi ^ 2 := by ring
    _ = phi * (phi + 1) := by rw [phi_quadratic]
    _ = phi ^ 2 + phi := by ring
    _ = (phi + 1) + phi := by rw [phi_quadratic]
    _ = 2 * phi + 1 := by ring

lemma phi_neg3 : phi ^ (-3 : ℤ) = 2 * phi - 3 := by
  have h1 : phi ≠ 0 := phi_nonzero
  rw [zpow_neg, zpow_ofNat]
  have h2 : phi ^ 3 = 2 * phi + 1 := phi_cubed
  have h3 : phi ^ 3 ≠ 0 := by
    apply pow_ne_zero 3
    exact phi_nonzero
  have h4 : phi ^ 2 = phi + 1 := phi_quadratic
  field_simp [h2]
  -- We need to show: 1 = (2*phi+1)(2*phi-3)
  -- Expand: (2*phi+1)(2*phi-3) = 4*phi^2 - 4*phi - 3
  -- Substitute phi^2 = phi + 1: 4*(phi+1) - 4*phi - 3 = 4*phi + 4 - 4*phi - 3 = 1
  nlinarith [h4, sq_nonneg (phi - 1), sq_nonneg (phi + 1)]

lemma phi_fourth : phi ^ 4 = 3 * phi + 2 := by
  calc
    phi ^ 4 = phi * phi ^ 3 := by ring
    _ = phi * (2 * phi + 1) := by rw [phi_cubed]
    _ = 2 * phi ^ 2 + phi := by ring
    _ = 2 * (phi + 1) + phi := by rw [phi_quadratic]
    _ = 3 * phi + 2 := by ring

lemma phi_fifth : phi ^ 5 = 5 * phi + 3 := by
  calc
    phi ^ 5 = phi * phi ^ 4 := by ring
    _ = phi * (3 * phi + 2) := by rw [phi_fourth]
    _ = 3 * phi ^ 2 + 2 * phi := by ring
    _ = 3 * (phi + 1) + 2 * phi := by rw [phi_quadratic]
    _ = 5 * phi + 3 := by ring

lemma phi_sixth : phi ^ 6 = 8 * phi + 5 := by
  calc
    phi ^ 6 = phi * phi ^ 5 := by ring
    _ = phi * (5 * phi + 3) := by rw [phi_fifth]
    _ = 5 * phi ^ 2 + 3 * phi := by ring
    _ = 5 * (phi + 1) + 3 * phi := by rw [phi_quadratic]
    _ = 8 * phi + 5 := by ring

lemma phi_seventh : phi ^ 7 = 13 * phi + 8 := by
  calc
    phi ^ 7 = phi * phi ^ 6 := by ring
    _ = phi * (8 * phi + 5) := by rw [phi_sixth]
    _ = 8 * phi ^ 2 + 5 * phi := by ring
    _ = 8 * (phi + 1) + 5 * phi := by rw [phi_quadratic]
    _ = 13 * phi + 8 := by ring

lemma phi_eighth : phi ^ 8 = 21 * phi + 13 := by
  calc
    phi ^ 8 = phi * phi ^ 7 := by ring
    _ = phi * (13 * phi + 8) := by rw [phi_seventh]
    _ = 13 * phi ^ 2 + 8 * phi := by ring
    _ = 13 * (phi + 1) + 8 * phi := by rw [phi_quadratic]
    _ = 21 * phi + 13 := by ring