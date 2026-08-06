# Wave Loop 108 Report — IGLA CODER / IGLA RACE Gap Closure + Pipeline Wiring

**Date:** 2026-06-16
**Branch:** trinity-rust-rings
**Suite:** 564/564 PASS | Clippy: 0 warnings | Seal mismatches: 0
**Commit count:** 361 (pre-commit)

---

## Executive Summary

Wave Loop 108 closes four infrastructure gaps identified in W107 retrospective:
1. **Orphaned diversity functions** — W107 added `generate_diverse_candidates` and `reject_resample_for_sacred`, but they were never called from the main generation pipeline.
2. **No synthesis comparison primitives** — Yosys report parsing existed, but no `compare_synth_reports` or normalized `synthesis_score`.
3. **No Verilator lint integration** — linting is a standard EDA step; Trinity had no conceptual bridge.
4. **No round-trip reasoning** — RTL-BenchLS introduced round-trip reasoning and repository-issue fixing as novel tasks; Trinity had no conceptual stubs.

Additionally, **OpenRTLSet** (arXiv:2606.10285v1) was discovered — a 131K-module open-source dataset that achieves 89.3% Pass@10 when fine-tuning Qwen2.5-32B. This represents a **dataset-scale threat** that Trinity cannot match with its ~1,280 conceptual samples.

---

## Honest Gap Assessment (W107 → W108)

| Gap | Severity | Status |
|-----|----------|--------|
| Diversity + sacred filtering orphaned in pipeline | **HIGH** | ✅ Closed — `generate_verilog_ai_with_diversity_and_sacred` |
| No synthesis comparison / normalized scoring | **MEDIUM** | ✅ Closed — `compare_synth_reports`, `synthesis_score` |
| No Verilator lint conceptual bridge | **MEDIUM** | ✅ Closed — `run_verilator_lint`, `parse_verilator_lint` |
| No round-trip reasoning / repo-issue fixing | **MEDIUM** | ✅ Closed — `compress_rtl_to_abstract`, `round_trip_accuracy`, `fix_repository_issue` |
| No OpenRTLSet competitor tracking | **HIGH** | ✅ Closed — competitor preset + dataset scale comparison |

---

## New Competitive Intelligence (June 2026)

### OpenRTLSet — Dataset-Scale Threat (arXiv:2606.10285v1)

| Attribute | OpenRTLSet | Trinity S³AI |
|-----------|------------|--------------|
| **Dataset size** | **131,000 modules** | ~1,280 conceptual samples |
| **Data sources** | GitHub (102K), VHDL translations (5K), HLS-generated (24K) | Hand-written templates + composition |
| **Model size** | Qwen2.5-32B | Sub-1B (conceptual) |
| **Pass@10** | **89.3%** on VerilogEval-Machine | **Not measured** |
| **Open source** | ✅ Fully open | ✅ Specs open, model weights conceptual |
| **Differentiation** | Scale | Sacred constraint (R-SI-1), formal verification bridge |

**Key insight:** OpenRTLSet proves that **dataset scale dominates model size** for RTL generation. A 32B model on 131K modules outperforms most smaller models. Trinity's ~1,280 samples are **100× smaller**. The dataset gap is now the primary blocker to competitive Pass@K scores.

---

## Track-by-Track Implementation

### Track A — OpenRTLSet + Dataset Scale Metrics (`benchmark.t27`)

**New functions:**
- `openrtlset_competitor()` — Pass@10 = 0.893, benchmark = "VerilogEval-machine (dataset-scale)".
- `dataset_scale_competitor_comparison(trinity_size, competitor_size) -> f32` — returns size ratio. For Trinity 1,280 vs OpenRTLSet 131,000: **ratio = 0.0098** (less than 1%).

**Tests added:** 4 (competitor score, parity ratio, smaller ratio, zero guard).

---

### Track B — Diversity + Sacred Wired into Pipeline (`pipeline.t27`)

**New functions:**
- `generate_verilog_ai_with_diversity_and_sacred(prompt, bank, cfg) -> string` — end-to-end pipeline:
  1. Generates 4 diverse candidates via `generate_diverse_candidates`.
  2. Filters to R-SI-1 compliant via `reject_resample_for_sacred`.
  3. Scores remaining candidates with `eval::score_rtl_with_yosys`.
  4. Selects best via `select_best_synth_candidate` (recursive PPA comparison).
  5. Falls back to `generate_verilog_ai_with_steering` if no sacred candidates survive.

- `select_best_synth_candidate(candidates, idx, best_report) -> string` — recursive YosysReport comparison across candidate list.

**Tests added:** 3 (non-empty output, single candidate, synthesis preference).

---

### Track C — Verilator Lint + Synthesis Comparison (`eval.t27`)

**New functions:**
- `run_verilator_lint(rtl) -> bool` — conceptual Verilator lint runner (checks module/endmodule presence).
- `parse_verilator_lint(log) -> bool` — parses `%Error` prefix from Verilator stdout.
- `compare_synth_reports(a, b) -> YosysReport` — named wrapper around `compare_ppa_reports`.
- `synthesis_score(report) -> f32` — normalized quality score [0.0, 1.0]:
  - 0.0 = synthesis failed
  - 0.5 = synthesis succeeded
  - +0.15 = LUT < 200
  - +0.30 = LUT < 50
  - +0.10 = MHz > 100
  - +0.20 = MHz > 200

**Tests added:** 6 (lint valid/invalid, parse pass/error, synth comparison, score failed/excellent).

---

### Track D — Round-Trip Reasoning + Repository-Issue Fixing (`dataset.t27`)

**New functions:**
- `compress_rtl_to_abstract(rtl) -> string` — conceptual RTL → natural-language abstract (inspired by RTL-BenchLS round-trip reasoning).
- `regenerate_from_abstract(abstract) -> string` — conceptual abstract → RTL regeneration.
- `round_trip_accuracy(original, regenerated) -> f32` — character-level overlap ratio [0.0, 1.0].
- `fix_repository_issue(rtl, issue_description) -> string` — conceptual bug-fix from GitHub issue description (inspired by RTL-BenchLS repository-issue reasoning).

**Tests added:** 4 (compress valid/invalid, perfect accuracy, zero accuracy, issue fix append).

---

## Suite Impact

| Metric | W107 | W108 | Δ |
|--------|------|------|---|
| Total specs | 564 | 564 | — |
| Parse | 564 | 564 | — |
| Typecheck | 564 | 564 | — |
| Gen Zig | 564 | 564 | — |
| Gen Rust | 564 | 564 | — |
| Gen Verilog | 564 | 564 | — |
| Gen C | 564 | 564 | — |
| Seal verify | 564 | 564 | — |
| New functions (across 4 files) | — | ~12 | — |
| New tests (across 4 files) | — | ~17 | — |
| Clippy warnings | 0 | 0 | — |

---

## Supplementary Track E — Competitive Intelligence Expansion (Post-Report Update)

Added 6 additional competitors to `benchmark.t27` after initial report:

**RTL Generation:**
- **VerilogCL** (arXiv:2604.18162) — Contrastive learning for robust generation
- **EvolVE** (arXiv:2601.18067) — 98.1% VerilogEval v2, evolutionary+MCTS
- **VeriGraphi** (arXiv:2604.14550v2) — Multi-agent hierarchical RTL
- **LLM4RTL-2026** (arXiv:2606.15500) — Tool-assisted JRCRC pipeline

**Physics Formal Verification:**
- **PhysicsAsCode-SU5** (arXiv:2603.28406) — SU(5) GUT in Lean 4 (Krippendorf + Tooby-Smith)
- **SK_EFT_Hawking** (GitHub/NetRxn) — 9944 Lean 4 theorems, 0 sorry, SM fingerprints

**Tests added:** 6 (one per competitor name/benchmark verification).

## Supplementary Track F — L4 Benchmark Expansion (Post-Report Update)

Added bench blocks to 4 naked specs:
- `specs/compiler/lexer.t27` — 2 bench blocks (lexer_init_latency, keyword_check_latency)
- `specs/compiler/parser.t27` — 2 bench blocks (parser_init_latency, node_kind_comparison_latency)
- `specs/brain/bus.t27` — 1 bench block (brain_bus_version_latency)
- `specs/brain/unified_state.t27` — 2 bench blocks (brain_state_init_latency, phi_coherence_latency)

**Suite:** 564/564 PASS, 0 seal mismatches after regeneration.

## Remaining Honest Gaps (W108 → W109)

| Gap | Severity | Notes |
|-----|----------|-------|
| **No empirical Pass@K score** | **CRITICAL** | Infrastructure complete; missing GPU/API budget |
| **90+ point gap to VeriAgent** | **CRITICAL** | Cannot close without trained model + real inference |
| **No real subprocess for Yosys/Icarus/Verilator** | **HIGH** | All synthesis/simulation/lint functions are conceptual stubs |
| **Dataset 100× smaller than OpenRTLSet** | **HIGH** | ~1,280 vs 131,000 modules; compositional expansion insufficient |
| **No PRM training loop** | **HIGH** | Static PRM vs trained PRM (StepPRM-RTL advantage) |
| **Round-trip reasoning is stub-only** | **MEDIUM** | No real LLM-based compression/regeneration |
| **.tri syntax stubs remain** | **MEDIUM** | 9 unmigrated stub files |
| **61 specs still have TODOs/placeholders** | **MEDIUM** | Gradual cleanup needed |

---

## Security & Compliance

- L1 TRACEABILITY: Plan-driven; no new issue closure this wave (infrastructure focus).
- L2 GENERATION: `gen/` untouched; spec edits only.
- L3 PURITY: ASCII-only, English identifiers.
- L4 TESTABILITY: 17+ new tests across 4 files; every new function covered.
- L7 UNITY: No new `.sh` on critical path.

---

## Conclusion

Wave Loop 108 is a **wiring wave** — it connects previously orphaned functions (diversity, sacred filtering, synthesis comparison) into a cohesive generation pipeline. The key architectural addition is `generate_verilog_ai_with_diversity_and_sacred`, which for the first time combines:
- **Diversity-oriented candidate generation**
- **R-SI-1 sacred constraint filtering**
- **Yosys PPA-based candidate ranking**
- **Steering fallback** for low-quality generations

The discovery of **OpenRTLSet** (131K modules, 89.3% Pass@10) makes it clear that **dataset scale is the dominant competitive factor** in RTL generation. Trinity's sustainable differentiation remains:
1. **R-SI-1 sacred invariant** — no competitor enforces this.
2. **Formal verification bridge** — Coq/Lean proof-as-reward is unique.
3. **Spec-first TDD** — 564 specs provide deterministic regression.

But without **empirical Pass@K scores** and **real dataset scale**, these advantages are theoretical. The next wave must prioritize either (a) real EDA toolchain integration, (b) dataset scale expansion to 10K+, or (c) empirical evaluation to establish honest baseline numbers.

---

**phi² + 1/φ² = 3 | TRINITY**
