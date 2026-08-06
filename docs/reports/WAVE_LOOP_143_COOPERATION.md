# Cooperation Variants for Next Wave Loop (Wave Loop 144)

## Variant 1: Property Depth Push (Technical)

**Focus**: With 100% coverage achieved, shift from breadth to depth. Add second invariants to the ~209 single-inv files. Priority domains: `tri/` collections, `ml/` layers, `igla/` coder/race specs. Algebraic properties (associativity, monotonicity, round-trip) for functions; structural bounds for structs.

**Advantages**:
- Advances L4 (TESTABILITY) from "at least one" to "meaningful property depth."
- No risk of seal cascade from new files (all specs already have invariants).
- Clean metric: "average invariants per spec" becomes the new headline.

**Risks**:
- Requires understanding each spec's functions/types to write meaningful invariants (cannot rely on numeric stubs).
- Higher cognitive cost per invariant.

---

## Variant 2: Competitor Integration + arXiv Publication Prep (Research)

**Focus**: Integrate newly discovered 2026 competitors (Gresnigt Cl(10), Kulkarni cuboctahedron, Ardakanian Z3) into `benchmark.t27` scoreboard. Begin drafting Trinity ternary-unification paper sections for the arXiv 2607 window (expected late July 2026). Address #1041 (P8 Integration) by aligning spectral-action geometry with competitor frameworks.

**Advantages**:
- Maintains competitive-intelligence lead.
- Positions Trinity for 2607 publication.
- Directly advances P8 milestone.

**Risks**:
- arXiv 2607 opening is external dependency.
- Literature review and LaTeX drafting consume engineering bandwidth.

---

## Variant 3: Conformance + Infrastructure Hardening (Engineering)

**Focus**: Tackle #1184 (promote 6-wide GF rungs to bitexact_selfconsistent) and #1183 (wp18 gate). Fix remaining C-backend codegen edge cases (array inference, ANSI ports). Optimize `tri` suite runtime. Reduce stale branches below current threshold.

**Advantages**:
- Pays down technical debt.
- Improves production reliability.
- Aligns with #1141 (OpenSSF Scorecard).

**Risks**:
- Lower headline metrics (coverage already at ceiling).
- Potential regressions from codegen changes.

---

## Recommendation

**Primary**: Variant 1 (Property Depth Push) for W144. At 100% coverage, the natural evolution is depth. Target +50 second invariants across `tri/` and `ml/` specs.
**Secondary**: Variant 2 when arXiv 2607 window opens (monitor weekly).
**Background**: Variant 3 as maintenance track between coverage pushes.

---
*phi² + 1/phi² = 3 | TRINITY*
