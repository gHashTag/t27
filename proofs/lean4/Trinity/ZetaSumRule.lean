/-
  Trinity.ZetaSumRule — sum-rule and number-variance rigidity for one-parameter
  deformations of Montgomery's pair correlation.

  WHAT THIS IS AND IS NOT.

  This module says NOTHING about the Riemann hypothesis. It contains no zeta
  function, no zeros, no L-functions. It formalises the small piece of algebra
  that the corpus's refutation of `R_phi(u) = 1 - phi^(-3) * sinc^2(u)` rests
  on, so that the refutation stops being prose.

  The refutation has three legs and only the third was ever in doubt:

    1. ALGEBRA (already machine-checked in `Trinity.T1Lucas`):
       `phi^3 - 4 = phi^(-3)` is the `n = 3` Lucas identity, a theorem about
       the golden ratio that claims nothing about zeta. Not repeated here.

    2. THE SUM RULE (this file). For any pair correlation of the shape
       `R(u) = 1 - c * K(u)` with `K` of unit mass, the structure factor at
       zero wavenumber is `S(0) = 1 - c`. Vanishing rigidity therefore forces
       `c = 1` — INDEPENDENTLY OF THE SHAPE OF K. That shape-independence is
       the point: the failure is not about `phi`, it is about the coefficient,
       and any constant other than 1 dies the same death. This is layer 2 in
       the matched-control taxonomy: "invariant and false".

    3. THE INPUT (not formalised, and deliberately so). That the zeros' number
       variance really is `o(L)` is Fujii 1975, an unconditional theorem of
       analytic number theory reached at the primary source but far outside
       what is formalised here. It enters below only as a HYPOTHESIS. Nothing
       in this file asserts it.

  So the file proves an implication, not a fact about zeros: IF the variance
  is bounded THEN c = 1. Keeping the analytic input as an explicit hypothesis
  is the whole discipline — a formalisation that quietly assumed Fujii would
  be a stronger-looking and worthless artefact.

  WHAT CHANGED 2026-08-13 (Track C). The `c`-dependence of the number variance
  used to be asserted in a docstring above `sigmaC`: the comment said the
  identity `Σ²(L) = L + 2∫₀^L (L-u)(R(u)-1) du` is linear in `c` and therefore
  `Σ²_c(L) = (1-c)L + c·G(L)`. A comment is not a proof. That step is now
  `V_scaled`, a theorem over mathlib's `intervalIntegral`, and `sigmaC` is a
  consequence of it via `V_eq_sigmaC` rather than a definition standing in for
  one. The functional `V` itself is still a DEFINITION — the identity relating
  Σ² to the pair correlation is classical input and is not derived from a point
  process here. What was promoted from prose to theorem is the linearity, not
  the identity.

  WHAT CHANGED 2026-08-13 (Track B). The claim that the concrete-`L` lemmas
  `Vaff_pins_weight` / `Vaff_pins_weight_two_kernels` pin the weight `2(L - u)`
  was DEMOTED by adversarial verification: they pin it only inside the affine
  family `a(L - u) + b`, and the quadratic weight `2(L - u) + u² - 2u + 2/3`
  passes all three of their witnesses while being a different functional. The
  final section closes that gap against a general `w : ℝ → ℝ → ℝ`. Two
  uniqueness theorems, with the test class named in each:
  `Vw_pins_weight_ae` (all measurable indicator kernels, one length, conclusion
  almost everywhere) and `Vw_pins_weight_continuous` (the one-parameter family of
  step kernels, plus continuity, conclusion pointwise on the open window). The
  counter-model is written down as `Vq` and formally shut out. The quantifier
  structure of each theorem is spelled out in its own docstring in plain words,
  because that is exactly what the demoted claim got wrong.

  MEASUREMENT INTERFACE. `measured_bound_forces_c` turns a number produced by
  `~/skills/claim-audit-lab/scripts/zeta_number_variance.py` into a bound on
  `c`. Measured 2026-08-13 on the corpus's own 100,000 Odlyzko zeros:
  `Sigma^2(500) = 0.371 +- 0.060`, so `<= 1/2` holds with room. The theorem
  then gives `c >= 999/1000`, and `phi^(-3) = 0.236` is not in that range.
-/

import Mathlib
import Trinity.CorePhi
import Trinity.T1Lucas

namespace Trinity.ZetaSumRule

/-! ### The sum rule -/

/-- Structure factor at zero wavenumber for `R(u) = 1 - c * K(u)` where `K`
has total mass `m`:  `S(0) = 1 + ∫ (R - 1) = 1 - c * m`. -/
def S0 (c m : ℝ) : ℝ := 1 - c * m

theorem S0_eq_zero_iff (c m : ℝ) : S0 c m = 0 ↔ c * m = 1 := by
  unfold S0; constructor <;> intro h <;> linarith

/-- **Shape-independence.** For a unit-mass kernel the sum rule fixes the
coefficient and says nothing whatever about the kernel. Every `c ≠ 1` fails
identically, so the failure is not a fact about the golden ratio. -/
theorem sumRule_forces_one {c : ℝ} (h : S0 c 1 = 0) : c = 1 := by
  unfold S0 at h; linarith

/-- Only the PRODUCT `c * m` is constrained. A deformation that rescales the
kernel's mass can always be compensated by rescaling `c`, which is why the sum
rule cannot by itself select a shape. -/
theorem sumRule_only_constrains_product {c m : ℝ} (hm : m ≠ 0)
    (h : S0 c m = 0) : c = 1 / m := by
  unfold S0 at h
  field_simp
  linarith

/-! ### The variance functional

`V R L` is the number variance a pair correlation `R` produces in a window of
length `L`, through the standard identity

    Σ²(L) = L + 2 ∫₀^L (L - u) (R(u) - 1) du.

That identity is classical INPUT and is a definition here, not a theorem: it is
how the functional is defined, and this file does not derive it from a point
process. What is proved below is everything downstream of it — in particular
that the `c`-dependence of `R(u) = 1 - c K(u)` factors out of the integral
exactly. That step used to live in a prose comment above `sigmaC`; it is now
`V_scaled`, and `sigmaC` is a consequence of it (`V_eq_sigmaC`) rather than an
assertion.
-/

/-- Number variance functional, `V R L = L + 2 ∫₀^L (L - u)(R u - 1) du`. -/
noncomputable def V (R : ℝ → ℝ) (L : ℝ) : ℝ :=
  L + 2 * ∫ u in (0:ℝ)..L, (L - u) * (R u - 1)

/-- **Poisson baseline.** `R ≡ 1` gives `Σ²(L) = L`: the integrand vanishes
identically. This is the positive control for the functional — the one case
whose answer is known without any analysis. -/
@[simp] theorem V_poisson (L : ℝ) : V (fun _ => 1) L = L := by
  simp [V]

/-- **The `c`-dependence factors out of the integral.** This is the content the
old comment claimed:

    V(1 - c·K, L) = (1 - c)·L + c·V(1 - K, L).

NO INTEGRABILITY HYPOTHESIS IS REQUIRED, and that is not an oversight. The two
integrands are pointwise proportional — `(L-u)(1 - cK(u) - 1) = c·(L-u)(1 - K(u) - 1)`
— so only scalar homogeneity of the integral is used, and mathlib's
`intervalIntegral.integral_const_mul` is unconditional: it holds even when the
integral is the junk value `0` for a non-integrable integrand, because both
sides are then `0`. An integrability hypothesis would make the statement look
more careful while proving strictly less. The hypothesis IS needed for the
additive form; see `V_mixture`, where it is stated explicitly. -/
theorem V_scaled (c : ℝ) (K : ℝ → ℝ) (L : ℝ) :
    V (fun u => 1 - c * K u) L = (1 - c) * L + c * V (fun u => 1 - K u) L := by
  have h : ∀ u : ℝ, (L - u) * ((1 - c * K u) - 1)
      = c * ((L - u) * ((1 - K u) - 1)) := fun u => by ring
  simp only [V, h, intervalIntegral.integral_const_mul]
  ring

/-- **Affine mixture of pair correlations, with the integrability hypothesis
stated in the open.** For `R = c·R₁ + (1-c)·R₂`,

    V(R, L) = c·V(R₁, L) + (1-c)·V(R₂, L).

Here additivity of the integral really is used — `intervalIntegral.integral_add`
carries integrability side conditions that `integral_const_mul` does not — so
both `(L-u)(Rᵢ(u)-1)` are required to be interval-integrable on `[0, L]`. The
hypotheses are USED, not decorative: unlike `V_scaled` this proof does not go
through without them. Whether some other proof could drop them is NOT settled
here, and no counterexample is formalised in this file; the honest statement is
just that additivity is the step and additivity needs integrability. -/
theorem V_mixture (R₁ R₂ : ℝ → ℝ) (c L : ℝ)
    (h₁ : IntervalIntegrable (fun u => (L - u) * (R₁ u - 1)) MeasureTheory.volume 0 L)
    (h₂ : IntervalIntegrable (fun u => (L - u) * (R₂ u - 1)) MeasureTheory.volume 0 L) :
    V (fun u => c * R₁ u + (1 - c) * R₂ u) L = c * V R₁ L + (1 - c) * V R₂ L := by
  have h : ∀ u : ℝ, (L - u) * ((c * R₁ u + (1 - c) * R₂ u) - 1)
      = c * ((L - u) * (R₁ u - 1)) + (1 - c) * ((L - u) * (R₂ u - 1)) := fun u => by ring
  simp only [V, h]
  rw [intervalIntegral.integral_add (h₁.const_mul c) (h₂.const_mul (1 - c)),
      intervalIntegral.integral_const_mul, intervalIntegral.integral_const_mul]
  ring

/-- **The two routes agree — a control, not a new fact.** `R_c = 1 - c·K` is the
same deformation as the mixture `c·(1 - K) + (1 - c)·1` of the kernel process
with Poisson. Reaching `V_scaled`'s conclusion through `V_mixture` (which needs
integrability) and through `V_scaled` (which does not) must give the same
number. If these ever disagreed, one of the two proofs would be wrong. -/
theorem V_scaled_via_mixture (K : ℝ → ℝ) (c L : ℝ)
    (h₁ : IntervalIntegrable (fun u => (L - u) * ((1 - K u) - 1)) MeasureTheory.volume 0 L) :
    V (fun u => 1 - c * K u) L = (1 - c) * L + c * V (fun u => 1 - K u) L := by
  have hpoi : IntervalIntegrable (fun u => (L - u) * ((1 : ℝ) - 1))
      MeasureTheory.volume 0 L := by
    simp only [sub_self, mul_zero]
    exact continuous_const.intervalIntegrable 0 L
  have hR : (fun u => 1 - c * K u) = fun u => c * ((1 : ℝ) - K u) + (1 - c) * 1 := by
    funext u; ring
  rw [hR, V_mixture (fun u => 1 - K u) (fun _ => 1) c L h₁ hpoi, V_poisson]
  ring

/-! ### Positive control: an explicitly integrated witness

`V_poisson` alone is a weak control — its integrand is identically zero, so it
would still hold if `intervalIntegral` returned the junk value everywhere and
`V_scaled` were off by any factor. The block below closes that hole with a
kernel whose integral is actually computed: `K ≡ 1`, giving
`∫₀^L (L-u) du = L²/2` and hence `V(1 - cK, L) = L - cL²`, a NONZERO,
`c`-dependent witness.

`V_kOne` reaches that closed form by integrating directly. `V_scaled_witness`
reaches the same closed form by routing through `V_scaled`. The two agree, and
they would not agree if the factored-out coefficient in `V_scaled` were wrong:
a spurious factor `α` on the `c` term turns the second route into
`L - αcL²`, and `ring` fails. That is the detuning null for this lemma.
-/

/-- The constant unit kernel, used only as an integration witness. -/
def kOne : ℝ → ℝ := fun _ => 1

/-- `∫₀^L (L - u) du = L²/2`. -/
theorem integral_triangle (L : ℝ) : (∫ x in (0:ℝ)..L, (L - x)) = L ^ 2 / 2 := by
  have hid : IntervalIntegrable (fun x : ℝ => x) MeasureTheory.volume 0 L :=
    (continuous_id : Continuous fun x : ℝ => x).intervalIntegrable 0 L
  rw [intervalIntegral.integral_sub intervalIntegrable_const hid, integral_id]
  simp
  ring

/-- **Direct route.** The integral is really performed; the answer is nonzero and
depends on `c`. -/
theorem V_kOne (c L : ℝ) : V (fun u => 1 - c * kOne u) L = L - c * L ^ 2 := by
  have h : ∀ u : ℝ, (L - u) * ((1 - c * kOne u) - 1) = (-c) * (L - u) := by
    intro u; simp [kOne]; ring
  simp only [V, h, intervalIntegral.integral_const_mul, integral_triangle]
  ring

theorem V_kOne_base (L : ℝ) : V (fun u => 1 - kOne u) L = L - L ^ 2 := by
  have h : (fun u => 1 - kOne u) = (fun u => 1 - (1 : ℝ) * kOne u) := by funext u; ring
  rw [h, V_kOne]; ring

/-- **Second route, through `V_scaled`.** Same statement as `V_kOne`, proved by
factoring the coefficient out instead of by integrating. Agreement is the test:
any wrong coefficient in `V_scaled` breaks the closing `ring`. -/
theorem V_scaled_witness (c L : ℝ) : V (fun u => 1 - c * kOne u) L = L - c * L ^ 2 := by
  rw [V_scaled, V_kOne_base]; ring

/-- The witness is not the junk value: at `L = 2` the deformed variance is `-2`
while the Poisson baseline is `+2`. A functional that silently returned `0` for
every integral could not produce these two different numbers. -/
theorem V_witness_nonzero : V (fun u => 1 - kOne u) 2 = -2 ∧ V (fun _ => 1) 2 = 2 := by
  refine ⟨?_, V_poisson 2⟩
  rw [V_kOne_base]; norm_num

/-- **Detuning null, machine-checked.** Perturb the coefficient that `V_scaled`
factors out — replace `c` by `a·c` for any `a ≠ 1` — and the identity becomes
FALSE, refuted at `c = 1, L = 2` by the witness kernel. So `V_scaled` is not a
statement that would hold for any coefficient: the `c` it factors out is pinned
by the integral, and a `±10%` misfactoring is detected. -/
theorem V_scaled_detuning_null (a : ℝ) (ha : a ≠ 1) :
    ¬ (∀ c L : ℝ, V (fun u => 1 - c * kOne u) L
        = (1 - c) * L + a * c * V (fun u => 1 - kOne u) L) := by
  intro h
  have h1 := h 1 2
  rw [V_kOne, V_kOne_base] at h1
  norm_num at h1
  exact ha h1

/-! ### Making the weight load-bearing (Track B, 2026-08-13)

`V_scaled` is BLIND to the functional it is named after, and this block says so
with a theorem instead of in prose. Adversarial verification on 2026-08-13 showed
that `V_scaled`'s literal proof script also compiles against

    Vbad R L = L + 7 * ∫₀^L (L - u)(R u - 1) du,

so scalar homogeneity of an interval integral pins neither the factor `2` nor the
weight `(L - u)`. `Vaff_scaled` below turns that observation into a theorem: the
same `c`-linearity holds for EVERY member of the two-parameter family

    Vaff a b R L = L + ∫₀^L (a(L - u) + b)(R u - 1) du,

which contains `V = Vaff 2 0`, the counter-model `Vbad = Vaff 7 0`, and the
weightless variant `Vaff 0 2` that drops `(L - u)` altogether. A property shared
by an entire two-parameter family cannot identify a point in it.

What DOES identify the point is an explicitly computed value. `Vaff_kOne` and
`Vaff_kId` give the family's closed form on two witness kernels, and
`Vaff_pins_weight` / `Vaff_pins_weight_two_kernels` show that two such values
force `a = 2 ∧ b = 0` — the counter-model is then formally excluded
(`Vbad_ne_V`, `Vbad_cannot_agree`, `Vbad_fails_V_kOne`) rather than excluded in
a comment.

One witness is not enough and that is also a theorem, not a caveat:
`weightless_agrees_at_one_point` exhibits `Vaff 0 2` agreeing with `V` on the
constant kernel at `L = 2` and separating only on the second kernel.
-/

/-- The near-miss family: weight `a·(L - u) + b` in place of `2·(L - u)`. -/
noncomputable def Vaff (a b : ℝ) (R : ℝ → ℝ) (L : ℝ) : ℝ :=
  L + ∫ u in (0:ℝ)..L, (a * (L - u) + b) * (R u - 1)

/-- The family contains the real functional, at `a = 2, b = 0`. -/
theorem Vaff_two_zero (R : ℝ → ℝ) (L : ℝ) : Vaff 2 0 R L = V R L := by
  have h : ∀ u : ℝ, (2 * (L - u) + 0) * (R u - 1) = 2 * ((L - u) * (R u - 1)) :=
    fun u => by ring
  simp only [Vaff, V, h, intervalIntegral.integral_const_mul]

/-- **The verifier's counter-model, written down.** `2` replaced by `7`. -/
noncomputable def Vbad (R : ℝ → ℝ) (L : ℝ) : ℝ :=
  L + 7 * ∫ u in (0:ℝ)..L, (L - u) * (R u - 1)

theorem Vbad_eq_Vaff (R : ℝ → ℝ) (L : ℝ) : Vbad R L = Vaff 7 0 R L := by
  have h : ∀ u : ℝ, (7 * (L - u) + 0) * (R u - 1) = 7 * ((L - u) * (R u - 1)) :=
    fun u => by ring
  simp only [Vaff, Vbad, h, intervalIntegral.integral_const_mul]

/-- **The defect of `V_scaled`, as a theorem.** The `c`-linearity holds for every
`a` and `b`, so it carries no information whatever about the weight. This is
`V_scaled`'s own proof with `2·(L-u)` replaced by an arbitrary affine weight, and
it goes through unchanged — which is exactly the complaint. -/
theorem Vaff_scaled (a b c : ℝ) (K : ℝ → ℝ) (L : ℝ) :
    Vaff a b (fun u => 1 - c * K u) L
      = (1 - c) * L + c * Vaff a b (fun u => 1 - K u) L := by
  have h : ∀ u : ℝ, (a * (L - u) + b) * ((1 - c * K u) - 1)
      = c * ((a * (L - u) + b) * ((1 - K u) - 1)) := fun u => by ring
  simp only [Vaff, h, intervalIntegral.integral_const_mul]
  ring

/-- `∫₀^L (a(L - u) + b) du = a L²/2 + b L`. -/
theorem integral_affine_weight (a b L : ℝ) :
    (∫ u in (0:ℝ)..L, (a * (L - u) + b)) = a * L ^ 2 / 2 + b * L := by
  have hc : IntervalIntegrable (fun u : ℝ => a * (L - u)) MeasureTheory.volume 0 L :=
    (continuous_const.mul (continuous_const.sub continuous_id)).intervalIntegrable 0 L
  rw [intervalIntegral.integral_add hc intervalIntegrable_const,
      intervalIntegral.integral_const_mul, integral_triangle,
      intervalIntegral.integral_const]
  simp
  ring

/-- **First closed form, for the whole family.** On the constant kernel the
integral is elementary and the answer depends on both parameters:
`Vaff a b (1 - c·kOne) L = L - c(a L²/2 + b L)`. At `a = 2, b = 0` this is
`V_kOne`'s `L - c L²`; at `a = 7, b = 0` it is `L - 7cL²/2`. -/
theorem Vaff_kOne (a b c L : ℝ) :
    Vaff a b (fun u => 1 - c * kOne u) L = L - c * (a * L ^ 2 / 2 + b * L) := by
  have h : ∀ u : ℝ, (a * (L - u) + b) * ((1 - c * kOne u) - 1)
      = (-c) * (a * (L - u) + b) := by intro u; simp [kOne]; ring
  simp only [Vaff, h, intervalIntegral.integral_const_mul, integral_affine_weight]
  ring

theorem Vaff_kOne_base (a b L : ℝ) :
    Vaff a b (fun u => 1 - kOne u) L = L - (a * L ^ 2 / 2 + b * L) := by
  have h : (fun u => 1 - kOne u) = (fun u => 1 - (1:ℝ) * kOne u) := by funext u; ring
  rw [h, Vaff_kOne]; ring

/-- **Discrimination lemma, one kernel.** Two explicitly computed values on the
constant kernel — at `L = 1` and `L = 2` — already force `a = 2` and `b = 0`.
The two lengths are needed: a single value leaves the line `a/2 + b = 1`. -/
theorem Vaff_pins_weight {a b : ℝ}
    (h1 : Vaff a b (fun u => 1 - kOne u) 1 = V (fun u => 1 - kOne u) 1)
    (h2 : Vaff a b (fun u => 1 - kOne u) 2 = V (fun u => 1 - kOne u) 2) :
    a = 2 ∧ b = 0 := by
  rw [Vaff_kOne_base, V_kOne_base] at h1 h2
  norm_num at h1 h2
  constructor <;> linarith

/-- **The counter-model is separated by a computed number.** On the unit kernel at
`L = 2`, `V = -2` and `Vbad = -12`. -/
theorem Vbad_ne_V : Vbad (fun u => 1 - kOne u) 2 ≠ V (fun u => 1 - kOne u) 2 := by
  rw [Vbad_eq_Vaff, Vaff_kOne_base, V_kOne_base]; norm_num

/-- **No near miss reproduces the closed form.** Any `a ≠ 2` or `b ≠ 0` fails
`V_kOne`, so `L - c L²` is a property of the functional and not of the family. -/
theorem Vaff_fails_V_kOne {a b : ℝ} (h : a ≠ 2 ∨ b ≠ 0) :
    ¬ (∀ c L : ℝ, Vaff a b (fun u => 1 - c * kOne u) L = L - c * L ^ 2) := by
  intro hall
  have h1 := hall 1 1
  have h2 := hall 1 2
  rw [Vaff_kOne] at h1 h2
  norm_num at h1 h2
  rcases h with ha | hb
  · exact ha (by linarith)
  · exact hb (by linarith)

/-- The instance for the functional the verifier actually built. -/
theorem Vbad_fails_V_kOne :
    ¬ (∀ c L : ℝ, Vbad (fun u => 1 - c * kOne u) L = L - c * L ^ 2) := by
  intro h
  refine Vaff_fails_V_kOne (a := 7) (b := 0) (Or.inl (by norm_num)) (fun c L => ?_)
  rw [← Vbad_eq_Vaff]; exact h c L

/-- **The guard fires on the counter-model.** `Vbad = Vaff 7 0` cannot even
satisfy the hypotheses of the discrimination lemma: were it to agree with `V` at
the two witnesses, `Vaff_pins_weight` would return `7 = 2`. The counter-model is
now formally excluded, not excluded in prose. -/
theorem Vbad_cannot_agree :
    ¬ (Vaff 7 0 (fun u => 1 - kOne u) 1 = V (fun u => 1 - kOne u) 1
       ∧ Vaff 7 0 (fun u => 1 - kOne u) 2 = V (fun u => 1 - kOne u) 2) := by
  rintro ⟨h1, h2⟩
  have h := (Vaff_pins_weight h1 h2).1
  norm_num at h

/-- Second witness kernel, `K u = u`. Its integrand is genuinely quadratic, so it
weights the window differently from `kOne` and gives an INDEPENDENT equation. -/
def kId : ℝ → ℝ := fun u => u

/-- `∫₀^L (L - u)·u du = L³/6`. -/
theorem integral_triangle_id (L : ℝ) : (∫ u in (0:ℝ)..L, (L - u) * u) = L ^ 3 / 6 := by
  have h : ∀ u : ℝ, (L - u) * u = L * u - u ^ 2 := fun u => by ring
  have hlu : IntervalIntegrable (fun u : ℝ => L * u) MeasureTheory.volume 0 L :=
    (continuous_const.mul continuous_id).intervalIntegrable 0 L
  have hsq : IntervalIntegrable (fun u : ℝ => u ^ 2) MeasureTheory.volume 0 L :=
    (continuous_pow 2).intervalIntegrable 0 L
  simp only [h]
  rw [intervalIntegral.integral_sub hlu hsq, intervalIntegral.integral_const_mul,
      integral_id, integral_pow]
  norm_num
  ring

/-- `∫₀^L (a(L - u) + b)·u du = a L³/6 + b L²/2`. -/
theorem integral_affine_weight_id (a b L : ℝ) :
    (∫ u in (0:ℝ)..L, (a * (L - u) + b) * u) = a * L ^ 3 / 6 + b * L ^ 2 / 2 := by
  have h : ∀ u : ℝ, (a * (L - u) + b) * u = a * ((L - u) * u) + b * u := fun u => by ring
  have h1 : IntervalIntegrable (fun u : ℝ => a * ((L - u) * u)) MeasureTheory.volume 0 L :=
    (continuous_const.mul ((continuous_const.sub continuous_id).mul
      continuous_id)).intervalIntegrable 0 L
  have h2 : IntervalIntegrable (fun u : ℝ => b * u) MeasureTheory.volume 0 L :=
    (continuous_const.mul continuous_id).intervalIntegrable 0 L
  simp only [h]
  rw [intervalIntegral.integral_add h1 h2, intervalIntegral.integral_const_mul,
      intervalIntegral.integral_const_mul, integral_triangle_id, integral_id]
  norm_num
  ring

/-- **Second closed form for `V` itself**, on the identity kernel:
`V(1 - c·kId, L) = L - c L³/3`. Independent of `V_kOne`: it is cubic in `L`,
so the two together over-determine the weight. -/
theorem V_kId (c L : ℝ) : V (fun u => 1 - c * kId u) L = L - c * L ^ 3 / 3 := by
  have h : ∀ u : ℝ, (L - u) * ((1 - c * kId u) - 1) = (-c) * ((L - u) * u) := by
    intro u; simp [kId]; ring
  simp only [V, h, intervalIntegral.integral_const_mul, integral_triangle_id]
  ring

theorem V_kId_base (L : ℝ) : V (fun u => 1 - kId u) L = L - L ^ 3 / 3 := by
  have h : (fun u => 1 - kId u) = (fun u => 1 - (1:ℝ) * kId u) := by funext u; ring
  rw [h, V_kId]; ring

/-- The family's second closed form. -/
theorem Vaff_kId (a b c L : ℝ) :
    Vaff a b (fun u => 1 - c * kId u) L = L - c * (a * L ^ 3 / 6 + b * L ^ 2 / 2) := by
  have h : ∀ u : ℝ, (a * (L - u) + b) * ((1 - c * kId u) - 1)
      = (-c) * ((a * (L - u) + b) * u) := by intro u; simp [kId]; ring
  simp only [Vaff, h, intervalIntegral.integral_const_mul, integral_affine_weight_id]
  ring

theorem Vaff_kId_base (a b L : ℝ) :
    Vaff a b (fun u => 1 - kId u) L = L - (a * L ^ 3 / 6 + b * L ^ 2 / 2) := by
  have h : (fun u => 1 - kId u) = (fun u => 1 - (1:ℝ) * kId u) := by funext u; ring
  rw [h, Vaff_kId]; ring

/-- **Discrimination lemma, two kernels at one length.** Agreement with `V` on
`kOne` and on `kId`, both at `L = 2`, forces `a = 2 ∧ b = 0`. This is the
stronger form: the two equations come from two different SHAPES rather than from
two window lengths of the same shape. -/
theorem Vaff_pins_weight_two_kernels {a b : ℝ}
    (h1 : Vaff a b (fun u => 1 - kOne u) 2 = V (fun u => 1 - kOne u) 2)
    (h2 : Vaff a b (fun u => 1 - kId u) 2 = V (fun u => 1 - kId u) 2) :
    a = 2 ∧ b = 0 := by
  rw [Vaff_kOne_base, V_kOne_base] at h1
  rw [Vaff_kId_base, V_kId_base] at h2
  norm_num at h1 h2
  constructor <;> linarith

/-- **One agreeing number is not evidence.** The weightless variant `Vaff 0 2`,
which deletes `(L - u)` entirely, agrees with `V` on the constant kernel at
`L = 2` — both give `-2` — and is separated only by the second kernel, where it
gives `-2` against `V`'s `-2/3`. Stated as a theorem because the same trap is
what let the constant-7 model survive: a functional that matches at one point has
not been tested. -/
theorem weightless_agrees_at_one_point :
    Vaff 0 2 (fun u => 1 - kOne u) 2 = V (fun u => 1 - kOne u) 2
      ∧ Vaff 0 2 (fun u => 1 - kId u) 2 ≠ V (fun u => 1 - kId u) 2 := by
  constructor
  · rw [Vaff_kOne_base, V_kOne_base]; norm_num
  · rw [Vaff_kId_base, V_kId_base]; norm_num

/-- An empty window has no variance. Holds for every `R`, junk value or not. -/
theorem V_zero (R : ℝ → ℝ) : V R 0 = 0 := by simp [V]

/-! ### Number variance -/

/-- Number variance implied by `R(u) = 1 - c * K(u)`, given `G`, the variance
the same kernel produces at `c = 1`.

This is now a DERIVED shorthand, not an assumption: `V_eq_sigmaC` proves that
`V (1 - c·K) L = sigmaC c L (V (1 - K) L)` for every kernel `K`, straight from
`intervalIntegral`. Everything stated in terms of `sigmaC` below therefore
transfers to the integral functional; `rphi_excluded_by_measurement_V` carries
the end-to-end conclusion across. -/
def sigmaC (c L G : ℝ) : ℝ := (1 - c) * L + c * G

/-- **`sigmaC` is what `V` computes.** The bridge from the integral functional to
the algebraic shorthand the rest of the file uses. -/
theorem V_eq_sigmaC (c : ℝ) (K : ℝ → ℝ) (L : ℝ) :
    V (fun u => 1 - c * K u) L = sigmaC c L (V (fun u => 1 - K u) L) := by
  rw [V_scaled]; unfold sigmaC; ring

/-- At `c = 1` the extensive term vanishes. This IS the sum rule, seen at
finite `L` instead of at zero wavenumber. -/
theorem sigmaC_one (L G : ℝ) : sigmaC 1 L G = G := by unfold sigmaC; ring

theorem sigmaC_sub_base (c L G : ℝ) : sigmaC c L G - G = (1 - c) * (L - G) := by
  unfold sigmaC; ring

/-- **The mixture realises the same variance.** `R_c` is realised by a
non-ergodic coin flip: the determinantal process with probability `c`, a
Poisson process with probability `1 - c`. Both have unit intensity, so the
mean is common and `Var = E[Var] + Var[E]` loses its second term; the variance
is then the plain convex combination. That it coincides with `sigmaC` is the
statement that a pair correlation does not determine a point process — two
different objects, one `Σ²`. -/
theorem mixture_eq_sigmaC (c L G : ℝ) : c * G + (1 - c) * L = sigmaC c L G := by
  unfold sigmaC; ring

/-- Linearity of `ρ₂` in the law, which is what makes the mixture work. -/
theorem mixture_pairCorr (c k : ℝ) : c * (1 - k) + (1 - c) * 1 = 1 - c * k := by
  ring

/-- **Extensivity.** For `c < 1` the variance passes any bound: the deformation
leaves a fixed fraction `1 - c` of Poisson fluctuation that never decays. -/
theorem sigmaC_exceeds {c : ℝ} (hc : c < 1) (G B : ℝ) (hG : 0 ≤ c * G) :
    ∃ L₀ : ℝ, ∀ L ≥ L₀, B ≤ sigmaC c L G := by
  refine ⟨B / (1 - c), fun L hL => ?_⟩
  have h1 : (0 : ℝ) < 1 - c := by linarith
  have h2 : B ≤ (1 - c) * L := by
    rw [ge_iff_le, div_le_iff₀ h1] at hL; linarith
  unfold sigmaC; linarith

/-- **Rigidity forces `c = 1`.** The contrapositive of extensivity, and the
form the analytic input actually arrives in: Fujii 1975 gives `Σ² = O(log L)`,
hence `o(L)`, hence bounded on any bounded set of ratios. Fujii is a
HYPOTHESIS here, never an assertion. -/
theorem bounded_variance_forces_one {c G B : ℝ} (hc : c ≤ 1) (hG : 0 ≤ c * G)
    (hB : ∀ L : ℝ, 0 ≤ L → sigmaC c L G ≤ B) : c = 1 := by
  by_contra hne
  have hlt : c < 1 := lt_of_le_of_ne hc hne
  obtain ⟨L₀, hL₀⟩ := sigmaC_exceeds hlt G (B + 1) hG
  have hmax : (0 : ℝ) ≤ max L₀ 0 := le_max_right _ _
  have h1 := hL₀ (max L₀ 0) (le_max_left _ _)
  have h2 := hB (max L₀ 0) hmax
  linarith

/-! ### Turning a measurement into a bound -/

/-- **The measurement interface.** A single observed variance at a single
window length bounds the coefficient from below. With the 2026-08-13 reading
`Σ²(500) = 0.371 ± 0.060 ≤ 1/2` on the corpus's 100,000 Odlyzko zeros, this
gives `c ≥ 999/1000`.

`0 ≤ c` and `0 ≤ G` are not decoration: without them a large negative `c * G`
could absorb the extensive term, and the bound would be false. -/
theorem measured_bound_forces_c {c G L B : ℝ} (hc : 0 ≤ c) (hG : 0 ≤ G)
    (hL : 0 < L) (h : sigmaC c L G ≤ B) : 1 - B / L ≤ c := by
  unfold sigmaC at h
  have hcG : 0 ≤ c * G := mul_nonneg hc hG
  have h1 : (1 - c) * L ≤ B := by linarith
  have h2 : 1 - c ≤ B / L := by rw [le_div_iff₀ hL]; linarith
  linarith

/-- The concrete instance: `Σ²(500) ≤ 1/2` forces `c ≥ 999/1000`. -/
theorem measured_2026_08_13 {c G : ℝ} (hc : 0 ≤ c) (hG : 0 ≤ G)
    (h : sigmaC c 500 G ≤ 1 / 2) : (999 : ℝ) / 1000 ≤ c := by
  have := measured_bound_forces_c hc hG (by norm_num : (0:ℝ) < 500) h
  norm_num at this ⊢
  linarith

/-! ### The φ instance -/

/-- `φ⁻³ ≠ 1`. Everything above is shape- and constant-free; this is the only
line in the file where the golden ratio appears at all, and it appears only to
be substituted in. -/
theorem phi_inv_cube_ne_one : (1 : ℝ) / phi ^ 3 ≠ 1 := by
  have h3 : phi ^ 3 = 2 * phi + 1 := phi_cubed
  have hp : phi > 1 := by
    have h5 : Real.sqrt 5 > 2 := by
      nlinarith [sqrt5_sq, Real.sqrt_nonneg 5]
    rw [phi]; linarith
  have hne : phi ^ 3 ≠ 1 := by rw [h3]; linarith
  have hpos : phi ^ 3 > 0 := by rw [h3]; linarith
  intro h
  rw [div_eq_one_iff_eq (ne_of_gt hpos)] at h
  exact hne h.symm

/-- `R_φ` fails the sum rule. -/
theorem rphi_fails_sumRule : S0 (1 / phi ^ 3) 1 ≠ 0 := by
  intro h
  exact phi_inv_cube_ne_one (sumRule_forces_one h)

/-- **The deficiency factor is the proposal's own constant.** `S(0) = 1 - φ⁻³`,
and the factor by which the coefficient misses is `1 / φ⁻³ = φ³`. -/
theorem deficiency_factor : (1 : ℝ) / (1 / phi ^ 3) = phi ^ 3 := by
  have hpos : phi ^ 3 > 0 := by rw [phi_cubed]; nlinarith [phi_pos]
  field_simp

/-- Three closed forms for `S(0)` agree: `1 - φ⁻³ = 3 - √5 = 2φ⁻²`. -/
theorem S0_phi_closed_forms :
    S0 (1 / phi ^ 3) 1 = 3 - Real.sqrt 5 ∧ S0 (1 / phi ^ 3) 1 = 2 / phi ^ 2 := by
  have h5 : Real.sqrt 5 ^ 2 = 5 := sqrt5_sq
  have hp : phi > 1 := by
    have : Real.sqrt 5 > 2 := by nlinarith [Real.sqrt_nonneg 5]
    rw [phi]; linarith
  have hpos : phi ^ 3 > 0 := by positivity
  have h2 : phi ^ 2 > 0 := by positivity
  constructor
  · unfold S0
    rw [phi]
    field_simp
    nlinarith [h5, Real.sqrt_nonneg 5]
  · unfold S0
    rw [phi]
    field_simp
    nlinarith [h5, Real.sqrt_nonneg 5]

/-- **The measured exclusion, end to end.** `φ⁻³` is below the floor that the
2026-08-13 number-variance reading imposes, so no `Σ²(500) ≤ 1/2` observation
is compatible with `R_φ`. -/
theorem rphi_excluded_by_measurement {G : ℝ} (hG : 0 ≤ G) :
    ¬ sigmaC (1 / phi ^ 3) 500 G ≤ 1 / 2 := by
  intro h
  have hp : phi > 1 := by
    have : Real.sqrt 5 > 2 := by nlinarith [sqrt5_sq, Real.sqrt_nonneg 5]
    rw [phi]; linarith
  have hpos : (0 : ℝ) < phi ^ 3 := by positivity
  have hc : 0 ≤ 1 / phi ^ 3 := le_of_lt (by positivity)
  have hlow := measured_2026_08_13 hc hG h
  -- φ³ = 2φ + 1 > 3, so φ⁻³ < 1/3 < 999/1000.
  have h3 : phi ^ 3 = 2 * phi + 1 := phi_cubed
  have hgt : phi ^ 3 > 3 := by rw [h3]; linarith
  have : (1 : ℝ) / phi ^ 3 < 1 / 3 := by
    apply div_lt_div_of_pos_left (by norm_num) (by norm_num) hgt
  linarith

/-- **The exclusion, stated on the integral functional.** Identical in content to
`rphi_excluded_by_measurement`, but with `sigmaC` replaced by `V`, so the
conclusion is now attached to `L + 2∫₀^L (L-u)(R(u)-1) du` itself rather than to
an algebraic shorthand. `K` is arbitrary: no property of the kernel is used, which
is the shape-independence of the sum rule seen at finite `L`. -/
theorem rphi_excluded_by_measurement_V (K : ℝ → ℝ)
    (hG : 0 ≤ V (fun u => 1 - K u) 500) :
    ¬ V (fun u => 1 - (1 / phi ^ 3) * K u) 500 ≤ 1 / 2 := by
  rw [V_eq_sigmaC]
  exact rphi_excluded_by_measurement hG

/-! ### Track B closed: uniqueness against a GENERAL weight `w : ℝ → ℝ → ℝ`

WHAT WAS WRONG BEFORE, IN ONE SENTENCE. `Vaff_pins_weight` and
`Vaff_pins_weight_two_kernels` pin `a = 2 ∧ b = 0` only INSIDE the affine family
`a(L-u) + b`; they say nothing about a weight that is not affine in `u`. The
verifier's counter-model

    w_q(L, u) = 2(L - u) + u² - 2u + 2/3

satisfies the hypotheses of BOTH of those lemmas and is nevertheless a different
functional. That is not asserted here as a caveat — it is `Vq_passes_all_three_witnesses`
below, machine-checked, together with `Vq_separated_at_three` which exhibits the
disagreement (`-8` against `-6`).

WHAT IS PROVED NOW. Two uniqueness theorems against an arbitrary
`w : ℝ → ℝ → ℝ`, differing in which class of pair correlations `R` they quantify
over and in how strong a conclusion they reach.

  A. `Vw_pins_weight_ae` — quantifies over `R = 1 - 1_s` for EVERY measurable
     `s ⊆ ℝ`, at ONE fixed window length `L ≥ 0`; concludes `w L u = 2(L - u)`
     for ALMOST EVERY `u ∈ (0, L]`. Almost-everywhere is the strongest possible
     conclusion from integral data alone, since altering `w L ·` on a null set
     changes no value of the functional.

  B. `Vw_pins_weight_continuous` — quantifies over the ONE-PARAMETER family of
     step kernels `R = 1 - 1_{(0,t]}`, `0 ≤ t ≤ L`, at one fixed `L`, and adds
     the hypothesis that `w L ·` is continuous; concludes `w L t = 2(L - t)` for
     EVERY `t` in the OPEN interval `(0, L)`. Pointwise, not almost-everywhere,
     at the price of continuity and of a countably-generated test family.

WHY EACH TEST CLASS SUFFICES, STATED PLAINLY.

  A's class is the indicator class. Agreement of the two functionals on
  `R = 1 - 1_s` says exactly `∫_{(0,L] ∩ s} w L = ∫_{(0,L] ∩ s} 2(L - ·)` for
  every measurable `s`. That is verbatim the hypothesis of mathlib's
  `MeasureTheory.Integrable.ae_eq_of_forall_setIntegral_eq`, which is the
  statement that the indicator class separates `L¹`. No smaller class of that
  shape would do: a class closed under nothing at all cannot separate.

  B's class is the step class `{1_{(0,t]}}`, indexed by a single real `t`.
  Agreement gives `∫₀^t (w L - 2(L - ·)) = 0` for every `t ∈ [0, L]`; the
  fundamental theorem of calculus then differentiates in `t`. Continuity of
  `w L ·` is what licenses the differentiation and is USED — without it the
  conclusion has to retreat to A's almost-everywhere form.

THE ENDPOINTS ARE NOT PINNED AND CANNOT BE. B concludes on the open interval
`(0, L)`, A on `(0, L]` up to a null set. Changing a weight at the two endpoints
changes no integral, so no integral-agreement hypothesis can ever reach them.
This is a property of the problem, not a gap in the proof.

WHAT IS STILL NOT ESTABLISHED. Neither theorem quantifies over a class of
CONTINUOUS kernels. Polynomials or continuous bump kernels also separate, by
Weierstrass, but that route is not formalised here and nothing below should be
read as covering it. `shift_poly_excluded` handles POLYNOMIAL PERTURBATIONS OF
THE WEIGHT, which is a different statement from a polynomial test class.
-/

/-- **General-weight variance functional.** `w L u` is an arbitrary weight
depending on both the window length and the position inside it:

    Vw w R L = L + ∫₀^L w(L, u) (R u - 1) du.

`V = Vw (fun L u => 2(L - u))` (`Vw_canonical`) and `Vaff a b = Vw (fun L u =>
a(L-u) + b)` (`Vaff_eq_Vw`), so every functional discussed in this file is a
point of this space. -/
noncomputable def Vw (w : ℝ → ℝ → ℝ) (R : ℝ → ℝ) (L : ℝ) : ℝ :=
  L + ∫ u in (0:ℝ)..L, w L u * (R u - 1)

open MeasureTheory

/-- The real functional is the point `w(L, u) = 2(L - u)` of the general space. -/
theorem Vw_canonical (R : ℝ → ℝ) (L : ℝ) : Vw (fun L u => 2 * (L - u)) R L = V R L := by
  have h : ∀ u : ℝ, 2 * (L - u) * (R u - 1) = 2 * ((L - u) * (R u - 1)) := fun u => by ring
  simp only [Vw, V, h, intervalIntegral.integral_const_mul]

/-- The affine near-miss family sits inside the general space, so the old
two-parameter discrimination lemmas are special cases of what follows. -/
theorem Vaff_eq_Vw (a b : ℝ) (R : ℝ → ℝ) (L : ℝ) :
    Vaff a b R L = Vw (fun L u => a * (L - u) + b) R L := rfl

/-- The indicator kernel of a set. `R = 1 - kInd s` is a legitimate pair
correlation shape — bounded and measurable — and is the test object of
`Vw_pins_weight_ae`. -/
noncomputable def kInd (s : Set ℝ) : ℝ → ℝ := Set.indicator s (fun _ => (1:ℝ))

/-- Evaluating the general functional on an indicator kernel turns it into a set
integral of the weight. This is the whole bridge between "two functionals agree"
and "two `L¹` functions have equal set integrals". -/
theorem Vw_indicator {w : ℝ → ℝ → ℝ} {L : ℝ} (hL : 0 ≤ L) {s : Set ℝ} (hs : MeasurableSet s) :
    Vw w (fun u => 1 - kInd s u) L = L - ∫ u in Set.Ioc 0 L ∩ s, w L u := by
  have hfun : (fun u => w L u * ((1 - kInd s u) - 1))
      = fun u => -(Set.indicator s (fun u => w L u) u) := by
    funext u
    by_cases hu : u ∈ s <;> simp [kInd, hu]
  rw [Vw, hfun, intervalIntegral.integral_of_le hL, MeasureTheory.integral_neg,
      MeasureTheory.setIntegral_indicator hs]
  ring

/-- The same, for the step kernel `1_{(0,t]}` with `0 ≤ t ≤ L`: the functional
reads off the weight's antiderivative at `t`. -/
theorem Vw_step {w : ℝ → ℝ → ℝ} {L t : ℝ} (hL : 0 ≤ L) (ht0 : 0 ≤ t) (htL : t ≤ L) :
    Vw w (fun u => 1 - kInd (Set.Ioc 0 t) u) L = L - ∫ u in (0:ℝ)..t, w L u := by
  rw [Vw_indicator hL measurableSet_Ioc,
      Set.inter_eq_right.2 (Set.Ioc_subset_Ioc_right htL),
      intervalIntegral.integral_of_le ht0]

/-- **UNIQUENESS OF THE WEIGHT, ALMOST EVERYWHERE (Track B headline).**

QUANTIFIER STRUCTURE, IN PLAIN WORDS. Fix any window length `L ≥ 0` and any
weight `w : ℝ → ℝ → ℝ` whose slice `w L ·` is integrable on `(0, L]`. IF for
EVERY measurable set `s` the general functional `Vw w` and the real functional
`V` return the same number on the pair correlation `R = 1 - 1_s` at that one
length `L`, THEN `w L u = 2(L - u)` for almost every `u ∈ (0, L]`.

The `∀ s` is inside; the `L` is fixed and arbitrary; `w` is a completely general
function of two variables, NOT of `u` alone and NOT affine. That is precisely
what the demoted claim lacked.

`Vw_pins_weight_ae_all_lengths` re-exports this with the length quantified.

WHY THE INTEGRABILITY HYPOTHESIS IS THERE AND NOT DECORATIVE: it is the
hypothesis of mathlib's separation lemma, and the proof does not run without it.
No claim is made here that it is necessary.

WHY THE CONCLUSION IS ALMOST-EVERYWHERE AND NOT POINTWISE: two weights differing
on a Lebesgue-null set give the same value to every integral, so they are
indistinguishable by ANY hypothesis of this form. A pointwise conclusion is
available only by adding regularity — see `Vw_pins_weight_continuous`. -/
theorem Vw_pins_weight_ae {w : ℝ → ℝ → ℝ} {L : ℝ} (hL : 0 ≤ L)
    (hw : IntegrableOn (fun u => w L u) (Set.Ioc 0 L))
    (h : ∀ s : Set ℝ, MeasurableSet s →
      Vw w (fun u => 1 - kInd s u) L = V (fun u => 1 - kInd s u) L) :
    (fun u => w L u) =ᵐ[volume.restrict (Set.Ioc 0 L)] (fun u => 2 * (L - u)) := by
  have hcont : Continuous (fun u : ℝ => 2 * (L - u)) :=
    continuous_const.mul (continuous_const.sub continuous_id)
  have hw2 : IntegrableOn (fun u : ℝ => 2 * (L - u)) (Set.Ioc 0 L) :=
    (intervalIntegrable_iff_integrableOn_Ioc_of_le hL).1 (hcont.intervalIntegrable 0 L)
  refine MeasureTheory.Integrable.ae_eq_of_forall_setIntegral_eq _ _ hw hw2 ?_
  intro s hs _
  have hkey := h s hs
  rw [Vw_indicator hL hs, ← Vw_canonical, Vw_indicator hL hs] at hkey
  have heq : (∫ u in Set.Ioc 0 L ∩ s, w L u) = ∫ u in Set.Ioc 0 L ∩ s, 2 * (L - u) := by
    linarith
  rw [Measure.restrict_restrict hs, Set.inter_comm s (Set.Ioc 0 L)]
  exact heq

/-- **Non-vacuity control for both uniqueness theorems.** A hypothesis that no
weight could satisfy would make the theorems above true and worthless, so the
canonical weight is exhibited passing both tests — every indicator kernel and
every step kernel, at every length. Together with `Vq_weight_fails_step_kernels`
this brackets the hypothesis: something meets it and something does not. -/
theorem Vw_hypothesis_satisfiable (L : ℝ) :
    (∀ s : Set ℝ, MeasurableSet s →
        Vw (fun L u => 2 * (L - u)) (fun u => 1 - kInd s u) L
          = V (fun u => 1 - kInd s u) L)
    ∧ (∀ t : ℝ, 0 ≤ t → t ≤ L →
        Vw (fun L u => 2 * (L - u)) (fun u => 1 - kInd (Set.Ioc 0 t) u) L
          = V (fun u => 1 - kInd (Set.Ioc 0 t) u) L) :=
  ⟨fun _ _ => Vw_canonical _ L, fun _ _ _ => Vw_canonical _ L⟩

/-- The same statement with the window length quantified: IF the two functionals
agree on every indicator kernel at every length, THEN for every `L ≥ 0` the slice
`w L ·` equals `2(L - ·)` almost everywhere on `(0, L]`. -/
theorem Vw_pins_weight_ae_all_lengths {w : ℝ → ℝ → ℝ}
    (hw : ∀ L : ℝ, 0 ≤ L → IntegrableOn (fun u => w L u) (Set.Ioc 0 L))
    (h : ∀ (s : Set ℝ), MeasurableSet s → ∀ L : ℝ,
      Vw w (fun u => 1 - kInd s u) L = V (fun u => 1 - kInd s u) L) :
    ∀ L : ℝ, 0 ≤ L →
      (fun u => w L u) =ᵐ[volume.restrict (Set.Ioc 0 L)] (fun u => 2 * (L - u)) :=
  fun L hL => Vw_pins_weight_ae hL (hw L hL) (fun s hs => h s hs L)

/-- **UNIQUENESS OF THE WEIGHT, POINTWISE, FROM A ONE-PARAMETER TEST FAMILY.**

QUANTIFIER STRUCTURE, IN PLAIN WORDS. Fix any window length `L` and any general
weight `w : ℝ → ℝ → ℝ` whose slice `w L ·` is continuous. IF for EVERY `t` with
`0 ≤ t ≤ L` the two functionals agree on the step kernel `R = 1 - 1_{(0,t]}`,
THEN for EVERY `t` strictly between `0` and `L`, `w L t = 2(L - t)` exactly.

This is direction 2 of the task made precise: uniqueness WITHIN the separating
family `{1_{(0,t]} : 0 ≤ t ≤ L}`, which is a single real parameter rather than
all measurable sets. It buys a pointwise conclusion with continuity, by the
fundamental theorem of calculus.

The conclusion stops at the open interval on purpose: the endpoints `0` and `L`
are two points, they carry no measure, and no hypothesis about integrals can
constrain the weight there. -/
theorem Vw_pins_weight_continuous {w : ℝ → ℝ → ℝ} {L : ℝ} (hL : 0 ≤ L)
    (hw : Continuous (fun u => w L u))
    (h : ∀ t : ℝ, 0 ≤ t → t ≤ L →
      Vw w (fun u => 1 - kInd (Set.Ioc 0 t) u) L = V (fun u => 1 - kInd (Set.Ioc 0 t) u) L)
    {t : ℝ} (ht0 : 0 < t) (htL : t < L) : w L t = 2 * (L - t) := by
  have hlin : Continuous (fun u : ℝ => 2 * (L - u)) :=
    continuous_const.mul (continuous_const.sub continuous_id)
  set g : ℝ → ℝ := fun u => w L u - 2 * (L - u) with hg
  have hgc : Continuous g := hw.sub hlin
  have hF : ∀ x : ℝ, 0 ≤ x → x ≤ L → (∫ u in (0:ℝ)..x, g u) = 0 := by
    intro x hx0 hxL
    have hk := h x hx0 hxL
    rw [Vw_indicator hL measurableSet_Ioc, ← Vw_canonical,
        Vw_indicator hL measurableSet_Ioc] at hk
    have hinter : Set.Ioc (0:ℝ) L ∩ Set.Ioc (0:ℝ) x = Set.Ioc (0:ℝ) x :=
      Set.inter_eq_right.2 (Set.Ioc_subset_Ioc_right hxL)
    rw [hinter] at hk
    have h1 : (∫ u in Set.Ioc (0:ℝ) x, w L u) = ∫ u in Set.Ioc (0:ℝ) x, 2 * (L - u) := by
      linarith
    have hi1 : IntegrableOn (fun u => w L u) (Set.Ioc 0 x) :=
      (intervalIntegrable_iff_integrableOn_Ioc_of_le hx0).1 (hw.intervalIntegrable 0 x)
    have hi2 : IntegrableOn (fun u : ℝ => 2 * (L - u)) (Set.Ioc 0 x) :=
      (intervalIntegrable_iff_integrableOn_Ioc_of_le hx0).1 (hlin.intervalIntegrable 0 x)
    rw [hg, intervalIntegral.integral_of_le hx0, MeasureTheory.integral_sub hi1 hi2, h1]
    ring
  have hd : HasDerivAt (fun x => ∫ u in (0:ℝ)..x, g u) (g t) t :=
    (hgc.integral_hasStrictDerivAt 0 t).hasDerivAt
  have hev : (fun x => ∫ u in (0:ℝ)..x, g u) =ᶠ[nhds t] (fun _ => (0:ℝ)) := by
    filter_upwards [Ioo_mem_nhds ht0 htL] with x hx
    exact hF x (le_of_lt hx.1) (le_of_lt hx.2)
  have hd0 : HasDerivAt (fun x => ∫ u in (0:ℝ)..x, g u) 0 t :=
    (hasDerivAt_const t (0:ℝ)).congr_of_eventuallyEq hev
  have hgt : g t = 0 := hd.unique hd0
  have hzero : w L t - 2 * (L - t) = 0 := hgt
  linarith

/-- **The old affine result, recovered from the general theorem.** Agreement of
`Vaff a b` with `V` on the step kernels at the single length `L = 2` already
forces `a = 2 ∧ b = 0`. Stated to check that the general theorem specialises to
what the family lemmas said, rather than replacing them with something weaker. -/
theorem Vaff_pins_weight_step_kernels {a b : ℝ}
    (h : ∀ t : ℝ, 0 ≤ t → t ≤ 2 →
      Vaff a b (fun u => 1 - kInd (Set.Ioc 0 t) u) 2
        = V (fun u => 1 - kInd (Set.Ioc 0 t) u) 2) :
    a = 2 ∧ b = 0 := by
  have hcont : Continuous (fun u : ℝ => a * ((2:ℝ) - u) + b) :=
    (continuous_const.mul (continuous_const.sub continuous_id)).add continuous_const
  have h' : ∀ t : ℝ, 0 ≤ t → t ≤ 2 →
      Vw (fun L u => a * (L - u) + b) (fun u => 1 - kInd (Set.Ioc 0 t) u) 2
        = V (fun u => 1 - kInd (Set.Ioc 0 t) u) 2 := by
    intro t ht0 ht2; rw [← Vaff_eq_Vw]; exact h t ht0 ht2
  have e1 := Vw_pins_weight_continuous (w := fun L u => a * (L - u) + b) (L := 2)
    (by norm_num) hcont h' (t := 1) (by norm_num) (by norm_num)
  have e2 := Vw_pins_weight_continuous (w := fun L u => a * (L - u) + b) (L := 2)
    (by norm_num) hcont h' (t := 1/2) (by norm_num) (by norm_num)
  norm_num at e1 e2
  constructor <;> linarith

/-! #### Direction 3: weights of the form `2(L - u) + q(u)`, and the R that separates them

The counter-model is of this shape. The separating pair correlation is named
explicitly: it is `R = 1 - kOne`, the constant kernel, and the separating datum
is `∫₀^L q`. For a nonzero continuous `q` that integral is nonzero at some `L`,
because otherwise the fundamental theorem of calculus would differentiate it back
to `q ≡ 0`. So the ONE kernel `kOne`, tested at ALL lengths, already kills every
nonzero continuous perturbation of the weight that depends on `u` alone —
including every nonzero polynomial one.
-/

/-- The general functional on the constant kernel, for a `u`-only shifted weight:
the whole effect of the shift is the number `∫₀^L q`. -/
theorem Vw_shift_kOne {q : ℝ → ℝ} (hq : Continuous q) (L : ℝ) :
    Vw (fun L u => 2 * (L - u) + q u) (fun u => 1 - kOne u) L
      = V (fun u => 1 - kOne u) L - ∫ u in (0:ℝ)..L, q u := by
  have hfun : (fun u => (2 * (L - u) + q u) * ((1 - kOne u) - 1))
      = fun u => -(2 * (L - u)) - q u := by
    funext u; simp [kOne]; ring
  have ha : IntervalIntegrable (fun u : ℝ => -(2 * (L - u))) volume 0 L :=
    ((continuous_const.mul (continuous_const.sub continuous_id)).neg).intervalIntegrable 0 L
  have hb : IntervalIntegrable q volume 0 L := hq.intervalIntegrable 0 L
  have htri : (∫ u in (0:ℝ)..L, -(2 * (L - u))) = -L ^ 2 := by
    have h2 : ∀ u : ℝ, -(2 * (L - u)) = (-2) * (L - u) := fun u => by ring
    simp only [h2, intervalIntegral.integral_const_mul, integral_triangle]; ring
  rw [Vw, hfun, intervalIntegral.integral_sub ha hb, htri, V_kOne_base]
  ring

/-- **One kernel, all lengths, forces the shift to vanish identically.**

QUANTIFIER STRUCTURE. IF `q` is continuous and the shifted-weight functional
agrees with `V` on the single pair correlation `R = 1 - kOne` at EVERY window
length `L`, THEN `q` is the zero function — everywhere, not almost everywhere,
because continuity plus the fundamental theorem of calculus gives the pointwise
value back. -/
theorem shift_zero_of_agree_kOne {q : ℝ → ℝ} (hq : Continuous q)
    (h : ∀ L : ℝ, Vw (fun L u => 2 * (L - u) + q u) (fun u => 1 - kOne u) L
          = V (fun u => 1 - kOne u) L) : q = 0 := by
  have hF : (fun x => ∫ u in (0:ℝ)..x, q u) = fun _ => (0:ℝ) := by
    funext L
    have hL := h L
    rw [Vw_shift_kOne hq L] at hL
    linarith
  funext t
  have hd : HasDerivAt (fun x => ∫ u in (0:ℝ)..x, q u) (q t) t :=
    (hq.integral_hasStrictDerivAt 0 t).hasDerivAt
  have hd0 : HasDerivAt (fun x => ∫ u in (0:ℝ)..x, q u) 0 t := by
    rw [hF]; exact hasDerivAt_const t 0
  simpa using hd.unique hd0

/-- The contrapositive: EVERY nonzero continuous shift of the weight is refuted
by the constant kernel at some length. -/
theorem shift_excluded_of_ne_zero {q : ℝ → ℝ} (hq : Continuous q) (hne : q ≠ 0) :
    ¬ (∀ L : ℝ, Vw (fun L u => 2 * (L - u) + q u) (fun u => 1 - kOne u) L
        = V (fun u => 1 - kOne u) L) :=
  fun h => hne (shift_zero_of_agree_kOne hq h)

/-- **Direction 3, polynomial case.** No weight `2(L - u) + p(u)` with `p` a
nonzero polynomial can reproduce `V` on the constant kernel at all lengths. -/
theorem shift_poly_excluded {p : Polynomial ℝ} (hp : p ≠ 0) :
    ¬ (∀ L : ℝ, Vw (fun L u => 2 * (L - u) + p.eval u) (fun u => 1 - kOne u) L
        = V (fun u => 1 - kOne u) L) := by
  refine shift_excluded_of_ne_zero p.continuous ?_
  intro hzero
  refine hp (Polynomial.funext (fun r => ?_))
  have hr : p.eval r = (0 : ℝ → ℝ) r := congrFun hzero r
  simpa using hr

/-! #### The verifier's quadratic counter-model, written down and excluded -/

/-- Cubic antiderivative, used only to evaluate the counter-model's integrals. -/
theorem integral_cubic (a b c d L : ℝ) :
    (∫ u in (0:ℝ)..L, (a * u ^ 3 + b * u ^ 2 + c * u + d))
      = a * L ^ 4 / 4 + b * L ^ 3 / 3 + c * L ^ 2 / 2 + d * L := by
  have i3 : IntervalIntegrable (fun u : ℝ => a * u ^ 3) volume 0 L :=
    (continuous_const.mul (continuous_pow 3)).intervalIntegrable 0 L
  have i2 : IntervalIntegrable (fun u : ℝ => b * u ^ 2) volume 0 L :=
    (continuous_const.mul (continuous_pow 2)).intervalIntegrable 0 L
  have i1 : IntervalIntegrable (fun u : ℝ => c * u) volume 0 L :=
    (continuous_const.mul continuous_id).intervalIntegrable 0 L
  rw [intervalIntegral.integral_add ((i3.add i2).add i1) intervalIntegrable_const,
      intervalIntegral.integral_add (i3.add i2) i1,
      intervalIntegral.integral_add i3 i2,
      intervalIntegral.integral_const_mul, intervalIntegral.integral_const_mul,
      intervalIntegral.integral_const_mul, integral_pow, integral_pow, integral_id,
      intervalIntegral.integral_const]
  norm_num
  ring

/-- The general functional on the identity kernel, for a `u`-only shifted weight. -/
theorem Vw_shift_kId {q : ℝ → ℝ} (hq : Continuous q) (L : ℝ) :
    Vw (fun L u => 2 * (L - u) + q u) (fun u => 1 - kId u) L
      = V (fun u => 1 - kId u) L - ∫ u in (0:ℝ)..L, q u * u := by
  have hfun : (fun u => (2 * (L - u) + q u) * ((1 - kId u) - 1))
      = fun u => -(2 * ((L - u) * u)) - q u * u := by
    funext u; simp [kId]; ring
  have ha : IntervalIntegrable (fun u : ℝ => -(2 * ((L - u) * u))) volume 0 L :=
    ((continuous_const.mul ((continuous_const.sub continuous_id).mul
      continuous_id)).neg).intervalIntegrable 0 L
  have hb : IntervalIntegrable (fun u => q u * u) volume 0 L :=
    (hq.mul continuous_id).intervalIntegrable 0 L
  have htri : (∫ u in (0:ℝ)..L, -(2 * ((L - u) * u))) = -(L ^ 3 / 3) := by
    have h2 : ∀ u : ℝ, -(2 * ((L - u) * u)) = (-2) * ((L - u) * u) := fun u => by ring
    simp only [h2, intervalIntegral.integral_const_mul, integral_triangle_id]; ring
  rw [Vw, hfun, intervalIntegral.integral_sub ha hb, htri, V_kId_base]
  ring

/-- The counter-model's shift, `q(u) = u² - 2u + 2/3`. Its two defining
properties are `∫₀^1 q = 0` and `∫₀^2 q = 0` — which is exactly why it survives
both concrete-`L` lemmas — while `∫₀^3 q = 2 ≠ 0`. -/
noncomputable def qQuad : ℝ → ℝ := fun u => u ^ 2 - 2 * u + 2 / 3

theorem qQuad_continuous : Continuous qQuad := by
  unfold qQuad; continuity

/-- **The counter-model functional**, weight `2(L - u) + u² - 2u + 2/3`. -/
noncomputable def Vq (R : ℝ → ℝ) (L : ℝ) : ℝ := Vw (fun L u => 2 * (L - u) + qQuad u) R L

theorem integral_qQuad (L : ℝ) :
    (∫ u in (0:ℝ)..L, qQuad u) = L ^ 3 / 3 - L ^ 2 + 2 * L / 3 := by
  have h : ∀ u : ℝ, qQuad u = 0 * u ^ 3 + 1 * u ^ 2 + (-2) * u + 2 / 3 := by
    intro u; simp [qQuad]; ring
  simp only [h, integral_cubic]
  ring

theorem integral_qQuad_id (L : ℝ) :
    (∫ u in (0:ℝ)..L, qQuad u * u) = L ^ 4 / 4 - 2 * L ^ 3 / 3 + L ^ 2 / 3 := by
  have h : ∀ u : ℝ, qQuad u * u = 1 * u ^ 3 + (-2) * u ^ 2 + (2 / 3) * u + 0 := by
    intro u; simp [qQuad]; ring
  simp only [h, integral_cubic]
  ring

/-- Closed form of the counter-model on the constant kernel: `L/3 - L³/3`,
against `V`'s `L - L²`. The two curves cross at `L = 1` and `L = 2`. -/
theorem Vq_kOne (L : ℝ) : Vq (fun u => 1 - kOne u) L = L / 3 - L ^ 3 / 3 := by
  rw [Vq, Vw_shift_kOne qQuad_continuous, integral_qQuad, V_kOne_base]
  ring

/-- Closed form of the counter-model on the identity kernel. -/
theorem Vq_kId (L : ℝ) :
    Vq (fun u => 1 - kId u) L = L - L ^ 3 / 3 - (L ^ 4 / 4 - 2 * L ^ 3 / 3 + L ^ 2 / 3) := by
  rw [Vq, Vw_shift_kId qQuad_continuous, integral_qQuad_id, V_kId_base]

/-- **The demotion, as a theorem.** `Vq` satisfies the hypothesis of
`Vaff_pins_weight` (constant kernel at `L = 1` and `L = 2`) AND the hypothesis of
`Vaff_pins_weight_two_kernels` (constant and identity kernels at `L = 2`), on all
three witnesses at once — and is still a different functional. This is why the
earlier headline was demoted, and it is now checked rather than asserted. -/
theorem Vq_passes_all_three_witnesses :
    Vq (fun u => 1 - kOne u) 1 = V (fun u => 1 - kOne u) 1
    ∧ Vq (fun u => 1 - kOne u) 2 = V (fun u => 1 - kOne u) 2
    ∧ Vq (fun u => 1 - kId u) 2 = V (fun u => 1 - kId u) 2 := by
  refine ⟨?_, ?_, ?_⟩
  · rw [Vq_kOne, V_kOne_base]; norm_num
  · rw [Vq_kOne, V_kOne_base]; norm_num
  · rw [Vq_kId, V_kId_base]; norm_num

/-- The separating number: at `L = 3` on the constant kernel the counter-model
gives `-8` where `V` gives `-6`. -/
theorem Vq_separated_at_three :
    Vq (fun u => 1 - kOne u) 3 = -8 ∧ V (fun u => 1 - kOne u) 3 = -6 := by
  constructor
  · rw [Vq_kOne]; norm_num
  · rw [V_kOne_base]; norm_num

theorem Vq_ne_V : Vq (fun u => 1 - kOne u) 3 ≠ V (fun u => 1 - kOne u) 3 := by
  rw [Vq_kOne, V_kOne_base]; norm_num

/-- **The counter-model is excluded** — by exhibiting the length at which the
constant kernel separates it. -/
theorem Vq_excluded :
    ¬ (∀ L : ℝ, Vq (fun u => 1 - kOne u) L = V (fun u => 1 - kOne u) L) :=
  fun h => Vq_ne_V (h 3)

/-- The same exclusion obtained from the general direction-3 theorem instead of
from the computed number, so the two routes agree. -/
theorem Vq_excluded_via_uniqueness :
    ¬ (∀ L : ℝ, Vw (fun L u => 2 * (L - u) + qQuad u) (fun u => 1 - kOne u) L
        = V (fun u => 1 - kOne u) L) := by
  refine shift_excluded_of_ne_zero qQuad_continuous ?_
  intro hzero
  have h0 : qQuad 0 = (0 : ℝ → ℝ) 0 := congrFun hzero 0
  simp [qQuad] at h0

/-- **The guard fires on the counter-model.** `Vq`'s weight cannot satisfy the
hypothesis of `Vw_pins_weight_continuous` at `L = 3`: if it did, the theorem
would return `w(3, 1) = 4`, whereas that weight is `4 - 1/3`. So the counter-model
is now formally shut out of the new uniqueness theorem, not merely absent from it. -/
theorem Vq_weight_fails_step_kernels :
    ¬ (∀ t : ℝ, 0 ≤ t → t ≤ 3 →
        Vw (fun L u => 2 * (L - u) + qQuad u) (fun u => 1 - kInd (Set.Ioc 0 t) u) 3
          = V (fun u => 1 - kInd (Set.Ioc 0 t) u) 3) := by
  intro h
  have hcont : Continuous (fun u : ℝ => 2 * ((3:ℝ) - u) + qQuad u) :=
    (continuous_const.mul (continuous_const.sub continuous_id)).add qQuad_continuous
  have hpin := Vw_pins_weight_continuous (w := fun L u => 2 * (L - u) + qQuad u) (L := 3)
    (by norm_num) hcont h (t := 1) (by norm_num) (by norm_num)
  norm_num [qQuad] at hpin

/-- **The test family has to be the whole family — one member is not enough.**
The counter-model's weight AGREES with `2(L - u)` on the step kernel `1_{(0,1]}`
at `L = 2`, because `∫₀^1 q = 0`. So `Vw_pins_weight_continuous` genuinely needs
its `∀ t`, and a single step kernel would repeat exactly the mistake that
`Vq_passes_all_three_witnesses` records. Compare
`weightless_agrees_at_one_point`, the same trap one level down. -/
theorem qQuad_agrees_on_one_step_kernel :
    Vw (fun L u => 2 * (L - u) + qQuad u) (fun u => 1 - kInd (Set.Ioc 0 1) u) 2
      = V (fun u => 1 - kInd (Set.Ioc 0 1) u) 2 := by
  rw [Vw_step (L := 2) (t := 1) (by norm_num) (by norm_num) (by norm_num), ← Vw_canonical,
      Vw_step (L := 2) (t := 1) (by norm_num) (by norm_num) (by norm_num)]
  have ha : IntervalIntegrable (fun u : ℝ => 2 * ((2:ℝ) - u)) volume 0 1 :=
    (continuous_const.mul (continuous_const.sub continuous_id)).intervalIntegrable 0 1
  have hb : IntervalIntegrable qQuad volume 0 1 := qQuad_continuous.intervalIntegrable 0 1
  rw [intervalIntegral.integral_add ha hb, integral_qQuad]
  norm_num

end Trinity.ZetaSumRule
