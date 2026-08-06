/-
  Trinity.ExactIdentities - Lucas, Fibonacci, and Pell Numbers via Phi
  Lean 4 / Mathlib translation of proofs/trinity/ExactIdentities.v
  Part of Trinity S³AI Lean 4 Bridge (Wave Loop 53)
  Status: MANUAL TRANSLATION - awaiting verification with `lake build`

  Translates exact algebraic identities involving Lucas numbers, Fibonacci numbers,
  and Pell numbers expressed through the golden ratio φ.
-/

import Mathlib
import Trinity.CorePhi

open Real

-- psi = (1 - sqrt(5))/2 = 1 - phi = -1/phi
noncomputable def psi : ℝ := (1 - sqrt 5) / 2

-- Key identity: phi + psi = 1
lemma phi_plus_psi : phi + psi = 1 := by
  rw [phi, psi]
  field_simp
  ring

-- Key identity: phi * psi = -1
lemma phi_times_psi : phi * psi = -1 := by
  rw [phi, psi]
  have h : (sqrt 5 : ℝ) ^ 2 = 5 := sqrt5_sq
  field_simp
  nlinarith

-- Key identity: psi = -1/phi
lemma psi_eq : psi = -1 / phi := by
  have h1 : phi ≠ 0 := phi_nonzero
  field_simp
  nlinarith [phi_times_psi]

-- Key identity: 1/phi = phi - 1 = -psi
lemma inv_phi_psi : 1 / phi = -psi := by
  have h1 : phi ≠ 0 := phi_nonzero
  field_simp
  nlinarith [phi_times_psi]

-- Lucas number L_n = phi^n + psi^n
noncomputable def lucasPhi (n : ℕ) : ℝ := phi ^ n + psi ^ n

-- L_0 = phi^0 + psi^0 = 1 + 1 = 2
lemma lucasPhi_0 : lucasPhi 0 = 2 := by
  rw [lucasPhi]
  norm_num

-- L_1 = phi + psi = 1
lemma lucasPhi_1 : lucasPhi 1 = 1 := by
  rw [lucasPhi]
  simp [phi_plus_psi]

-- Key identity: psi^2 = psi + 1 (both phi and psi are roots of x^2 - x - 1 = 0)
lemma psi_quadratic : psi ^ 2 = psi + 1 := by
  rw [psi]
  have h : (sqrt 5 : ℝ) ^ 2 = 5 := sqrt5_sq
  field_simp
  nlinarith

-- L_2 = phi^2 + psi^2 = (phi+1) + (2-phi) = 3
lemma lucasPhi_2 : lucasPhi 2 = 3 := by
  rw [lucasPhi]
  have hphi : phi ^ 2 = phi + 1 := phi_square
  have hpsi : psi ^ 2 = 2 - phi := by
    have h1 : psi ^ 2 = psi + 1 := psi_quadratic
    have h2 : phi + psi = 1 := phi_plus_psi
    nlinarith
  norm_num [hphi, hpsi]
  ring

-- L_3 = phi^3 + psi^3 = 4
lemma lucasPhi_3 : lucasPhi 3 = 4 := by
  rw [lucasPhi]
  have hphi : phi ^ 3 = 2 * phi + 1 := phi_cubed
  have hpsi : psi ^ 3 = 3 - 2 * phi := by
    calc
      psi ^ 3 = psi * psi ^ 2 := by ring
      _ = psi * (psi + 1) := by rw [psi_quadratic]
      _ = psi ^ 2 + psi := by ring
      _ = (psi + 1) + psi := by rw [psi_quadratic]
      _ = 2 * psi + 1 := by ring
      _ = 2 * (1 - phi) + 1 := by rw [show psi = 1 - phi by linarith [phi_plus_psi]]
      _ = 3 - 2 * phi := by ring
  norm_num [hphi, hpsi]
  ring

-- L_4 = phi^4 + psi^4 = 7
lemma lucasPhi_4 : lucasPhi 4 = 7 := by
  rw [lucasPhi]
  have hphi : phi ^ 4 = 3 * phi + 2 := phi_fourth
  have hpsi : psi ^ 4 = 5 - 3 * phi := by
    calc
      psi ^ 4 = (psi ^ 2) ^ 2 := by ring
      _ = (psi + 1) ^ 2 := by rw [psi_quadratic]
      _ = psi ^ 2 + 2 * psi + 1 := by ring
      _ = (psi + 1) + 2 * psi + 1 := by rw [psi_quadratic]
      _ = 3 * psi + 2 := by ring
      _ = 3 * (1 - phi) + 2 := by rw [show psi = 1 - phi by linarith [phi_plus_psi]]
      _ = 5 - 3 * phi := by ring
  norm_num [hphi, hpsi]
  ring

-- Main recurrence theorem: L_{n+2} = L_{n+1} + L_n
lemma lucas_recurrence (n : ℕ) : lucasPhi (n + 2) = lucasPhi (n + 1) + lucasPhi n := by
  rw [lucasPhi]
  have hphi : phi ^ (n + 2) = phi ^ (n + 1) + phi ^ n := by
    calc
      phi ^ (n + 2) = phi ^ 2 * phi ^ n := by rw [← pow_add]; ring_nf
      _ = (phi + 1) * phi ^ n := by rw [phi_square]
      _ = phi ^ (n + 1) + phi ^ n := by ring
  have hpsi : psi ^ (n + 2) = psi ^ (n + 1) + psi ^ n := by
    calc
      psi ^ (n + 2) = psi ^ 2 * psi ^ n := by rw [← pow_add]; ring_nf
      _ = (psi + 1) * psi ^ n := by rw [psi_quadratic]
      _ = psi ^ (n + 1) + psi ^ n := by ring
  simp [lucasPhi, hphi, hpsi]
  ring

-- Lucas standard integer definition via recurrence
def lucasStdPair (n : ℕ) : ℤ × ℤ :=
  match n with
  | 0 => (2, 1)
  | n' + 1 =>
      let (a, b) := lucasStdPair n'
      (b, a + b)

def lucasStd (n : ℕ) : ℤ := (lucasStdPair n).1

-- Base cases verified by computation
lemma lucasStd_0 : lucasStd 0 = 2 := by
  simp [lucasStd, lucasStdPair]

lemma lucasStd_1 : lucasStd 1 = 1 := by
  simp [lucasStd, lucasStdPair]

lemma lucasStd_2 : lucasStd 2 = 3 := by
  simp [lucasStd, lucasStdPair]

lemma lucasStd_3 : lucasStd 3 = 4 := by
  simp [lucasStd, lucasStdPair]

lemma lucasStd_4 : lucasStd 4 = 7 := by
  simp [lucasStd, lucasStdPair]

-- For even n: phi^(2n) + 1/phi^(2n) = L_{2n}
lemma lucasPhi_inv_even (n : ℕ) :
    lucasPhi (2 * n) = phi ^ (2 * n) + 1 / (phi ^ (2 * n)) := by
  rw [lucasPhi]
  have h : psi ^ (2 * n) = 1 / (phi ^ (2 * n)) := by
    have h1 : psi = -1 / phi := psi_eq
    have h2 : psi ^ (2 * n) = ((-1 / phi) ^ 2) ^ n := by
      rw [h1]
      rw [show (2 * n : ℕ) = 2 * n by rfl]
      rw [← pow_mul]
    have h3 : (-1 / phi : ℝ) ^ 2 = 1 / phi ^ 2 := by
      field_simp [phi_nonzero]
    rw [h2, h3]
    have h4 : (1 / phi ^ 2 : ℝ) ^ n = 1 / (phi ^ (2 * n)) := by
      have h5 : (1 / phi ^ 2 : ℝ) = (1 / phi) ^ 2 := by
        field_simp [phi_nonzero]
      rw [h5]
      rw [← pow_mul]
      have h6 : (1 / phi : ℝ) ^ (2 * n) = 1 / (phi ^ (2 * n)) := by
        field_simp [phi_nonzero]
        rw [← mul_pow]
        field_simp [phi_nonzero]
        norm_num
      exact h6
    exact h4
  rw [h]

-- Specific case: phi^0 + 1/phi^0 = 2 = L_0
lemma phi_inv_power_0 : phi ^ 0 + 1 / (phi ^ 0) = 2 := by
  norm_num

-- Specific case: phi^2 + 1/phi^2 = 3 = L_2
lemma phi_inv_power_2 : phi ^ 2 + 1 / (phi ^ 2) = 3 := by
  rw [← lucasPhi_inv_even 1]
  exact lucasPhi_2

-- Specific case: phi^4 + 1/phi^4 = 7 = L_4
lemma phi_inv_power_4 : phi ^ 4 + 1 / (phi ^ 4) = 7 := by
  rw [← lucasPhi_inv_even 2]
  exact lucasPhi_4
