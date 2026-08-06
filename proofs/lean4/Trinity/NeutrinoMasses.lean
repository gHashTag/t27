/-
  Trinity.NeutrinoMasses - Positivity and Normal Ordering of Neutrino Masses
  Lean 4 / Mathlib translation of proofs/trinity/NeutrinoMasses.v (Section 4b-4c)
  Part of Trinity S³AI Lean 4 Bridge (Wave Loop 59)
  Status: MANUAL TRANSLATION - structural positivity and ordering theorems
-/

import Mathlib
import Trinity.CorePhi

open Real

-- Physical constants (in GeV, natural units)
def h_H4 : ℝ := 30

def M_Planck : ℝ := 1.2209e19

def v_EW : ℝ := 246.0

def m_electron : ℝ := 0.000511

def m_muon : ℝ := 0.105658

def m_tau : ℝ := 1.77686

-- 600-cell spectral cutoff (postulated identification)
def Lambda_600 : ℝ := M_Planck / (h_H4 * phi)

-- Majorana mass scale (NCG cutoff scale)
def M_R_majorana : ℝ := Lambda_600

-- Light neutrino masses via seesaw
def m_nu_electron : ℝ := m_electron^2 / M_R_majorana

def m_nu_muon : ℝ := m_muon^2 / M_R_majorana

def m_nu_tau : ℝ := m_tau^2 / M_R_majorana

-- Convert to eV
def m_nu_electron_eV : ℝ := m_nu_electron * 1e9

def m_nu_muon_eV : ℝ := m_nu_muon * 1e9

def m_nu_tau_eV : ℝ := m_nu_tau * 1e9

-- Positivity lemmas
lemma h_H4_pos : 0 < h_H4 := by norm_num

lemma M_Planck_pos : 0 < M_Planck := by norm_num

lemma v_EW_pos : 0 < v_EW := by norm_num

lemma m_electron_pos : 0 < m_electron := by norm_num

lemma m_muon_pos : 0 < m_muon := by norm_num

lemma m_tau_pos : 0 < m_tau := by norm_num

lemma Lambda_600_pos : 0 < Lambda_600 := by
  unfold Lambda_600 M_Planck h_H4
  have hphi : 0 < phi := phi_pos
  apply div_pos
  · norm_num
  · nlinarith [hphi]

lemma M_R_majorana_pos : 0 < M_R_majorana := by
  unfold M_R_majorana
  exact Lambda_600_pos

lemma pow2_pos {x : ℝ} (hx : 0 < x) : 0 < x^2 := by
  nlinarith [hx]

lemma m_nu_electron_pos : 0 < m_nu_electron := by
  unfold m_nu_electron
  apply div_pos
  · apply pow2_pos
    exact m_electron_pos
  · exact M_R_majorana_pos

lemma m_nu_muon_pos : 0 < m_nu_muon := by
  unfold m_nu_muon
  apply div_pos
  · apply pow2_pos
    exact m_muon_pos
  · exact M_R_majorana_pos

lemma m_nu_tau_pos : 0 < m_nu_tau := by
  unfold m_nu_tau
  apply div_pos
  · apply pow2_pos
    exact m_tau_pos
  · exact M_R_majorana_pos

lemma m_nu_electron_eV_pos : 0 < m_nu_electron_eV := by
  unfold m_nu_electron_eV
  nlinarith [m_nu_electron_pos]

lemma m_nu_muon_eV_pos : 0 < m_nu_muon_eV := by
  unfold m_nu_muon_eV
  nlinarith [m_nu_muon_pos]

lemma m_nu_tau_eV_pos : 0 < m_nu_tau_eV := by
  unfold m_nu_tau_eV
  nlinarith [m_nu_tau_pos]

-- Strict square monotonicity for positive reals
lemma pow2_pos_lt {a b : ℝ} (ha : 0 < a) (hab : a < b) : a^2 < b^2 := by
  nlinarith [ha, hab]

-- Seesaw ordering monotonicity
lemma seesaw_ordering {a b M_R : ℝ}
    (ha : 0 < a) (hab : a < b) (hM_R : 0 < M_R) :
    a^2 / M_R < b^2 / M_R := by
  apply div_lt_div_of_pos_right
  · exact hM_R
  · apply pow2_pos_lt
    · exact ha
    · exact hab

-- Normal ordering theorem
lemma m_electron_lt_m_muon : m_electron < m_muon := by norm_num

lemma m_muon_lt_m_tau : m_muon < m_tau := by norm_num

theorem neutrino_normal_ordering :
    m_nu_electron < m_nu_muon ∧ m_nu_muon < m_nu_tau := by
  constructor
  · unfold m_nu_electron m_nu_muon
    apply seesaw_ordering
    · exact m_electron_pos
    · exact m_electron_lt_m_muon
    · exact M_R_majorana_pos
  · unfold m_nu_muon m_nu_tau
    apply seesaw_ordering
    · exact m_muon_pos
    · exact m_muon_lt_m_tau
    · exact M_R_majorana_pos

-- Mass-squared differences
noncomputable def Delta_m2_21 : ℝ := m_nu_muon_eV^2 - m_nu_electron_eV^2

def Delta_m2_31 : ℝ := m_nu_tau_eV^2 - m_nu_electron_eV^2

lemma Delta_m2_21_pos : 0 < Delta_m2_21 := by
  unfold Delta_m2_21
  have h1 : m_nu_electron_eV < m_nu_muon_eV := by
    unfold m_nu_electron_eV m_nu_muon_eV
    have h : m_nu_electron < m_nu_muon := neutrino_normal_ordering.1
    nlinarith [h]
  have h2 : 0 < m_nu_muon_eV^2 - m_nu_electron_eV^2 := by
    apply sub_pos_of_lt
    apply pow2_pos_lt
    · exact m_nu_electron_eV_pos
    · exact h1
  linarith

lemma Delta_m2_31_pos : 0 < Delta_m2_31 := by
  unfold Delta_m2_31
  have h1 : m_nu_electron_eV < m_nu_tau_eV := by
    unfold m_nu_electron_eV m_nu_tau_eV
    have h : m_nu_electron < m_nu_tau := by
      linarith [neutrino_normal_ordering.1, neutrino_normal_ordering.2]
    nlinarith [h]
  have h2 : 0 < m_nu_tau_eV^2 - m_nu_electron_eV^2 := by
    apply sub_pos_of_lt
    apply pow2_pos_lt
    · exact m_nu_electron_eV_pos
    · exact h1
  linarith

-- Sum of neutrino masses
noncomputable def Sum_m_nu : ℝ := m_nu_electron_eV + m_nu_muon_eV + m_nu_tau_eV

lemma Sum_m_nu_pos : 0 < Sum_m_nu := by
  unfold Sum_m_nu
  nlinarith [m_nu_electron_eV_pos, m_nu_muon_eV_pos, m_nu_tau_eV_pos]
