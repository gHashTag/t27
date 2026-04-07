# Hybrid conjecture — formal sketch (issue #277)

English-first research note. Implementation lives in `specs/physics/pellis-formulas.t27` and `tri math compare`.

## Definitions (as implemented today)

- **Trinity monomial ladder (diagnostic):** normalized weights from \(\phi^k\) for \(k = 0..4\).
- **Pell ladder (integer, \(\sqrt{2}\) structure):** \(P_1..P_5 = 1,2,5,12,29\) as explicit constants in the spec; CLI uses the same recurrence for the hybrid map.
- **Hybrid score:** dot product of the two normalized vectors above (dimensionless scalar). **Not** a physical observable until mapped to measured quantities.

## Conjecture (falsifiable, weak form)

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
