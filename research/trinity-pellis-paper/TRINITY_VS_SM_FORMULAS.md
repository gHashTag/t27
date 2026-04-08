# Trinity / Pellis vs Standard Model — formula map

Side-by-side reference: **what lives in this repo’s Trinity–Pell layer** vs **the usual SM definition**. Not a claim of equivalence unless the Notes column says so. Canonical IDs: `[FORMULA_TABLE.md](FORMULA_TABLE.md)`.

---

## 1. Algebraic core (exact in ℝ)


| Topic                | Trinity / t27 (symbolic)     | Standard Model (usual form)                                                    | Notes                                                                            |
| -------------------- | ---------------------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| **L5 “Trinity sum”** | \varphi^2 + \varphi^{-2} = 3 | *(no single SM identity)* — gauge group is e.g. SU(3)\times SU(2)\times U(1)_Y | SM does not postulate \varphi; L5 is **repo K2 / numeric anchor** (L5 IDENTITY). |
| **Golden equation**  | \varphi^2 = \varphi + 1      | —                                                                              | Defines \varphi = (1+\sqrt{5})/2; orthogonal to SM Lagrangian structure.         |


---

## 2. Integer / recurrence structures


| Topic                | Trinity / Pellis                                                       | Standard Model                                      | Notes                                                                             |
| -------------------- | ---------------------------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------------------------------------- |
| **Pell ladder**      | P_0=0, P_1=1, P_n = 2P_{n-1}+P_{n-2}; block **P_1..P_5 = 1,2,5,12,29** | No canonical “Pell” sector                          | Used in hybrid maps and Pellis spec (`pellis-formulas.t27`).                      |
| **Structural scale** | \varphi^5 \approx 11.09 (diagnostic)                                   | \alpha^{-1}(m_e) \approx 137 from QED + measurement | CLI compares **difference**; equality is **false** by design (tests enforce gap). |


---

## 3. Fine-structure constant \alpha


| Topic                                  | Trinity / Pellis                                                                                                                             | Standard Model                                                     | Notes                                                                              |
| -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| **Inverse \alpha^{-1} (reference)**    | CODATA **2022-class** central in spec / CLI: **137.035999166** (aligned with **Rev. Mod. Phys.**-style **(15)** tail in PDG/CODATA extracts) | \alpha = g^2/(4\pi) in QED; **measured** / **running** \alpha(\mu) | Row 4 — **reference only**.                                                        |
| **Pellis closed form (phenomenology)** | \alpha^{-1} \stackrel{?}{\sim} \dfrac{360}{\varphi^2} - \dfrac{2}{\varphi^3} + \dfrac{1}{(3\varphi)^5} → **137.035999164766…**               | Same **\alpha^{-1}** from CODATA + QED link                        | Row 31; **not** SM-derived; see § Pre-registered checkpoint in `FORMULA_TABLE.md`. |


**CODATA 2022 — 166 vs 177 (same adjustment, two listings):** The adjustment publishes **α** and **α⁻¹** separately. [NIST \alpha](https://physics.nist.gov/cgi-bin/cuu/Value?alph) implies **1/α ≈ 137.035999177(21)**; the **direct** **α⁻¹** row is **137.035999166(15)** in [Rev. Mod. Phys. **97**, 025002 (2025)](https://journals.aps.org/rmp/abstract/10.1103/RevModPhys.97.025002) / [J. Phys. Chem. Ref. Data **54**, 033105](https://pubs.aip.org/aip/jpr/article/54/3/033105/3363695/CODATA-recommended-values-of-the-fundamental). **Inversion does not preserve** the symmetric uncertainty; this repo uses the **direct α⁻¹** (**166**) for specs and CLI. Reviewer-ready sentence: see `**FORMULA_TABLE.md`** § Pre-registered checkpoint (subsection *reviewer FAQ*). [Wikipedia](https://en.wikipedia.org/wiki/Fine-structure_constant) often cites both. Passive benchmark: [CODATA 2026](https://codata.org/initiatives/data-science-and-stewardship/fundamental-physical-constants/) (results **~early 2027**).

---

## 4. Electroweak (Weinberg angle)


| Topic              | Trinity / Pellis (conjectural rows)                                      | Standard Model                                                                                                         | Notes                                                                                                                                                                               |
| ------------------ | ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Weinberg angle** | **Ansatz:** \sin^2\theta_W \stackrel{?}{\sim} \varphi^{-3} \approx 0.236 | \sin^2\theta_W = g'^2/(g^2+g'^2); or tree \cos\theta_W = m_W/m_Z; **\overline{\mathrm{MS}}** value **scale-dependent** | Row 22; **~2.1%** vs PDG-like \sin^2\theta_W \approx 0.23122 at M_Z; falsifiability: P2@MESA / DUNE — `FORMULA_TABLE.md`. [Wikipedia](https://en.wikipedia.org/wiki/Weinberg_angle) |


---

## 5. Quark mixing (CKM)


| Trinity / Pellis (ansatz)                   | Standard Model                                                               | Notes                                                                                              |
| ------------------------------------------- | ---------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| |V_{us}| \stackrel{?}{\sim} \varphi^{-3}    | |V_{us}| from **unitary CKM** + PDG (Wolfenstein \lambda \approx 0.225 etc.) | Row 23; **~4.9%** vs central — Cabibbo–Weinberg proximity is SM **numerology**, not derived in SM. |
| |V_{cb}| \stackrel{?}{\sim} \varphi^{-6.5}  | PDG modulus (~**0.041** class; cite extract)                                   | Row 24; **\varphi^{-6.5} \approx 0.0438**, **δ ~ 6.3%** vs ~0.0412. (*\varphi^{-7} \approx 0.0344* is worse — **~16%** vs central.)          |
| |V_{ub}| \stackrel{?}{\sim} \varphi^{-11.5} | PDG modulus (~**0.0038** class)                                                | Row 25; **\varphi^{-11.5} \approx 0.00395**, **δ ~ 3.4%** vs ~0.00382. **LHCb / Belle II** refine \|V_{ub}\|, \|V_{cb}\| over time.          |
| **Full CKM**                                | V_{\rm CKM} unitary, **V V^\dagger = I**                                     | Rows 11–13, 16–19 — **PDG references** in CLI, not φ closures.                                     |


**Literature link (φ across sectors):** Rodejohann & Datta discuss golden-ratio–flavored connections between **Cabibbo** and **neutrino** angles ([PRD **76**, 117301 (2007)](https://journals.aps.org/prd/abstract/10.1103/PhysRevD.76.117301)) — **not** proof of rows 22–25; context only.

---

## 6. Boson masses (reference level)


| Observable    | Trinity / repo                         | Standard Model                                                                           | Notes                                    |
| ------------- | -------------------------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------- |
| m_W, m_Z, m_H | **PDG numbers** in `--pellis-extended` | EWSB: m_W = \tfrac{1}{2} g v, m_Z = \tfrac{1}{2}\sqrt{g^2+g'^2}v, m_H = \sqrt{2\lambda}v | Rows 7–9 — **reference**, not φ closure. |


---

## 7. Lepton masses


| Topic                    | Trinity / Pellis                                                                                                                   | Standard Model                                                                       | Notes                                                                                                                                                                                                                        |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Koide Q**              | Q = \dfrac{m_e+m_\mu+m_\tau}{(\sqrt{m_e}+\sqrt{m_\mu}+\sqrt{m_\tau})^2} \approx \dfrac{2}{3} (**~3.3 ppm** vs 2/3 with PDG masses) | Yukawa couplings y_\ell with m_\ell = y_\ell v/\sqrt{2} — **no** Koide theorem in SM | Row 26; **empirical**; not Trinity-derived.                                                                                                                                                                                  |
| **Mass ratios (ansatz)** | m_\tau/m_e \stackrel{?}{\sim} \varphi^{17}, m_\mu/m_e \stackrel{?}{\sim} \varphi^{11}                                              | Ratios from **Yukawas** (free parameters)                                            | Rows 28–29; **CONJECTURAL**. **Epistemic caveat:** choosing **integer** exponents n in \varphi^n after seeing the data gives **freedom**; **~3–5%** agreement over a handful of trials is **not** strong evidence by itself. |


---

## 8. Quark mass ratio (ansatz)


| Trinity / Pellis                       | Standard Model                                           | Notes                                                                                     |
| -------------------------------------- | -------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| m_b/m_s \stackrel{?}{\sim} \varphi^{8} | Ratio from **running masses** / scheme — not fixed by SM | Row 30; **lattice QCD** and scheme choices move m_s,m_b; same **integer-n** caveat as §7. |


---

## 9. Neutrinos


| Topic                                   | Trinity / Pellis                                                                          | Standard Model                                              | Notes                                                                                                                                                                                                                                       |
| --------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Golden ratio A (GRa1), row 27**       | \cot\theta_{12}=\varphi \Leftrightarrow \theta_{12}=\arctan(1/\varphi)\approx 31.72^\circ | **PMNS**; \theta_{12} from global fits (**NuFIT 6.0** etc.) | **Disfavored** as **parameter-free LO** sum rule in recent global analyses (e.g. **~3σ** class constraints on GR mixing templates — cite the fit paper you use).                                                                            |
| **Golden ratio B (GRB)**                | \cos\theta_{12}=\varphi/2 \Rightarrow \theta_{12}\approx 36.0^\circ                       | same                                                        | Different φ construction; also **poor fit** to data in recent studies ([e.g. arXiv:2510.06526](https://arxiv.org/abs/2510.06526) — verify claims in primary text).                                                                          |
| **Viable A₅-flavor direction**          | LO φ relations + **NLO** / charged-lepton / seesaw **corrections**                        | SM + effective neutrino mass operators                      | Phenomenology survives only with **subleading** structure, not bare LO golden-ratio equalities. Review context: [Nucl. Phys. B **994** (2023)](https://linkinghub.elsevier.com/retrieve/pii/S0550321325000100) (retrieve URL may redirect). |
| **\Delta m^2_{21}, \Delta m^2_{3\ell}** | Table / PDG placeholders                                                                  | Oscillation Hamiltonian, **\Delta m^2** splittings          | Rows 14–15 — reference, not φ.                                                                                                                                                                                                              |
| **m_{\nu_1}/m_{\nu_2} placeholder**     | CLI placeholder                                                                           | Seesaw / unknown absolute masses                            | Row 10 — **not** PDG.                                                                                                                                                                                                                       |


Row **27** in `FORMULA_TABLE.md` remains **honest LO benchmark**; compare to **NuFIT 6.0** central and errors ([arXiv:2410.05380](https://arxiv.org/abs/2410.05380)).

---

## 10. Hybrid diagnostics (φ–Pell geometry only)

These have **no standard “SM formula twin”**; they are **constructed observables** for Trinity–Pell comparison in `tri math compare`.


| Map                      | Trinity / Pellis definition                                                    | SM analogue | Notes                                                   |
| ------------------------ | ------------------------------------------------------------------------------ | ----------- | ------------------------------------------------------- |
| **Hybrid v1** H_5^{(v1)} | u_k=\varphi^k, v_k=P_{k+1}; a_k=u_k/\sum u, b_k=v_k/\max v; **H=\sum a_k b_k** | —           | Row 6; **L1 + max-Pell**; not \cos of a physical angle. |
| **Hybrid v2** H_N^{(v2)} | L2-unit \hat u,\hat v from \varphi^i, P_{i+1}; **H=\hat u\cdot\hat v**         | —           | Row 20; `--hybrid-v2 --n N`.                            |
| **\theta_N**             | \theta_N = \arccos(\mathrm{clip}(H_N^{(v2)})) in degrees                       | —           | Row 21; diagnostic angle in **construction space**.     |


---

## 11. Discrete flavor templates (neutrino sector — not SSOT)

Illustrative **symmetry benchmarks** vs **global fits** (NuFIT-class). Your row **27** (GRa1) sits in the **disfavored LO** bucket; **TM1 + CSD(3)**-type schemes are often **closer** to best-fit \theta_{12} today — **DUNE / JUNO** will sharpen this by **~2030**.


| Template            | Group / idea  | \theta_{12} prediction (typical) | Fit status (literature snapshot)                                                                                        |
| ------------------- | ------------- | -------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Tri-bimaximal (TBM) | A_4, S_4 etc. | 35.26^\circ                      | **Excluded** (needs \theta_{13}\neq 0 etc.) — [e.g. arXiv:1301.1340](https://arxiv.org/abs/1301.1340)                   |
| **Golden ratio A**  | A_5 / φ       | **31.72^\circ**                  | **Disfavored** at LO in global GR-mixing fits                                                                           |
| Golden ratio B      | φ (alternate) | **\sim 36^\circ**                | **Poor fit** — verify in primary sources                                                                                |
| Hexagonal           | S_3 etc.      | 30^\circ                         | **Disfavored** — [example NP B context](https://linkinghub.elsevier.com/retrieve/pii/S055032131500108X)                 |
| TM1 + CSD(3)        | S_4 chain     | \sim 34.3^\circ                  | Often **surviving** in comparative fits — see e.g. [Universe **9** (2023) 472](https://www.mdpi.com/2218-1997/9/11/472) |


---

## 12. What tests are trustworthy? (honest tier table)


| Tier                               | What is checked                                                         | How                                                                                                                                                                                               | Reliability                                                                               |
| ---------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| **Math**                           | \varphi identities, Pell block, Pellis arithmetic **137.035999164766…** | `t27` / `tri`, high-precision replay                                                                                                                                                              | **Theorem-level** in ℝ; **f64** is tol-bounded (see Flocq / `PhiFloat.v`).                |
| **Pellis vs CODATA**               | Sub-ppb alignment vs **2022** recommended \alpha^{-1}                   | **Passive:** unchanged formula vs **CODATA 2026+** ([CODATA](https://codata.org/initiatives/data-science-and-stewardship/fundamental-physical-constants/)); refresh `FORMULA_TABLE.md` checkpoint | **High** for **pre-registration** story; **low** for “proof of nature” without mechanism. |
| **\sin^2\theta_W vs \varphi^{-3}** | **~2.1%** gap                                                           | **Active:** P2@MESA (**~0.15%** class), DUNE ND (**~2031–2033**); see `FORMULA_TABLE.md`                                                                                                          | **Medium** — real falsifiers; **2%** ansatz may fail sharply.                             |
| **CKM φ ansätze**                  | **~5%** level                                                           | LHCb Run 3, Belle II on |V_{ub}|,|V_{cb}|                                                                                                                                                         | **Low** as discriminators — too coarse unless tightened.                                  |
| **Mass ratios \varphi^n**          | **~3–5%**                                                               | Lattice + scheme for m_s,m_b; **integer n** freedom                                                                                                                                               | **Low** — easy to cherry-pick n.                                                          |


---

## 13. Quick index → `FORMULA_TABLE.md` rows


| Rows       | Content                                |
| ---------- | -------------------------------------- |
| 1–2        | L5 / \varphi algebra                   |
| 3–5        | Pell block, \alpha^{-1} ref, \varphi^5 |
| 6–7, 20–21 | Hybrid v1 / v2                         |
| 7–9, 11–19 | SM references in CLI                   |
| 22–25      | EW + CKM **φ ansätze**                 |
| 26–30      | Koide, \theta_{12}, mass-ratio ansätze |
| 31         | Pellis \alpha^{-1} closed form         |
| 32         | Conjecture **H2:** \sin\theta_{13} = \varphi^{-4} |


---

**Maintenance:** When `FORMULA_TABLE.md` gains new IDs, extend §13. **SSOT** for executable checks remains `**specs/**/*.t27*`* + `tri`, not this Markdown file. `**ALPHA_INV_REFERENCE**` in `pellis-formulas.t27` / CLI tracks **CODATA 2022 central 166** — bump when PDG/CODATA releases the next recommendation and re-seal. For **100+ digit** replay of φ-only rows (incl. Pellis α⁻¹), run `**scripts/verify_precision.py`** (mpmath) or `**scripts/print_pellis_seal_decimal.py**` (stdlib). Zig/GMP plan: `**GMP_MPFR_ROADMAP.md**`.