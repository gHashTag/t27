# Wave Loop 126 Plan — IGLA CODER + IGLA RACE

**Date:** 2026-06-16  
**Scope:** Close weakest coverage gaps + competitive intelligence + seal verification

---

## Track A — IGLA RACE Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/race/eda.t27` | 8 tests, 3 benches | 10 tests, 3 benches | +2 tests |
| `specs/igla/race/cordic_fixed.t27` | 9 tests, 3 benches | 11 tests, 3 benches | +2 tests |
| `specs/igla/race/backend.t27` | 10 tests, 2 benches | 12 tests, 3 benches | +2 tests, +1 bench |
| `specs/igla/race/bram_weights.t27` | 10 tests, 2 benches | 12 tests, 2 benches | +2 tests |

**Rationale:** These are the 4 weakest IGLA RACE specs after excluding those already expanded in W125. Each receives exactly 2 tests for proportional growth.

---

## Track B — IGLA CODER Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/coder/bench_proxy.t27` | 18 tests, 3 benches | 20 tests, 3 benches | +2 tests |
| `specs/igla/coder/training.t27` | 20 tests, 3 benches | 22 tests, 3 benches | +2 tests |
| `specs/igla/coder/tokenizer.t27` | 21 tests, 2 benches | 23 tests, 2 benches | +2 tests |

**Rationale:** Proportional growth on the 3 weakest CODER specs.

---

## Track C — Competitive Intelligence

| Competitor | Source | Threat |
|-----------|--------|--------|
| **CktFormalizer** | arXiv:2605.07782v2 (May 2026) | **EXTREME** — Lean 4 dependently-typed HDL, formal equivalence proofs, OpenROAD tapeout. Direct competitor to Trinity's spec-first + Coq approach |
| **FormalRTL** | arXiv:2603.08738v1 (March 2026) | HIGH — C/C++ golden model as executable formal spec, hw-cbmc equivalence checking, industrial datapath blocks >1k LoC RTL |

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
- 127 competitors tracked in `COMPETITIVE_POSITIONING.md`
- All weakest specs show measurable coverage improvement

φ² + 1/φ² = 3
