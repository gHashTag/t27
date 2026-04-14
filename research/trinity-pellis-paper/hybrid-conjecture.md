# Hybrid conjecture — formal sketch (issue #277)

English-first research note. Implementation lives in `specs/physics/pellis-formulas.t27` and `tri math compare`.

## Definitions (as implemented today)

- **Trinity monomial ladder (diagnostic):** normalized weights from \(\phi^k\) for \(k = 0..4\).
- **Pell ladder (integer, \(\sqrt{2}\) structure):** \(P_1..P_5 = 1,2,5,12,29\) as explicit constants in the spec; CLI uses the same recurrence for the hybrid map.
- **Hybrid score:** dot product of the two normalized vectors above (dimensionless scalar). **Not** a physical observable until mapped to measured quantities.

## Conjecture H1 (structural projection, research — not proven in code)

**H1 (projection):** A Trinity monomial of the form

\[
M = 2^{a}\,3^{b}\,\varphi^{p}\,\pi^{m}\,e^{q}
\]

(with integer exponents \((a,b,p,m,q)\) in a stated finite range) is the **image** of a truncated Pellis-type expansion

\[
\sum_{k=0}^{N} c_k\,\varphi^{-k}
\quad\text{with}\quad N \leq 3
\]

under a fixed linear or renormalization map \(T\) (to be specified: coefficients \(c_k\) from Pell data, truncation rule, and normalization).

**Testable corollary (paper-grade, requires map \(T\) fixed):** if H1 holds for the chosen \(T\) and the constant catalog is extended in a documented way, a **renormalized hybrid statistic** \(H\) (derived from the same geometry as today’s CLI inner product, but possibly rescaled) should **converge** to a stable value (one natural target is \(H \to 1\) under a chosen normalization). If, under repeated controlled extensions, \(H\) fails to stabilize, that is **falsification of H1** for that \(T\).

**Relation to current CLI:** `tri math compare --hybrid` implements a **concrete diagnostic** inner product (~0.564 on IEEE `f64`); it is **not** yet the general \(T\) in H1. Porting H1 into code means defining \(T\), truncation \(N\), and the renormalized \(H\) explicitly, then locking them in `.t27` + seals.

## Conjecture (falsifiable, weak form — operational)

If a renormalization-style map links Pell-weighted thin-structure data to an effective \(\phi\)-scaling law, then **extensions of the constant set** (neutrino sector, CKM, electroweak masses) should move the hybrid score **predictably** under a *stated* embedding rule — or the conjecture fails for that rule.

**Strong form (not claimed in code):** the hybrid score tends to a fixed value as the formula catalog grows. **Current code does not test convergence**; it only prints one number per run.

## What the CLI falsifies immediately

- **Naive identity \(\phi^5 = \alpha^{-1}\)** is false for the CODATA-class reference used in code (\(|\phi^5 - \alpha^{-1}| \approx 1.26\times 10^2\)). Honesty is part of the instrument design.

## Sensitivity flag (precision note)

`--sensitivity` reports a **numeric partial derivative of the L5 sum** \(\mathrm{TRINITY} = \phi^2 + \phi^{-2}\) with respect to \(\phi\), and (with `--hybrid`) of the hybrid score. This measures **stability of those scalars** under tiny \(\phi\) perturbations, not sensitivity of the whole 152-formula catalog unless each formula is wired similarly.

## Experience log (audit trail)

Each `tri math compare` run appends one JSON line to `.trinity/experience/math_compare.jsonl` (gitignored locally). Fields include flags, computed scalars, and, when `.trinity/seals/PellisFormulas.json` exists, **`pellis_spec_seal_hash`** tying the run to the sealed spec revision.

## Open work (post-merge checklist)

| Item | Status |
|------|--------|
| Replace neutrino ratio placeholder with PDG-consistent documentation | Open |
| Grow `FORMULA_TABLE.md` toward 152 rows with spec links | Open |
| Define a **testable** convergence criterion for hybrid score under catalog extension | Open |
| Optional: correlate hybrid score with seal / spec version across CI artifacts | Open |

## Collaboration anchor

For external reviewers (Pellis / Olsen / others): reproducible entrypoints are the spec file, `tri math compare`, and this note — no Python on the verification path for these commands.
