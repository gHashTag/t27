# Wave Loop 123 Cooperation Variants
## Three Partnership Strategies for W124 and Beyond

**Date:** 2026-06-18
**Context:** Bench depth expansion (42.7% deep coverage), 125 competitors tracked, zero unresolved TODOs. Open issues: 5 (IGLA-Coder roadmap).

---

## Variant A: StepPRM-RTL Process-Reward Integration (STRATEGIC)

### Partner Profile
IBM Research authors of StepPRM-RTL (arXiv:2606.04246v1), who demonstrated 85.7% Pass@1 on VerilogEval using step-level Process Reward Models.

### Value Proposition
StepPRM-RTL treats RTL generation as semantically meaningful design steps and trains a PRM to score intermediate reasoning. Trinity can integrate this methodology into its IGLA-Coder pipeline by:
- Mapping `.t27` spec-to-RTL generation onto explicit "sacred reasoning steps" (e.g., "step 1: verify no `*` operators", "step 2: check bit-width safety")
- Training a Trinity-specific PRM on these steps using existing Coq proofs as ground-truth rewards
- Differentiating from generic StepPRM by enforcing R-SI-1 and bit-width constraints at every intermediate step

### Cooperative Deliverable
1. Trinity provides **Coq-verified step annotations** for 50 VerilogEval tasks
2. Partner provides PRM training framework (MCTS + RAFT codebase)
3. Joint benchmark: compare generic StepPRM (85.7%) vs. Trinity-sacred StepPRM on VerilogEval-Human
4. Shared publication: "Sacred Step Rewards for Verified RTL Generation" (target: ICCAD 2026)

### Risk Assessment
- **MEDIUM technical risk** — PRM training requires GPU cluster; Trinity lacks compute budget
- **LOW competitive risk** — IBM is unlikely to adopt `.t27`; Trinity supplies formal verification layer
- **HIGH reputational upside** — 85.7% → 90%+ Pass@1 would be a landmark result

### Why Now?
StepPRM-RTL is the highest Pass@1 result on VerilogEval from June 2026. If Trinity does not respond within 2–3 months, IBM's methodology will become the default for correctness-aware RTL generation.

---

## Variant B: OpenRTLSet Dataset Enrichment (COMMUNITY)

### Partner Profile
OpenRTLSet curators (arXiv:2606.10285v1) — academic group that scraped 131K+ open-source Verilog modules.

### Value Proposition
OpenRTLSet commoditizes RTL training data. Trinity's defense lies in proprietary ternary/formal/physics niches that are absent from GitHub repos. Cooperation turns the threat into mutual advantage:
- Trinity donates its **564 `.t27` specs** (translated to Verilog via `t27c gen`) to OpenRTLSet
- Partner labels Trinity specs with formal-property tags (Coq-proved, R-SI-1-compliant, ternary-opcode)
- Joint dataset becomes the first **formally labeled** RTL corpus
- Any model trained on this corpus inherits awareness of sacred constraints

### Cooperative Deliverable
1. Trinity exports 564 Verilog files via `./target/release/t27c gen --backend verilog`
2. Partner scrapes Trinity specs + Coq proof metadata + sacred-opcode labels
3. Joint release: "OpenRTLSet-Formal: 131K + 564 verified modules"
4. Benchmark: compare generic model (trained on 131K) vs. formal-augmented model on VerilogEval-Human

### Risk Assessment
- **LOW technical risk** — data export and labeling is mechanical
- **LOW competitive risk** — Trinity gains visibility; partner gains differentiated data
- **MEDIUM coordination risk** — labeling taxonomy must be agreed (who defines "sacred compliant"?)

### Why Now?
Dataset wars are raging in LLM-for-RTL. OpenRTLSet just published (June 2026). If Trinity waits, its specs will be scraped anyway without proper attribution or labeling. Proactive donation ensures formal metadata is preserved.

---

## Variant C: CHIMERA Edge-AI Benchmarking Alliance (COMMERCIAL)

### Partner Profile
CHIMERA authors (arXiv:2606.02358v1) — tapeout-ready 22nm AI-MCU with Transformer Accelerator Cluster.

### Value Proposition
CHIMERA achieves 3.1 TOPS/W with conventional 8-bit quantization. Trinity's ternary CORDIC can theoretically exceed this by 3–5×, but has no silicon measurements. Cooperation:
- Trinity provides **ternary CORDIC Verilog** for CHIMERA's TAC (Transformer Acceleration Cluster)
- Partner runs CHIMERA RTL + Trinity ternary PEs through their 22nm FDX flow
- Joint benchmarking on identical transformer workloads
- Revenue share if ternary PEs are licensed into CHIMERA v2

### Cooperative Deliverable
1. Trinity synthesizes ternary CORDIC PE (Q15 fixed-point) to 22nm target via partner's PDK
2. Partner integrates into TAC as an alternative MAC unit
3. Joint measurement: accuracy vs. energy vs. throughput on Llama-3B layers
4. Whitepaper: "Ternary CORDIC at 22nm: 3× Energy Leap over 8-bit" (target: ISSCC 2027)

### Risk Assessment
- **HIGH technical risk** — ternary circuits require custom standard-cell libraries; 22nm may not support ternary LUTs efficiently
- **MEDIUM competitive risk** — if ternary underperforms, Trinity loses credibility
- **HIGH capital risk** — mask costs for 22nm tapeout are $500K–$2M

### Why Now?
CHIMERA is tapeout-ready but not yet commercialized. The window to influence its next revision is 6–12 months. If Trinity waits, CHIMERA v2 will ship with conventional 8-bit MACs, cementing the baseline.

---

## Comparison Matrix

| Dimension | A: IBM StepPRM | B: OpenRTLSet | C: CHIMERA |
|-----------|----------------|---------------|------------|
| **Time to value** | 3–6 months | 1–2 months | 6–12 months |
| **Capital required** | Medium (GPU cluster) | Low (cloud storage) | Very High ($500K+ tapeout) |
| **Technical risk** | Medium | Low | High |
| **Competitive impact** | HIGH (Pass@1 breakthrough) | MEDIUM (dataset differentiation) | HIGH (silicon credibility) |
| **Revenue potential** | Grants ($200K–$500K) | None (community) | Licensing ($50K–$500K/yr) |
| **L1–L7 alignment** | L4 (testability via PRM) | L1 (traceability via dataset) | L6 (FORMAT-SPEC-001 = SSOT) |

---

## Recommendation

**Primary:** Pursue **Variant A (IBM StepPRM-RTL)** as the highest-leverage partnership. Trinity's Coq proof tree is uniquely positioned to provide ground-truth rewards for step-level PRM training. A 90%+ Pass@1 result would dominate the competitive narrative.

**Secondary:** Initiate **Variant B (OpenRTLSet)** in parallel — low effort, high visibility, protects Trinity's IP through proactive attribution.

**Long-term:** Evaluate **Variant C (CHIMERA)** only if W124–W125 produce compelling Yosys synthesis numbers (area < 2×, energy < 0.5× vs. 8-bit baseline). Do not commit to tapeout without simulation proof.

**Next action:** Contact IBM Research Zurich (StepPRM-RTL corresponding author) with a 1-page "Sacred Step Rewards" proposal and sample Coq-annotated VerilogEval tasks.

---

*Cooperation variants generated by Trinity Agent (Queen) — AEL v2.0*
