# Wave Loop 124 Plan — IGLA CODER + IGLA RACE

**Date:** 2026-06-16  
**Scope:** Close weakest coverage gaps + competitive intelligence + seal verification

---

## Track A — IGLA RACE Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/race/eda.t27` | 6 tests, 3 benches | 8 tests, 3 benches | +2 tests |
| `specs/igla/race/cordic_fixed.t27` | 7 tests, 2 benches | 9 tests, 3 benches | +2 tests, +1 bench |
| `specs/igla/race/formal.t27` | 7 tests, 2 benches | 9 tests, 2 benches | +2 tests |
| `specs/igla/race/backend.t27` | 8 tests, 2 benches | 10 tests, 2 benches | +2 tests |

**Rationale:** These are the 4 weakest IGLA RACE specs after W123 improvements. Each receives exactly 2 tests to maintain proportional growth.

---

## Track B — IGLA CODER Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/coder/bench_proxy.t27` | 12 tests, 2 benches | 14 tests, 3 benches | +2 tests, +1 bench |
| `specs/igla/coder/tokenizer.t27` | 17 tests, 2 benches | 19 tests, 2 benches | +2 tests |
| `specs/igla/coder/weights.t27` | 19 tests, 2 benches | 21 tests, 2 benches | +2 tests |

**Rationale:** These are the 3 weakest IGLA CODER specs. Proportional growth with 2 tests each, 1 bench for the weakest.

---

## Track C — Competitive Intelligence

| Competitor | Source | Threat |
|-----------|--------|--------|
| **RTLScout** | arXiv:2606.06530v1 (June 2026) | HIGH — agentic code + synthesis optimization, 35% area / 45% delay reduction |
| **EstRTL** | arXiv:2606.09867 (June 2026) | MEDIUM-HIGH — functional estimation without testbenches, 3-stage agent |

**Actions:**
- Add competitor profiles to `specs/igla/coder/benchmark.t27`
- Update `docs/COMPETITIVE_POSITIONING.md`

---

## Track D — Seal Verification

- Regenerate seals for all 7 modified specs
- Run full `t27c suite` and confirm 564/564 PASS

---

## Success Criteria

- 564/564 PASS with zero seal mismatches
- 122 competitors tracked in `COMPETITIVE_POSITIONING.md`
- All weakest specs show measurable coverage improvement

φ² + 1/φ² = 3
