# Hybrid conjecture — formal sketch (issues #277, #287)

English-first research note. Implementation lives in `specs/physics/pellis-formulas.t27` and `tri math compare`. **Hybrid v2 plan:** GitHub issue [#287](https://github.com/gHashTag/t27/issues/287).

## Hybrid v1 vs v2 — do not mix in outreach or papers

These are **different maps**. Using one number while describing the other is ambiguous.

| Version | Normalization | Pell window | Typical value (IEEE f64) | Theta note |
|--------|----------------|------------|---------------------------|------------|
| **v1** (shipped) | L1 (simplex) on phi^k; Pell side scaled by max = P5 | fixed length 5 (k = 0..4) | **~0.563780474444** | arccos(0.5638) ~ 55.7 deg is **not** a Euclidean angle between L2-unit vectors; treat v1 as a **scalar diagnostic** unless you define an L2 embedding first. |
| **v2** (proposed, #287) | L2 unit vectors on both sides | N grows (e.g. 2 to 152) | **~0.9617** (target once coded) | theta ~ 15.90 deg only after v2 is **frozen** and reproduced in a clean checkout. |

**Outreach rule:** do **not** update the Pellis letter or `FORMULA_TABLE.md` outreach snippet with v2 numbers until `tri math compare --hybrid-v2` (and optional `--theta`) is merged, golden-tested, and CI-green. Until then, cite **v1 ~0.564** only, with normalization spelled out.

**H1 wording rule:** “H1 partially confirmed” is **not** SSOT-honest until v2 reproduces the agreed golden value at stated N from pure `git clone` + `tri` — not before.

## Unlock sequence (aligned with #287)

1. **Formulas:** v1 and v2 are written in the subsections below; keep them in sync with Rust when implemented.
2. Implement `--hybrid-v2` / `--theta` in Rust (`bootstrap/src/math_compare.rs`); no Python on the verification path.
3. Lock golden values for v2 at N in **{5, 10, 15, 20, 50, 152}** (and any other agreed checkpoints) in `.t27` test/invariant and/or Rust tests; refresh seals as required.
4. Extend experience JSONL with `hybrid_v2`, `theta_deg`, `N`, `pellis_spec_seal_hash` (keep `hybrid_inner` as v1 for regression).
5. After CI is green, update outreach and external letters.

## Hybrid v1 (shipped — `tri math compare --hybrid`)

**Fixed dimension:** only **N = 5** (indices k = 0..4). The current CLI does not expose N.

**Pell numbers:** P(0)=0, P(1)=1, P(n)=2 P(n-1)+P(n-2). The code uses **P_1..P_5** = 1, 2, 5, 12, 29 aligned with k.

**Raw vectors (length 5):**

- u_k = phi^k, k in {0,1,2,3,4}
- v_k = P_{k+1} (v_0=P_1, …, v_4=P_5)

**L1 (simplex) normalization on the phi side:**

- a_k = u_k / (sum_{j=0}^4 u_j)

**Max normalization on the Pell side:**

- b_k = v_k / (max_{j=0}^4 v_j) = v_k / P_5

**Hybrid v1 scalar (matches `bootstrap/src/math_compare.rs`):**

- **H_5^(v1) = sum_{k=0}^4 a_k * b_k**

Reference IEEE f64: **~0.563780474444**. Do not treat arccos(H_5^(v1)) as a Euclidean angle between L2-unit vectors; H_5^(v1) is a **named diagnostic** only.

## Hybrid v2 (implemented — issue #287, #339)

**Goal:** cosine similarity between two **L2-normalized** positive vectors in R^N, then optional **theta_N** in degrees.

**Dimension N:** integer **N >= 2**, chosen per run (flag `--n N`). Roadmap checkpoints: **{5, 10, 15, 20, 50, 152}**. N is the **length of both ladders**; tying N=152 to the sacred formula catalog is a **convention** until the catalog feeds the same builder.

**Raw vectors (length N):**

- u_i = phi^i, i = 0..N-1
- v_i = P_{i+1}, i = 0..N-1

**L2 normalization:**

- u_hat = u / ||u||_2, v_hat = v / ||v||_2

**Hybrid v2 scalar:**

- **H_N^(v2) = u_hat . v_hat** (in [0,1] for this construction)

**Angle (flag `--theta`, degrees):**

- **theta_N = arccos(clip(H_N^(v2), -1, 1)) * (180 / pi)**

**Golden values (computed by `tri math compare --hybrid --hybrid-v2 --theta --n <N>`):**

| N | H_N^(v2) | theta (deg) |
|---|-----------|-------------|
| 5 | 0.9649159951 | 15.2219 |
| 10 | 0.9617744938 | 15.8931 |
| 15 | 0.9617437739 | 15.8995 |
| 20 | 0.9617435184 | 15.8995 |
| 50 | 0.9617435163 | 15.8995 |
| 152 | 0.9617435163 | 15.8995 |

**Plateau:** H_N^(v2) converges to ~0.9617435 by N=15, stable to 10+ digits by N=20.

## Conjecture H1 (structural projection, research — not proven in code)

**H1 (projection):** A Trinity monomial of the form


M = 2^{a}3^{b}\varphi^{p}\pi^{m}e^{q}


(with integer exponents (a,b,p,m,q) in a stated finite range) is the **image** of a truncated Pellis-type expansion


\sum_{k=0}^{N} c_k\varphi^{-k}
\quad\text{with}\quad N \leq 3


under a fixed linear or renormalization map T (to be specified: coefficients c_k from Pell data, truncation rule, and normalization).

**Testable corollary (paper-grade, requires map T fixed):** if H1 holds for the chosen T and the constant catalog is extended in a documented way, a **renormalized hybrid statistic** H (derived from the same geometry as today’s CLI inner product, but possibly rescaled) should **converge** to a stable value (one natural target is H \to 1 under a chosen normalization). If, under repeated controlled extensions, H fails to stabilize, that is **falsification of H1** for that T.

**Relation to current CLI:** `tri math compare --hybrid` implements a **concrete diagnostic** inner product (~0.564 on IEEE `f64`); it is **not** yet the general T in H1. Porting H1 into code means defining T, truncation N, and the renormalized H explicitly, then locking them in `.t27` + seals.

## Conjecture (falsifiable, weak form — operational)

If a renormalization-style map links Pell-weighted thin-structure data to an effective \phi-scaling law, then **extensions of the constant set** (neutrino sector, CKM, electroweak masses) should move the hybrid score **predictably** under a *stated* embedding rule — or the conjecture fails for that rule.

**Strong form (not claimed in code):** the hybrid score tends to a fixed value as the formula catalog grows. **Current code does not test convergence**; it only prints one number per run.

## What the CLI falsifies immediately

- **Naive identity \phi^5 = \alpha^{-1}** is false for the CODATA-class reference used in code (|\phi^5 - \alpha^{-1}| \approx 1.26\times 10^2). Honesty is part of the instrument design.

## Sensitivity flag (precision note)

`--sensitivity` reports a **numeric partial derivative of the L5 sum** \mathrm{TRINITY} = \phi^2 + \phi^{-2} with respect to \phi, and (with `--hybrid`) of the hybrid score. This measures **stability of those scalars** under tiny \phi perturbations, not sensitivity of the whole 152-formula catalog unless each formula is wired similarly.

## Experience log (audit trail)

Each `tri math compare` run appends one JSON line to `.trinity/experience/math_compare.jsonl` (gitignored locally). **Today:** flags, v1 scalars, `pellis_spec_seal_hash` when the seal file exists. **Planned (#287):** add `hybrid_v2` (H_N^(v2)), `theta_deg`, `N`, while retaining v1 under e.g. `hybrid_v1` or the existing `hybrid_inner` key for regression.

## Open work (post-merge checklist)


| Item                                                                                 | Status |
| ------------------------------------------------------------------------------------ | ------ |
| Hybrid v2 + goldens + CLI flags                                                      | [#287](https://github.com/gHashTag/t27/issues/287) |
| Replace neutrino ratio placeholder with PDG-consistent documentation                 | Open   |
| Grow `FORMULA_TABLE.md` toward 152 rows with spec links                              | Open   |
| Define a **testable** convergence criterion for hybrid score under catalog extension | Open   |
| Optional: correlate hybrid score with seal / spec version across CI artifacts        | Open   |


## Collaboration anchor

For external reviewers (Pellis / Olsen / others): reproducible entrypoints are the spec file, `tri math compare`, and this note — no Python on the verification path for these commands.