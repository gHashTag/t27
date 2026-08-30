# NOW -- The quantifier census read the spelling, not the keyword (2026-08-30)

## The quantifier census read the spelling, not the keyword (Refs #2774)

- `scan_clauses` matched `find("forall ")` -- the letters plus a trailing space. `specs/igla/coder/benchmark.t27:3827` writes the keyword alone, binders absent, predicate on the next line, so it matched nothing and never became a clause.
- A guard on the SPELLING decided a question about the KIND of the line. The census has a bucket for exactly that shape -- `no binder this can read` -- and the clause could not reach it.
- The spec typechecks clean, so this was a live clause in a healthy file, not debt. Clauses 922 -> 923, forall 882 -> 883, no-binder 23 -> 24; walkable held at 119, which is correct because a clause with no binder is not walkable.
- Measured before changing the matcher: of 883 forall lines, ZERO have an identifier character before the keyword, so reading it as a word cannot drop a clause this corpus already counts. The change is additive by measurement, not by hope.
- Vacuity held at 15 and that is right: the quantifier is noise but the predicate asserts something real, so `no binder` is the honest bucket and `vacuous` is not.
- New `tri quantifiers audit`: reads the corpus with the bare letters -- a deliberately looser matcher -- and names every line the strict scanner did not turn into a clause. A control sharing the strict matcher would agree by construction and measure nothing.
- Mutation-checked: restoring `find("forall ")` makes the audit name benchmark.t27:3827 and exit 1; removing the word boundary fails the counter-example test; both new tests fail under their own mutation and pass when restored.
