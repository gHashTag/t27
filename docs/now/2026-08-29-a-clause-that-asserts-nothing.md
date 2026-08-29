# NOW -- A clause that asserts nothing (2026-08-29)

## A clause that asserts nothing (Refs #2774)

- Last pass found six quantified invariants that are FALSE. This pass looked in
  the same 924 clauses for the opposite defect: **vacuous** ones -- true in every
  model, whatever the functions do. A false invariant is a wrong claim; a vacuous
  one is NO claim, dressed as a checked property, and nothing will ever flag it
  because there is nothing to flag.
- The field's word, not one invented here: **vacuity** (Beer, Ben-David, Eisner &
  Rodeh). A guard never true is **antecedent failure**; true under every
  interpretation is a **tautology**.
- Measured over all 924, from the clause TEXT alone with no type table:
  **15 vacuous** -- 3 binder-unused, 6 `A == A`, 6 `X != undefined`.
  `forall a : i32, adder_tree_4(0, 0, 0, 0) == 0` quantifies over 2^32 values
  and mentions none of them.
- **Two kinds counted zero and are printed as zeros.** `P ==> P` over the 358
  clauses containing an implication; antecedent failure over 166 binder-vs-
  literal comparisons evaluated against the declared domain. Both are real
  classes in the literature and neither occurs here. A shape you can imagine is
  not a defect class until you count it.
- **The origin story did not survive.** A reading traced all of them to
  `specs/igla/` and one commit. The base rate kills it: 867 of 899
  binder-carrying clauses are already in `specs/igla` -- 96.4% -- so ten landing
  there is ~70% likely by chance. Only author clustering holds, and there is one
  author.
- **Type-level vacuity is NOT shipped**, deliberately: `x >= 0` on an unsigned,
  `x <= 255` on a u8, all enum variants listed. It needs a type table, and a
  table carries judgement -- a name with two definitions flips a verdict on a
  refactor, and a check that goes red for a reason nobody caused gets muted,
  taking the judgement-free kinds down with it.
- **Report only, no gate.** 15 hits on master; a gate red on arrival is reverted
  within a day, and a ratchet starting at 15 means "fifteen is fine".
- One-line fix landed alongside: `clause_body`'s stop list was missing `bench `,
  so an invariant at indent 0 (`gemm.t27:260`) swallowed the whole `bench` block
  after it. Measured: 1 clause in 924 overruns, 5 have an indent-0 head. Third
  list-shaped guard to go stale by addition this week.
- ci-gates 246-250. 296 tests pass; the four headline buckets unmoved.
