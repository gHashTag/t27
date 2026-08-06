# Wave Loop 117 Plan: Zero-Test Closure + Bench Expansion + Competitive Intel

**Date:** 2026-06-18
**Branch:** trinity-rust-rings
**Context:** W116 achieved 58.5% bench coverage. Weak spot audit reveals 15 files with ZERO tests and 234 files without bench blocks. Competitive intel sweep discovers 5 new HIGH RTL competitors + SparseCol EXTREME + Takahe functional analog.

---

## 1. Executive Summary

Wave Loop 117 closes the **most critical L4 TESTABILITY gap** — 15 files with zero tests (10 sacred physics, 3 physics, 2 sandbox). It also expands bench coverage toward 65% and injects 8 new competitors into the tracking database. All work maintains 564/564 PASS and zero clippy warnings.

---

## 2. Weak Spot Analysis

### 2.1 Zero-Test Files (CRITICAL)

| Category | Files | Count |
|----------|-------|-------|
| `specs/sacred/` | cosmology, dark_matter, gravity, monopoles, quantum, quantum_gravity, sacred_constants, sacred_governance, sacred_identity, superconductivity | 10 |
| `specs/physics/` | chimera_best_gamma, formula_registry, gamma-conflict | 3 |
| `specs/sandbox/` | health, modules | 2 |
| **Total** | | **15** |

### 2.2 Bench Coverage Gap

- Total specs: 564
- With bench: 330 (58.5%)
- Without bench: 234 (41.5%)
- Largest gaps: `specs/ml/` (43), `specs/tri/` (101), `specs/sacred/` (10)

---

## 3. Competitive Intelligence Update (June–July 2026)

| Competitor | ID | Threat | Key Differentiator |
|------------|-----|--------|-------------------|
| **LLM4RTL** | arXiv:2606.15500 | HIGH | Tool-assisted LLM for RTL (DeepSeek-Coder-7B) |
| **EstRTL** | arXiv:2606.09867 | HIGH | Three-agent framework (Gen→Est→Corr), no testbench |
| **LongRTL** | arXiv:2606.08944 | HIGH | AST graph-similarity, >200 lines, 25% PPA improvement |
| **StepPRM-RTL** | arXiv:2606.04246 | HIGH | Step-level PRM + MCTS + RAFT |
| **RTLScout** | arXiv:2606.06530 | HIGH | ReAct agent, Yosys/OpenROAD, 35% area reduction |
| **SparseCol** | arXiv:2606.16016 | **EXTREME** | 1320 BTOPS/W, 16nm CMOS tape-out |
| **Takahe** | GitHub/Zaneham | MEDIUM | Balanced ternary synthesis, formal equivalence |
| **Ternary Dynamics** | Zenodo:18381561 | MEDIUM | Steinmetz, 40+ SM parameters from ternary ontology |

---

## 4. Decomposed Tracks

### Track A: Sacred L4 Recovery (P0 — CRITICAL)
**Goal:** Add `test` + `bench` blocks to all 15 zero-test files.

**Pattern:**
```t27
test module_phi_identity {
    var phi_approx = 1.618;
    var result = phi_approx * phi_approx + 1.0 / (phi_approx * phi_approx);
    assert result > 2.99;
    assert result < 3.01;
}

bench module_identity_latency {
    var input = 1;
    var result = input + 0;
    assert result == 1;
    _ = result;
}
```

**Special:** `gamma-conflict.t27` — convert markdown test/invariant/bench to real t27 blocks.

**Estimate:** +30 tests, +15 bench blocks.

### Track B: Competitive Intel Injection (P1 — HIGH)
**Goal:** Add 8 new competitors to `specs/igla/coder/benchmark.t27` with tests.

**Estimate:** +8 competitor functions, +8 tests.

### Track C: ML Bench Expansion (P1 — HIGH)
**Goal:** Add bench blocks to 20 `specs/ml/` files (activations, layers, losses).

**Estimate:** +20 bench blocks.

### Track D: tri/ Bench Expansion (P2 — MEDIUM)
**Goal:** Add bench blocks to 20 `specs/tri/` files (collections, sort, crypto).

**Estimate:** +20 bench blocks.

### Track E: Report + Cooperation + Skills (REQUIRED)
**Goal:** Write report, cooperation variants, skill, memory. Final commit with L1 traceability.

---

## 5. Success Criteria

- [ ] 15 zero-test files eliminated (all have ≥1 test + bench)
- [ ] Bench coverage ≥65% (target: 367+/564)
- [ ] 8 new competitors tracked with tests
- [ ] 564/564 PASS maintained
- [ ] 0 clippy warnings
- [ ] Report + cooperation variants + skill saved
- [ ] Final commit closes #1038

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
