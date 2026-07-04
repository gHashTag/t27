# Wave Loop 116 Plan — 6 New Competitors + EDA Subprocess Rewards + Generate-Verify-Debug + Ternary Accelerator Metrics

**Date:** 2026-06-18
**Focus:** IGLA CODER + IGLA RACE
**Suite Target:** 564/564 PASS maintained

---

## Competitive Intelligence Sweep (June 2026)

6 new competitors discovered across 3 axes:

| # | Competitor | arXiv | Threat | Key Differentiator |
|---|-----------|-------|--------|-------------------|
| 1 | **RTLSeek** | 2603.27630 | HIGH | GRPO + diversity-oriented multi-objective reward + EDA feedback |
| 2 | **FormalRTL** | 2603.08738 | HIGH | Formal equivalence checking (hw-cbmc) + C/C++ reference models |
| 3 | **RWOPD** | 2605.13501 | HIGH | Reward-weighted OPD + SymbiYosys+Z3 Property-Equivalence Checker |
| 4 | **Veri-Sure** | 2601.19747 | MEDIUM-HIGH | Contract-aware multi-agent + temporal tracing + formal verification |
| 5 | **Dr. RTL** | 2604.14989 | MEDIUM-HIGH | Timing closure multi-agent (21% WNS / 17% TNS improvement) |
| 6 | **VitaLLM** | 2604.27396 | MEDIUM | TSMC 16nm ternary accelerator, 17.4 TOPS/mm²/W |

**Total tracked after W116:** 35 competitors (29 → +6)

---

## Weaknesses to Close

1. **No EDA subprocess reward primitives** — eval.t27 has stubs for Yosys/Verilator/Icarus but no real reward extraction
2. **No generate-verify-debug loop** — Veri-Sure and FormalRTL both use this; Trinity has no temporal tracing or contract awareness
3. **No diversity reward** — RTLSeek's core innovation; Trinity has no structural diversity metric
4. **No property equivalence checking** — RWOPD uses SymbiYosys+Z3; Trinity has no formal property checker integration
5. **No ternary accelerator efficiency metrics** — VitaLLM tracks TOPS/mm²/W; Trinity race specs lack energy efficiency primitives

---

## Implementation Tracks

### Track A: Competitive Tracking (+6 competitors) → benchmark.t27
- `rtlseek_competitor()` — HIGH, GRPO + diversity reward
- `formalrtl_competitor()` — HIGH, hw-cbmc formal equivalence
- `rwopd_competitor()` — HIGH, reward-weighted OPD + PEC
- `verisure_competitor()` — MEDIUM-HIGH, contract-aware multi-agent
- `drrtl_competitor()` — MEDIUM-HIGH, timing closure agent
- `vitallm_competitor()` — MEDIUM, ternary accelerator 17.4 TOPS/mm²/W

### Track B: EDA Subprocess Reward Primitives → eval.t27
- `score_rtl_with_yosys_real(rtl) -> f32` — latency reward from Yosys synthesis (stub with realistic fallback)
- `diversity_reward(generated_a, generated_b) -> f32` — AST structural diversity score (Hamming-like)
- `property_equivalence_reward(rtl, spec) -> f32` — formal property check reward stub

### Track C: Generate-Verify-Debug Loop → pipeline.t27
- `Contract` type — temporal assertion contract for RTL modules
- `generate_verify_debug(spec, contracts) -> []AgentAction` — iterative debug loop
- `temporal_trace_check(rtl, contract) -> bool` — temporal tracing stub

### Track D: Ternary Accelerator Efficiency → race/*.t27
- `compute_tops_per_mm2_w(throughput, area_mm2, power_w) -> f32` — VitaLLM efficiency metric
- `energy_efficiency_reward(tokens_per_sec, watts) -> f32` — energy per token
- Add bench blocks to race specs still missing them

### Track E: Bench Block Hygiene → specs/tri/
- Add bench blocks to remaining specs/tri/ files without them (if any)

---

## Verification Criteria

- 564/564 PASS
- 0 seal mismatches
- 0 clippy warnings
- All new competitors have tests
- All new functions have tests

---

**phi² + 1/φ² = 3 | TRINITY**
