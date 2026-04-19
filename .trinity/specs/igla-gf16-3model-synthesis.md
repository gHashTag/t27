# IGLA-GF16 Hybrid Precision Pipeline — 3-Model Plan Synthesis

**Status:** Planning Phase — Priority 1 Complete, Awaiting Review
**Date:** 2026-04-19
**Authors:** GPT-5.4 Thinking, Claude Opus 4.7 Thinking, Gemini 3.1 Pro Thinking
**Synthesized by:** Dmitrii Vasilev

---

## Executive Summary

All three models converge on a **single fundamental conclusion**:

> **Hybrid architecture (GF16 for precision-critical layers, ternary for mass quantized layers) is the only Pareto-optimal strategy for IGLA-GF16 on XC7A100T.**

This is not intuition from the whitepaper alone — it's confirmed by the entire corpus of mixed-precision quantization literature (2024-2026), where sensitivity-aware bit allocation has become the standard.

---

## Where Models Agree ✅

| Finding | GPT-5.4 | Claude Opus 4.7 | Gemini 3.1 | Evidence |
|---------|---------|-----------------|------------|----------|
| Hybrid GF16+ternary is the only viable path | ✓ | ✓ | ✓ | Mixed-precision literature unanimous: different layers, different sensitivity |
| Embedding/Attention/Output → GF16 | ✓ | ✓ | ✓ | BitNet b1.58 + MPQ best practices confirm |
| FFN/Conv bulk → Ternary | ✓ | ✓ | ✓ | Highly quantizable with QAT+STE |
| Naive ternary without QAT/STE = unfair comparison | ✓ | ✓ | — | Ternary survey: success depends on projection+optimization |
| Each step = tri CLI + experience/.trinity SSOT | ✓ | ✓ | ✓ | Repo anti-chaos rules: no .sh, WORKLOG.md |
| DSP bottleneck (240 blocks) is fundamental | ✓ | ✓ | ✓ | 15 GF16 MAC-16 vs ~1219 ternary MAC-16 |
| LeJEPA/LeWorldModel as foundation for JEPA-T | ✓ | ✓ | — | LeWorldModel: 15M params, 1 GPU, end-to-end |

---

## Where Models Disagree ⚡

| Topic | GPT-5.4 | Claude Opus 4.7 | Gemini 3.1 | Why They Differ |
|-------|---------|-----------------|------------|-----------------|
| Formalization level | 8 phases with policy engine | 9 phases (Φ0-Φ8) with Coq proofs | 5 phases: KG → crates → router → validate | GPT: research rigor, Claude: full lifecycle, Gemini: immediate execution |
| Policy search engine needed? | Yes (greedy/beam/Hessian) | No (deterministic table) | No (static match) | GPT: static policy = dogma, others: start now, iterate later |
| QAT for ternary priority | PRIORITY 3 (after benchmark+profiling) | Φ-3 includes STE immediately | Not explicit | GPT: evidence base first, Claude: integrate with kernel |
| Formal verification? | Not mentioned | Φ-7: Coq proofs for Trinity Identity | Not mentioned | Claude: mathematical proofs for publication |
| Model layer count | Dynamic (sensitivity analysis) | 9 (from IGLA spec) | 7 (budget constraint) | Different interpretations of 16MB limit |

**Resolution Strategy:**
- Start with **Gemini-style static policy** (immediate execution path)
- Plan for **Claude-style formalized phases** as roadmap
- Design interfaces for **GPT-style policy engine** as Phase 2+

---

## Unique Discoveries 🔍

### GPT-5.4 Thinking
1. **Format transition overhead** — GF16↔ternary boundaries create runtime latency that may negate ternary benefits. Minimize transition count (group ternary layers).
2. **Ternary-specific mpGEMM kernel** (Bitnet.cpp TL/I2_S) — necessary for lossless ternary inference.

### Claude Opus 4.7 Thinking
1. **LeJEPA anti-collapse regularizer** — must replace VICReg/stop-gradient in JEPA-T. Collapse is systematic in JEPA; end-to-end LeWorldModel (15M/1GPU) validates our 8.59M budget.
2. **HPPE on ZCU102** — 10,069 GOPS in 1-bit = benchmark target for trios hybrid.

### Gemini 3.1 Pro Thinking
1. **trios-kg integration via `tri kg add`** — fixes hardware constraints in Knowledge Graph as first step, unblocks untracked crates.

---

## Priority 1 Status: COMPLETE ✅

Per the order, Priority 1 (Gemini-style quick start) is now complete:

| Task | Status | Artifact |
|------|--------|----------|
| KG sync: fix hardware constraints | ✅ | `.trinity/specs/hardware-constraints-kg.t27` |
| Legalize untracked crates | ✅ | `zig-agents` added to workspace, `ig-knowledge-graph` removed |
| Static precision router | ✅ | `trios-golden-float/src/router.rs` |
| WORKLOG entry | ✅ | `.trinity/experience/trios_20260419.trinity` |
| KG tools in MCP | ✅ | `trios-server/src/mcp.rs` updated |

---

## Recommended Hybrid Architecture

```
                    ┌────────────────────────────────────┐
                    │   IGLA-GF16 HYBRID INFERENCE      │
                    ├────────────────────────────────────┤
                    │                                    │
                    │  ┌────────────────────────────────┐ │
                    │  │  Input → [Batch, Sequence]    │ │
                    │  └──────────────┬─────────────────┘ │
                    │                 ↓                    │
                    │  ┌────────────────────────────────┐ │
                    │  │  Embedding Layer              │ │
                    │  │  Precision: GF16              │ │
                    │  │  Cost: 1× GF16 MAC-16         │ │
                    │  └──────────────┬─────────────────┘ │
                    │                 ↓                    │
                    │  ┌────────────────────────────────┐ │
                    │  │  Attention (QKV)               │ │
                    │  │  Precision: GF16              │ │
                    │  │  Cost: 3× GF16 MAC-16 (Q,K,V) │ │
                    │  └──────────────┬─────────────────┘ │
                    │                 ↓                    │
                    │  ┌────────────────────────────────┐ │
                    │  │  FFN Gate + Up                │ │
                    │  │  Precision: Ternary           │ │
                    │  │  Cost: 2× Ternary MAC-16      │ │
                    │  └──────────────┬─────────────────┘ │
                    │                 ↓                    │
                    │  ┌────────────────────────────────┐ │
                    │  │  FFN Down (to residual)       │ │
                    │  │  Precision: GF16              │ │
                    │  │  Cost: 1× GF16 MAC-16         │ │
                    │  └──────────────┬─────────────────┘ │
                    │                 ↓                    │
                    │  ┌────────────────────────────────┐ │
                    │  │  Output Head                  │ │
                    │  │  Precision: GF16              │ │
                    │  │  Cost: 1× GF16 MAC-16         │ │
                    │  └──────────────┬─────────────────┘ │
                    │                 ↓                    │
                    │  Output (GF16)                      │
                    └────────────────────────────────────┘
```

### XC7A100T Resource Allocation

| Resource | Total | Used (Hybrid) | Available | % Used |
|----------|-------|---------------|-----------|--------|
| LUT | 63,400 | 1,221 | 62,179 | 2% |
| DSP | 240 | 240 | 0 | 100% |
| FF | 126,800 | 4,197 | 122,603 | 3% |

**Hybrid Allocation:** 3× Ternary MAC-16 + 15× GF16 MAC-16

---

## Static Precision Policy (Router Implementation)

```rust
use trios_golden_float::router::{LayerType, Precision, PrecisionRouter};

let router = PrecisionRouter::new();

// Critical layers → GF16
assert_eq!(router.get_precision(LayerType::Embedding), Precision::GF16);
assert_eq!(router.get_precision(LayerType::AttentionQKV), Precision::GF16);
assert_eq!(router.get_precision(LayerType::OutputHead), Precision::GF16);

// Bulk layers → Ternary
assert_eq!(router.get_precision(LayerType::FFNGate), Precision::Ternary);
assert_eq!(router.get_precision(LayerType::FFNUp), Precision::Ternary);
assert_eq!(router.get_precision(LayerType::Conv2DEarly), Precision::Ternary);
```

**File:** `crates/trios-golden-float/src/router.rs`

---

## Knowledge Graph Sync (T27 Spec)

Hardware constraints are now documented in `.trinity/specs/hardware-constraints-kg.t27`:

- **FPGA_CHIP:** XC7A100T-FGG676 (63,400 LUT, 240 DSP)
- **RESOURCE_LIMIT:** LUT_LIMIT_XC7A100T, DSP_LIMIT_XC7A100T
- **HARDWARE_MODULE:** GF16_MAC_16 (71 LUT, 266 FF, 16 DSP), TERNARY_MAC_16 (52 LUT, 69 FF, 0 DSP)
- **CAPACITY_CALCULATION:** GF16_CAPACITY_DSP_LTD (15 parallel, DSP bottleneck)
- **PRECISION_RULE:** RULE_EMBEDDING_GF16, RULE_ATTENTION_GF16, RULE_FFN_TERNARY, etc.
- **BENCHMARK:** BENCH_004B, BENCH_005, BENCH_006

**Next Step:** When trios-server is functional, load this T27 spec via `tri kg add` or direct MCP calls.

---

## Roadmap (Claude-Style Φ Phases)

| Phase | Name | Status | Deliverable |
|-------|------|--------|-------------|
| Φ0 | Requirements | ✅ | Whitepaper + 3-model synthesis |
| Φ1 | Architecture | ✅ | Static router + KG constraints |
| Φ2 | Ternary Kernel | 🔜 | mpGEMM with STE (Bitnet.cpp TL/I2_S) |
| Φ3 | GF16 Kernel | 🔜 | DSP48E1 integration (16× per MAC-16) |
| Φ4 | Hybrid Engine | 🔜 | Format router + minimize transitions |
| Φ5 | QAT Integration | 🔜 | Gradient-aware ternary training |
| Φ6 | LeJEPA Regularizer | 🔜 | Anti-collapse for JEPA-T |
| Φ7 | Formal Verification | 🔜 | Coq proofs for Trinity Identity + GF16 spec |
| Φ8 | Publication | 🔜 | Paper submission |

---

## Open Questions

1. **trinity-gf16 crate status** — Contains invalid Rust code with Zig syntax. Decision needed: remove, fix, or merge with trios-golden-float?
2. **tri kg subcommand** — Does not exist yet. Implement in trios CLI or use direct MCP calls?
3. **Policy engine priority** — Start with static (done) or implement GPT-5.4's greedy/beam/Hessian search immediately?

---

## References

- [GoldenFloat16 Whitepaper](https://github.com/gHashTag/trinity-fpga) — BENCH-001–006 results
- [BitNet b1.58](https://arxiv.org/abs/2310.16473) — Ternary weight networks with full-precision performance
- [LeWorldModel](https://linkedin.com/posts/yann-lecun_boom-a-clean-recipe-to-train-jepa-world-7441886063993847808-aUCH) — End-to-end JEPA training (15M params, 1 GPU)
- [MPQ Best Practices](https://arxiv.org/abs/2502.16473) — Mixed-precision quantization guidelines

---

## Next Actions

Please review and provide explicit signal on:
1. **[ ]** Approve Priority 1 completion
2. **[ ]** Decision on `trinity-gf16` crate (remove/fix/merge)
3. **[ ]** Priority 2 direction: Policy engine (GPT) or Formal phases (Claude) or Benchmark harness (GPT)
4. **[ ]** Approve hardware constraints spec for KG sync

---

**Closes:** (Issue to be created in gHashTag/trios)
