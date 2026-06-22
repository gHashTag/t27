# Wave Loop 127 Plan — IGLA CODER + IGLA RACE

**Date:** 2026-06-16
**Scope:** Close weakest coverage gaps + competitive intelligence + seal verification

---

## Track A — IGLA RACE Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/race/systolic_ternary.t27` | 9 tests, 3 benches | 11 tests, 3 benches | +2 tests |
| `specs/igla/race/cordic.t27` | 10 tests, 2 benches | 12 tests, 2 benches | +2 tests |
| `specs/igla/race/cordic_top.t27` | 10 tests, 2 benches | 12 tests, 2 benches | +2 tests |
| `specs/igla/race/gemm.t27` | 10 tests, 2 benches | 12 tests, 2 benches | +2 tests |

**Rationale:** These are the 4 weakest IGLA RACE specs. Each receives exactly 2 tests.

---

## Track B — IGLA CODER Test Expansion (Weakest Specs)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `specs/igla/coder/bench_proxy.t27` | 20 tests, 3 benches | 22 tests, 3 benches | +2 tests |
| `specs/igla/coder/prm.t27` | 22 tests, 3 benches | 24 tests, 3 benches | +2 tests |
| `specs/igla/coder/training.t27` | 22 tests, 3 benches | 24 tests, 3 benches | +2 tests |

**Rationale:** Proportional growth on the 3 weakest CODER specs.

---

## Track C — Competitive Intelligence

| Competitor | Source | Threat |
|-----------|--------|--------|
| **LLM4RTL** | arXiv:2606.15500 (June 2026) | HIGH — tool-assisted LLM with JRCRC pipeline, achieves GPT-4O parity on VerilogEval-human using smaller models (DeepSeek-Coder-7B). Cost-efficient RTL generation |
| **TeLLMe v2** | arXiv:2510.15926v1 (Oct 2025) | MEDIUM-HIGH — end-to-end ternary LLM FPGA accelerator, 25 tokens/s decode at 5W. Evolution of known competitor |

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
- 129 competitors tracked
- All weakest specs show measurable coverage improvement

φ² + 1/φ² = 3
