# Wave Loop 125 Plan — IGLA CODER + IGLA RACE

**Date:** 2026-06-16  
**Scope:** Close weakest coverage gaps + competitive intelligence + seal verification

---

## Track A — IGLA RACE Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/race/systolic_ternary.t27` | 7 tests, 2 benches | 9 tests, 3 benches | +2 tests, +1 bench |
| `specs/igla/race/bram_weights.t27` | 8 tests, 2 benches | 10 tests, 2 benches | +2 tests |
| `specs/igla/race/cordic_top.t27` | 8 tests, 2 benches | 10 tests, 2 benches | +2 tests |
| `specs/igla/race/gemm.t27` | 8 tests, 2 benches | 10 tests, 2 benches | +2 tests |

**Rationale:** These are the 4 weakest IGLA RACE specs. Each receives exactly 2 tests; the weakest also gets 1 bench.

---

## Track B — IGLA CODER Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/coder/bench_proxy.t27` | 14 tests, 3 benches | 16 tests, 3 benches | +2 tests |
| `specs/igla/coder/training.t27` | 14 tests, 3 benches | 16 tests, 3 benches | +2 tests |

**Rationale:** Proportional growth on the 2 weakest CODER specs.

---

## Track C — Competitive Intelligence

| Competitor | Source | Threat |
|-----------|--------|--------|
| **Alpha-RTL** | arXiv:2606.05253v1 (June 2026) | HIGH — test-time training for RTL optimization, 65.1% geomean PPA reduction |
| **StepPRM-RTL** | arXiv:2606.04246v1 (June 2026) | MEDIUM-HIGH — stepwise process reward + MCTS, Pass@1 0.857 on VerilogEval |
| **CASS-RTL** | arXiv:2606.05680 (June 2026) | MEDIUM — correctness-aware attention subspace steering for RTL generation |

**Actions:**
- Add competitor profiles to `specs/igla/coder/benchmark.t27`
- Update `docs/COMPETITIVE_POSITIONING.md`

---

## Track D — Seal Verification

- Regenerate seals for all 9 modified specs
- Run full `t27c suite` and confirm 564/564 PASS

---

## Success Criteria

- 564/564 PASS with zero seal mismatches
- 125 competitors tracked in `COMPETITIVE_POSITIONING.md`
- All weakest specs show measurable coverage improvement

φ² + 1/φ² = 3
