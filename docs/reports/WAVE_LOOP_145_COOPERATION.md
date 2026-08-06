# Cooperation Variants for Next Wave Loop (Wave Loop 146)

## Variant 1: Accelerated Depth Push to 2.25 Avg (Technical)

**Focus**: Continue adding second invariants to remaining 200 single-inv files. Target +40 second invariants, pushing avg from 2.12 → 2.25. Priority domains: `tri/collections/` (remaining 15 single-inv), `tri/utils/` (remaining 10), `tri/search/` (8), `tri/sort/` (6), `tri/encoding/` (5). Emphasize algebraic properties (associativity, commutativity, idempotence) for collections; monotonicity for sorts; round-trip for encodings.

**Advantages**:
- Largest single-wave depth gain possible.
- Proven parser-safe patterns (all 45 second invariants from W144–W145 passed).
- Clear metric progression.

**Risks**:
- Cognitive cost: +40 invariants requires understanding 40 distinct specs.
- Some remaining single-inv files are pure stubs (sacred/ cosmology, dark_matter, etc.) forcing numeric placeholders rather than semantic depth.

---

## Variant 2: Competitor Integration + Benchmark Taxonomy Expansion (Research)

**Focus**: Integrate Baroň (arXiv:2606.08459) into `benchmark.t27` as HIGH threat. Add previously queued competitors (Gresnigt, Kulkarni, Triality-Resolved Spectral Update Theory) if not yet integrated. Expand `docs/COMPETITIVE_POSITIONING.md` with ternary hierarchy analysis comparing Baroň to Trinity's φ-seesaw framework.

**Advantages**:
- Maintains competitive-intelligence lead.
- Addresses #1041 (P8 Integration) by expanding benchmark taxonomy.
- Positions Trinity for rapid-response to any 2607 papers.

**Risks**:
- Does not improve invariant metrics.
- Time-intensive literature review.

---

## Variant 3: Conformance Sprint + CI Optimization (Engineering)

**Focus**: Tackle #1184 (6-wide GF rungs) and #1183 (wp18 gate) in earnest. Reduce `tri` suite runtime by optimizing seal verification batching. Fix any lingering C-backend array inference edge cases. Prune stale branches below current threshold.

**Advantages**:
- Pays down technical debt.
- Improves CI velocity.
- Aligns with #1141 OpenSSF Scorecard.

**Risks**:
- No headline invariant metric improvement.
- Potential regressions from codegen changes.

---

## Recommendation

**Primary**: Variant 1 (Accelerated Depth Push) for W146. Continue depth trajectory 2.12 → 2.25.
**Secondary**: Variant 2 when arXiv 2607 opens or when new HIGH/EXTREME competitor surfaces.
**Background**: Variant 3 as maintenance sprint every 3 waves.

---
*phi² + 1/phi² = 3 | TRINITY*
