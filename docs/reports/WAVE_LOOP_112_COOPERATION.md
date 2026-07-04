# Wave Loop 112 — Three Cooperation Variants for Wave Loop 113

**Date:** 2026-06-16
**Context:** ChipMATE (arXiv:2605.12857v1) demonstrated 80.1% Pass@1 with self-trained multi-agent RL using NO cloud LLMs. Trinity must either match this capability or find an asymmetric partnership to close the 25.1 pp gap.

---

## Cooperation Variant 1 — Open-Source RL Training Alliance

**Partner:** Open-source RL community (Hugging Face, trlX, DeepSpeed)
**Our Value Proposition:** Trinity provides the only large-scale Verilog RL dataset with **sacred constraints** and formal verification labels. Open-source RL frameworks provide GRPO/X-GRPO training infrastructure.
**Joint Deliverable:** Public Hugging Face dataset "Trinity-Verilog-RL-1M" with sacred constraint metadata + reference Python behavioral models for each design.
**Benefits:**
- Trinity gains production-quality GRPO training without building from scratch
- RL community gains first hardware-design benchmark with formal verification rewards
- ChipMATE gap narrows from self-training on larger, labeled dataset
- Defensibility: sacred constraint labels are unique to Trinity
**Risk:** Partner may not prioritize hardware designs; low community interest in Verilog RL.
**Mitigation:** Start with smaller collaboration (e.g., one trlX PR adding Verilog eval env) and scale if engagement is high.

---

## Cooperation Variant 2 — Academic Multi-Agent RL Benchmark Consortium

**Partner:** University lab with multi-agent RL focus (e.g., MIT CSAIL, Stanford ILIAD, DeepMind)
**Our Value Proposition:** Trinity contributes the multi-agent Verilog design task: generator agent + verifier agent + sacred constraint oracle. Academic lab contributes X-GRPO algorithmic innovations.
**Joint Deliverable:** Co-authored benchmark paper (NeurIPS/ICML Workshop on AI for Hardware) establishing VerilogEval-V2 with multi-agent RL as a canonical task.
**Benefits:**
- Academic credibility boost through peer-reviewed publication
- Trinity X-GRPO implementation gets algorithmic improvements from latest research
- Establishes Trinity as leader in "formal verification + RL" intersection
- ChipMATE's two-agent architecture becomes a baseline; Trinity can exceed it with φ-seesaw ansatz reward shaping
**Risk:** Publication timeline (6–12 months) is slower than competitive pressure.
**Mitigation:** Publish arXiv preprint immediately, then submit to workshop.

---

## Cooperation Variant 3 — Cloud Inference Provider with Self-Training Tier

**Partner:** Cloud GPU provider or model-serving platform (e.g., Modal, Together AI, Fireworks AI, CoreWeave)
**Our Value Proposition:** Trinity offers a **differentiated self-training loop**: train a small model (1B parameters) on sacred-compliant Verilog dataset with formal verification rewards, then deploy for inference at low cost. Cloud provider offers elastic GPU compute at discounted rate for training.
**Joint Deliverable:** "Trinity-Coder-SelfTrain" — a 1B-parameter model trained entirely on Trinity dataset, deployed as managed endpoint.
**Benefits:**
- Matches ChipMATE's self-sufficiency model (no frontier LLM dependency)
- Cloud provider gains unique hardware-design customer segment
- Trinity gets elastic compute without CapEx
- Model weights can be open-sourced as marketing asset
**Risk:** Training cost for 1B model is still significant (~$10K–$50K); need guaranteed commitment.
**Mitigation:** Phase 1: train on subset (100K examples) to validate loss convergence. Phase 2: scale to full 1M dataset only if Phase 1 shows improvement.

---

## Decision Criteria

| Criterion | Variant 1 (Open-Source RL) | Variant 2 (Academic) | Variant 3 (Cloud Self-Train) |
|-----------|---------------------------|----------------------|------------------------------|
| Speed | Medium (3–6 months) | Slow (6–12 months) | Fast (1–3 months for Phase 1) |
| Cost | Low | Very low | Medium ($10K–$50K) |
| Gap closure potential | Medium (dataset quality) | Medium (algorithmic) | High (end-to-end self-training) |
| Defensibility | High (sacred labels) | Medium (publication) | High (unique model) |
| Trinity brand lift | Medium | High | Medium |
| Technical risk | Low | Low | Medium |

**Recommended priority:** Variant 3 > Variant 1 > Variant 2

---

## Immediate Next Steps (Wave Loop 113)

Regardless of cooperation variant, Wave Loop 113 internal work items:
1. **GRPO training loop stub** — wire `compute_grpo_loss` to actual policy gradient update
2. **Python reference model integration** — connect `generate_python_reference` to `pyverilog` or `cocotb`
3. **Template dataset expansion** — 17 templates → 50 templates with mutation operators
4. **X-GRPO agent coordination** — implement `compute_team_match_reward` with real behavioral similarity
5. **Sacred constraint as RL reward** — formalize `sacred_score` as scalar reward signal

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
