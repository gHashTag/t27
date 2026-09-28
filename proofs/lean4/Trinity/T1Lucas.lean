/-
  Trinity.T1Lucas — Theorem T1 (Lucas form) and its general family.

  T1.  phi^3 - 4 = phi^(-3)   and   phi^6 - 4*phi^3 = 1,  both EXACTLY.
       Hence the expression (phi^3 - 4)/gamma with gamma := phi^(-3),
       printed as "approx 1" in docs/papers/RIEMANN_GAMMA.tex eq. (line 107),
       is identically 1.  It is x/x, not a coincidence.

  General family:  psi := (1 - sqrt 5)/2 = -1/phi,  L_n := phi^n + psi^n,
       phi^n - L_n = (-1)^(n+1) * phi^(-n),
       phi^(2n) - L_n * phi^n + (-1)^n = 0.
  n = 3 is the odd case; nothing distinguishes it.

  NOTE: this file makes no claim about the Riemann zeta function.
-/

import Mathlib
import Trinity.CorePhi

open Real

namespace Trinity.T1

/-- The conjugate root of `x^2 = x + 1`. -/
noncomputable def psiT : ℝ := (1 - Real.sqrt 5) / 2

lemma phi_mul_psiT : phi * psiT = -1 := by
  rw [phi, psiT]
  have h : (Real.sqrt 5 : ℝ) ^ 2 = 5 := sqrt5_sq
  nlinarith [h]

lemma psiT_eq : psiT = -1 / phi := by
  have h0 : phi ≠ 0 := phi_nonzero
  field_simp
  nlinarith [phi_mul_psiT]

/-- Lucas numbers, defined analytically: `L_n = phi^n + psi^n`. -/
noncomputable def lucasT (n : ℕ) : ℝ := phi ^ n + psiT ^ n

/-! ### T1, the two exact statements -/

/-- **T1(a)**: `phi^3 - 4 = phi^(-3)` exactly. -/
theorem phi_cubed_sub_four : phi ^ 3 - 4 = phi ^ (-3 : ℤ) := by
  rw [phi_neg3, phi_cubed]; ring

/-- **T1(a')**: same statement with a natural-power reciprocal. -/
theorem phi_cubed_sub_four' : phi ^ 3 - 4 = 1 / phi ^ 3 := by
  have h0 : phi ≠ 0 := phi_nonzero
  have h3 : phi ^ 3 = 2 * phi + 1 := phi_cubed
  have h2 : phi ^ 2 = phi + 1 := phi_quadratic
  have hne : phi ^ 3 ≠ 0 := pow_ne_zero 3 h0
  field_simp
  nlinarith [h2, h3]

/-- **T1(b)**: `phi^6 - 4*phi^3 = 1` exactly. -/
theorem phi_six_sub_four_phi_cubed : phi ^ 6 - 4 * phi ^ 3 = 1 := by
  calc phi ^ 6 - 4 * phi ^ 3 = (phi ^ 3) ^ 2 - 4 * phi ^ 3 := by ring
    _ = (2 * phi + 1) ^ 2 - 4 * (2 * phi + 1) := by rw [phi_cubed]
    _ = 4 * phi ^ 2 - 4 * phi - 3 := by ring
    _ = 4 * (phi + 1) - 4 * phi - 3 := by rw [phi_quadratic]
    _ = 1 := by ring

/-- **T1(c)**: the literal expression of RIEMANN_GAMMA.tex is exactly 1,
    because it is `gamma / gamma`. -/
theorem paper_ratio_eq_one : (phi ^ 3 - 4) / (1 / phi ^ 3) = 1 := by
  have h0 : phi ≠ 0 := phi_nonzero
  have hne : phi ^ 3 ≠ 0 := pow_ne_zero 3 h0
  rw [phi_cubed_sub_four']
  field_simp

/-! ### The general family: T1 is the n = 3 member -/

lemma psiT_pow (n : ℕ) : psiT ^ n = (-1) ^ n / phi ^ n := by
  rw [psiT_eq, div_pow]

/-- `phi^n - L_n = (-1)^(n+1) * phi^(-n)`.  For odd `n` the right side is `+phi^(-n)`;
    `n = 3`, `L_3 = 4` is one instance of infinitely many. -/
theorem phi_pow_sub_lucas (n : ℕ) :
    phi ^ n - lucasT n = (-1) ^ (n + 1) / phi ^ n := by
  rw [lucasT, psiT_pow]
  ring

/-- `phi^n` is a root of `x^2 - L_n x + (-1)^n`. -/
theorem phi_pow_quadratic (n : ℕ) :
    phi ^ (2 * n) - lucasT n * phi ^ n + (-1) ^ n = 0 := by
  have h0 : phi ≠ 0 := phi_nonzero
  have hne : phi ^ n ≠ 0 := pow_ne_zero n h0
  rw [lucasT, psiT_pow]
  field_simp
  ring

/-- `L_3 = 4`, so T1(a) is the `n = 3` instance of `phi_pow_sub_lucas`. -/
theorem lucasT_three : lucasT 3 = 4 := by
  have h5 : (Real.sqrt 5 : ℝ) ^ 2 = 5 := sqrt5_sq
  rw [lucasT, phi, psiT]
  nlinarith [h5]

/-- `L_2 = 3`: the neighbouring "TRINITY identity" of the same paper,
    equally exact, equally uninformative about zeta. -/
theorem lucasT_two : lucasT 2 = 3 := by
  have h5 : (Real.sqrt 5 : ℝ) ^ 2 = 5 := sqrt5_sq
  rw [lucasT, phi, psiT]
  nlinarith [h5]

end Trinity.T1
