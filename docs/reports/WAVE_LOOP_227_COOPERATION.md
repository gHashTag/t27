# Wave Loop 227 — Cooperation Proposals for W228

*Date: 2026-06-19*
*Context: Wave Loop 227 completed (570/570 PASS, +11 tests, +5 invariants). Competitive velocity accelerating: 2 new entrants (Max042004/bitmamba.c ternary SSM FPGA offload, deveworld/bitnet-tt Tenstorrent custom silicon). Total tracked competitors: 227.*
*φ² + 1/φ² = 3 | TRINITY*

---

## Executive Summary

Wave Loop 227 delivered **horizontal coverage lift** across four RACE specs (cordic, cordic_fixed, systolic_array, systolic_ternary) and one **critical CODER depth push** on bench_proxy (the shallowest spec in the module at 24 tests). More significantly, competitive surveillance discovered **2 new entrants** in one wave — the highest velocity since the competitive tracking program began. The field is expanding beyond FPGA into custom silicon (Tenstorrent Blackhole) and state-space models (BitMamba-2), signaling a phase transition in ternary hardware acceleration.

With competition now attacking on three fronts simultaneously (FPGA, custom silicon, SSM architectures), **cooperation becomes strategically essential** to maintain Trinity's differentiation through formal proofs and geometric foundations.

---

## Variant A: Ternary Hardware Benchmark Consortium *(Recommended)*

### Mechanics
Form a **4-lab benchmark consortium** spanning the three hardware paradigms now present in the competitive landscape:
- **Trinity** — t27c compiler + formal proof framework + H₄-weight generation
- **Neumann-Labs** — Arty A7-35T FPGA (sub-watt, sparsity-skipping)
- **deveworld** — Tenstorrent Blackhole p150a (custom silicon, 73.4 tok/s)
- **shepherdscientific** — Xilinx Artix-7 (CERN-OHL-S, foundational RTL)

Each lab runs a shared `trinity-bench-ternary-v2` harness on their own stack. The harness includes:
1. **Sacred compliance gate** (R-SI-1: zero '*' operators, H₄-weight geometry)
2. **Throughput metric** (tok/s at batch=1)
3. **Energy metric** (J/token)
4. **Correctness oracle** (formal equivalence check via Yosys for RTL submissions)

Results are published quarterly as a **Joint Ternary Hardware Report** under a neutral brand.

### Value Proposition
- **Immediate multi-platform credibility:** Trinity gains validated benchmarks on FPGA + custom silicon without owning either hardware platform.
- **Defensive moat:** A published consortium report positions Trinity as the "honest broker" of ternary hardware evaluation. Competitors entering later must reference our methodology.
- **Low friction:** No IP transfer. Each lab retains its own stack. Only the benchmark protocol and report are shared.

### Preconditions
- Shared benchmark harness derived from `eval.t27` + `bench_proxy.t27` (now at 27/6, hardened this wave).
- Data sharing under CC-BY-4.0; toolchains under existing licenses.
- Trinity retains veto on any claim involving formal proofs, H₄ geometry, or spectral action.

### Who This Fits
**Neumann-Labs** (FPGA energy metrics), **deveworld** (custom silicon throughput), **shepherdscientific** (Xilinx toolchain + CERN-OHL-S governance).

---

## Variant B: BitMamba-SSM Bridge + Formal Verification Partnership

### Mechanics
A **targeted bilateral collaboration** with Max042004/bitmamba.c to bridge ternary state-space models into Trinity's formal verification framework:
1. **Trinity contributes:** A formal proof of correctness for the ternary MAC unit used in BitMamba's DE10-Nano FPGA offload path. Proof shows that ternary weight multiplication ({-1, 0, +1} × activation) preserves the spectral bounds required for SSM stability.
2. **bitmamba.c contributes:** Real hardware measurements (tok/s, power) from the DE10-Nano FPGA offload path, plus the actual Verilog netlist for the ternary MAC.
3. **Joint deliverable:** A co-authored technical note showing that BitMamba-2's inference pipeline satisfies R-SI-1 (zero multiplication operators in RTL) when compiled through t27c.

### Value Proposition
- **First formal-verified SSM:** No other ternary SSM project has machine-checked correctness proofs. This creates an immediate differentiation for both parties.
- **SSM market entry:** State-space models are gaining traction as efficient alternatives to transformers. Establishing Trinity's presence here early locks in the "formally verified ternary SSM" narrative.
- **Low risk:** BitMamba is a hobbyist/open-source project (C + Verilog). No corporate IP politics.

### Preconditions
- bitmamba.c DE10-Nano Verilog netlist shared under MIT license.
- Trinity formal proof scripts shared under Apache-2.0 upon completion.
- Joint publication under both authors' names (no sole-lead requirement).

### Who This Fits
**Max042004/bitmamba.c** (SSM + FPGA offload), **vfd-org** (geometric correctness oracle), **FormalRTL** (equivalence checking infrastructure).

---

## Variant C: Unified Ternary arXiv Preprint + Shared Reproducibility Package

### Mechanics
A **coordinated multi-party preprint** titled "Ternary Neural Network Inference: A Multi-Platform Study" with the following structure:
1. **Theory track (Trinity lead):** H₄/600-cell mass derivation, uniqueness constraints, spectral action bounds. Shows why ternary weights are geometrically optimal.
2. **FPGA track (Neumann-Labs + shepherdscientific):** Artix-7 (Arty A7-35T and A7-100T) implementation, energy measurements, floorplans, PPA.
3. **Custom silicon track (deveworld):** Tenstorrent Blackhole p150a implementation, throughput scaling, BFP2 packing efficiency.
4. **SSM track (Max042004):** BitMamba-2 ternary state-space model, DE10-Nano offload, stability proofs.
5. **Verification track (shared):** Yosys equivalence checks between high-level models and synthesized netlists; Coq proof of ternary MAC correctness.

Authorship order: determined by track contribution fractions. Trinity retains sole authorship on the theory track.

### Value Proposition
- **Highest academic impact:** A 5-track, multi-platform, formally verified preprint would be unprecedented in the ternary NN literature.
- **Reproducibility moat:** The shared reproducibility package (source code, benchmark harness, Coq scripts) becomes the de facto standard. Future papers citing our work reinforce our authority.
- **Cross-platform validation:** No single lab can cover FPGA + custom silicon + SSM + theory + verification alone. Collaboration is the only path to this scope.

### Preconditions
- All parties agree on sacred constants (φ² + 1/φ² = 3, H₄ embedding, 600-cell).
- Hardware data shared under embargo until preprint submission.
- Coq scripts released under Apache-2.0 upon acceptance.
- Benchmark harness maintained by Trinity under t27/tri pipeline (L7 compliance).

### Who This Fits
**Neumann-Labs** (FPGA track), **deveworld** (custom silicon), **Max042004** (SSM), **shepherdscientific** (Artix-7 validation + CERN-OHL-S governance).

---

## Comparative Matrix

| Dimension | Variant A (HW Consortium) | Variant B (BitMamba Bridge) | Variant C (Unified Preprint) |
|-----------|---------------------------|----------------------------|------------------------------|
| **Time to execute** | 3–5 weeks | 4–6 weeks | 2–4 months |
| **IP exposure** | Low | Medium | Medium-High |
| **Revenue potential** | None (grants/indirect) | None (technical note) | Indirect (citations, grants↑) |
| **Strategic defensibility** | Very High (benchmark lock-in) | High (first formal SSM) | Very High (proof monopoly) |
| **Partner enthusiasm** | High (low friction) | Medium (hobbyist project) | Medium (authorship politics) |
| **Trinity control** | High (formal proof veto) | High (proof authorship) | Medium (track ownership) |
| **Novelty factor** | Medium (multi-platform) | Very High (first formal SSM) | Very High (5-track unified) |

---

## Recommendation

**Lead with Variant A in W228, activate Variant B in parallel, prepare Variant C for W230.**

1. **Week 1 (W228):** Reach out to Neumann-Labs, deveworld, and shepherdscientific with a low-friction proposal: "Run our `trinity-bench-ternary-v2` harness on your hardware and share results; we jointly publish a quarterly report." This costs nothing and gauges openness.
2. **Week 2–3:** If Neumann-Labs responds positively, expand to deveworld for Tenstorrent data and shepherdscientific for Artix-7 coverage. Publish the first Joint Ternary Hardware Report as a living document on GitHub.
3. **Month 2:** If the consortium stabilizes, reach out to Max042004 for Variant B. Offer a formal proof of their ternary MAC in exchange for DE10-Nano measurements and netlist access. Publish a 2-page technical note.
4. **Month 3+:** If both Variant A and B succeed, escalate to Variant C for a unified multi-track arXiv preprint. This maximizes academic impact while distributing workload across partners.

**Immediate action items:**
1. Draft `docs/cooperation/TERNARY_HW_CONSORTIUM_W228.md` — lightweight protocol and benchmark checklist.
2. Open issues on `Neumann-Labs/ternfpga` and `deveworld/bitnet-tt` inviting collaboration on joint benchmarking.
3. Schedule internal Trinity review of `bench_proxy.t27` to ensure it can serve as the consortium's evaluation baseline.
4. Begin drafting the theory track outline for Variant C (can be written independently while partners join).

---

*Prepared by Trinity Agent (Queen) | Wave Loop 227*
*φ² + 1/φ² = 3 | Honest science is slow science | Verification pending*
