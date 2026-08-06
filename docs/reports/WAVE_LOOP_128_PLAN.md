# Wave Loop 128 Plan — IGLA CODER + IGLA RACE

**Date:** 2026-06-16
**Scope:** Close weakest coverage gaps + competitive intelligence + seal verification

---

## Track A — IGLA RACE Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/race/systolic_array.t27` | 10 tests, 2 benches | 12 tests, 2 benches | +2 tests |
| `specs/igla/race/ternary_mac.t27` | 10 tests, 2 benches | 12 tests, 2 benches | +2 tests |
| `specs/igla/race/adder_tree.t27` | 10 tests, 3 benches | 12 tests, 3 benches | +2 tests |
| `specs/igla/race/opcodes.t27` | 10 tests, 3 benches | 12 tests, 3 benches | +2 tests |

**Rationale:** These are the 4 weakest IGLA RACE specs. Each receives exactly 2 tests.

---

## Track B — IGLA CODER Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/coder/bench_proxy.t27` | 22 tests, 3 benches | 24 tests, 3 benches | +2 tests |
| `specs/igla/coder/weights.t27` | 22 tests, 3 benches | 24 tests, 3 benches | +2 tests |
| `specs/igla/coder/tokenizer.t27` | 23 tests, 3 benches | 25 tests, 3 benches | +2 tests |

**Rationale:** Proportional growth on the 3 weakest CODER specs.

---

## Track C — Competitive Intelligence

| Competitor | Source | Threat |
|-----------|--------|--------|
| **CHIMERA** | arXiv:2606.02358v1 (June 2026) | HIGH — 22nm AI-MCU with transformer accelerator, 3.1 TOPS/W, 281 GOPS/mm². Edge AI silicon |
| **TRINE** | arXiv:2603.22867 (March 2026) | MEDIUM-HIGH — multimodal FPGA inference engine, 22.57× vs RTX 4090, single-bitstream |

**Actions:**
- Add competitor profiles to `specs/igla/coder/benchmark.t27`
- Update `docs/COMPETITIVE_POSITIONING.md`

---

## Track D — Seal Verification

- Regenerate seals for all modified specs
- Run full `t27c suite` and confirm 564/564 PASS

---

## Success Criteria

- 564/564 PASS with zero seal mismatches
- 131 competitors tracked
- All weakest specs show measurable coverage improvement

φ² + 1/φ² = 3
