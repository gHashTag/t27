# Wave Loop 247 IGLA CODER+RACE Report

*Date: June 16, 2026 | Branch: trinity-rust-rings*

---

## Executive Summary

Wave Loop 247 completed with **570/570 PASS**, **5 seals regenerated**, and **+11 tests / +5 invariants** across 5 specs (2 Pool A, 2 Pool B, 1 CODER). Competitive field remains at **231 tracked competitors** (fifteenth zero-entrant wave overall, fourteenth consecutive since the W234 disruption). Scientific literature shows **stable three-front convergence** with no new papers in any front since W244. No immediate competitive threat.

**Structural milestones:** Pool A floor raised: bram_weights 11→12, formal 11→12. **All Pool A specs now ≥12 invariants** (confirmed after W246 claim). Pool B floor raised: ternary_gemm 12→13, ternary_mac 12→13. CODER floor raised: prm 6→7. **All CODER specs now ≥7 invariants** (first time).

---

## 1. Weak Points Investigated

### 1.1 BRAM Weights — Write/Read Roundtrip

`specs/igla/race/bram_weights.t27` had **92 tests / 11 invariants**, last edited W244. It loaded rows and flattened addresses but did not test that writing a weight changes the value or that flattening (0,0) returns 0. Added `bram_weight_row_count_equals_depth` invariant + two structural tests (`write_weight_changes_value`, `flatten_addr_first_element_zero`).

### 1.2 Formal — Count Proved Plus Admitted Bounded

`specs/igla/race/formal.t27` had **92 tests / 11 invariants**, last edited W244. It bounded proved and admitted counts individually but not their sum. Added `formal_count_proved_plus_admitted_bounded_by_len` invariant + two structural tests (`strings_equal_same`, `count_proved_single_proved`).

### 1.3 Ternary GEMM — Output First Element Identity

`specs/igla/race/ternary_gemm.t27` had **91 tests / 12 invariants**, last edited W242 (**oldest Pool B spec**). It tested zero weights and OOB access but did not invariantly guarantee that identity-like weights preserve the first activation element. Added `ternary_gemm_2x2_output_first_elem_identity` invariant + two structural tests (`identity_weights_first_elem`, `get_elem_2x2_oob_returns_zero`).

### 1.4 Ternary MAC — Zero Weight Identity

`specs/igla/race/ternary_mac.t27` had **94 tests / 12 invariants**, last edited W243. It bounded MAC output but did not guarantee that a zero-weight code leaves the accumulator unchanged. Added `ternary_mac_zero_weight_identity` invariant + two structural tests (`zero_acc_zero_activation`, `decode_zero_weight`).

### 1.5 PRM — Compute Step Reward Bounded

`specs/igla/coder/prm.t27` had **33 tests / 6 invariants**, last edited W236 (**oldest CODER spec, 11 waves untouched**). It bounded reward nonnegativity but not the upper bound. Added `prm_compute_step_reward_bounded` invariant + three structural tests (`reward_syntax_perfect_match`, `compute_step_reward_empty_golden_zero`, `reward_lint_no_mul_high_score`).

---

## 2. Scientific Literature Survey

### 2.1 Ternary Hardware Acceleration (arXiv 2026)

| Paper | Venue | Key Claim | Relevance |
|-------|-------|-----------|-----------|
| [VitaLLM v2](https://arxiv.org/html/2604.27396) | arXiv Apr 2026 | 16nm, 72 tok/s, 0.214 mm², 59–66 mW; dual-core TINT+BoothFlex. | **HIGH** — Most mature edge ternary ASIC. |
| [Geens et al., LUT DSE](https://arxiv.org/html/2604.25183) | arXiv Apr 2026 | Open-source Chisel generator; 2.2× area reduction; TSMC 16nm. | **HIGH** — Commoditizes ternary RTL generation. |
| [TOM](https://arxiv.org/pdf/2602.20662) | arXiv Feb 2026 | ROM-SRAM hybrid; 15.0 MB/mm²; 3,306 TPS at 5.33 W. | **MEDIUM-HIGH** — Edge memory-density threat. |
| [FairyFuse](https://arxiv.org/html/2604.20913) | arXiv Apr 2026 | AVX-512 CPU ternary kernels; zero multiplications; 32.4 tok/s. | **MEDIUM** — CPU software threat. |
| [T-SAR](https://arxiv.org/pdf/2511.13676) | arXiv Nov 2025 | CPU-only ternary via SIMD LUT; 5.6–24.5× GEMM reduction. | **MEDIUM** — ISA-extension approach. |

**No new ternary hardware papers since W244.** Field stable.

### 2.2 Formal Verification & Hardware Synthesis (arXiv 2026)

| Paper | Venue | Key Claim | Relevance |
|-------|-------|-----------|-----------|
| [CktFormalizer](https://arxiv.org/html/2605.07782v3) | arXiv May 2026 (v3) | Dependently-typed HDL in Lean 4; 95–100% backend realizability. | **HIGH** — Lean narrows Coq gap. v3 update noted. |
| [Sparkle HDL](https://github.com/Verilean/sparkle) | GitHub Jun 2026 | Lean 4 standalone HDL; 102 formal theorems, verified RISC-V SoC, BitNet accelerator. | **HIGH** — Production-grade verified IP. Still rising. |
| [FormalRTL](https://arxiv.org/html/2603.08738v1) | arXiv Mar 2026 | hw-cbmc verified RTL; C-reference specs. | **MEDIUM** — Stable. |
| [Arch HDL](https://arxiv.org/pdf/2604.05983) | arXiv Apr 2026 | AI-native HDL with SMT backend; `arch formal` command. | **MEDIUM** — SMT-based BMC. |
| [AutoINV](https://scirate.com/arxiv/2604.22285) | arXiv Apr 2026 | HLS invariant automation; 6.05× speedup. | **MEDIUM** — Adjacent to spec-first approach. |
| [HierSVA](https://arxiv.org/pdf/2606.13706) | arXiv Jun 2026 | Hierarchical SVA generation; assume-guarantee composition. | **MEDIUM** — Hierarchical formal verification. |
| [Interpretable HW Gen](https://arxiv.org/pdf/2606.19387v1) | arXiv Jun 2026 | Stepwise refinement with transformation rules. | **MEDIUM** — Refinement-calculus verified RTL. |

**Key observation:** CktFormalizer updated to v3 (May 2026), confirming active development. Sparkle HDL remains the most significant emerging threat. No new July 2026 papers.

### 2.3 E₈ / H₄ / 600-Cell Spectral Unification (2026)

| Source | Platform | Key Claim | Relevance |
|--------|----------|-----------|-----------|
| [Gray et al., arXiv 2604.00255](https://arxiv.org/abs/2604.00255v1) | arXiv Apr 2026 | 600-cell ↔ E₆/E₇/E₈ exact correspondence via H₃⊂H₄. | **MEDIUM** — Rigorous math. No follow-up. |
| [Martinetti, arXiv 2603.03216](https://arxiv.org/abs/2603.03216v1) | arXiv Mar 2026 | Twisted SM spectral triple; Krein structure; twistor symmetry. | **MEDIUM** — Peer-reviewed NCG. No follow-up. |
| [Morató de Dalmases, SGUP-600cell v5](https://zenodo.org/records/19927449) | Zenodo Apr 2026 | 600-cell spectral triple; RH claim; Millennium Problems. | **MEDIUM-HIGH** — Highest-altitude independent threat. No update. |
| [Dąbrowski et al., arXiv 2511.08159v3](https://arxiv.org/html/2511.08159v3) | arXiv Nov 2025 (v3) | Spectral torsion of internal SM NCG. | **LOW** — Technical; no 600-cell link. |

**All spectral-unification sources stable. No new arXiv submissions.**

---

## 3. Engineering Plan & Implementation

### 3.1 Spec Selection Rationale

| Spec | Pool | Previous Touch | Tests Before | Tests After | Inv Before | Inv After | Rationale |
|------|------|----------------|--------------|-------------|------------|-----------|-----------|
| `bram_weights.t27` | Pool A | W244 | 92 | 94 | 11 | 12 | Joint-lowest Pool A at 11 invariants; write/read roundtrip missing. |
| `formal.t27` | Pool A | W244 | 92 | 94 | 11 | 12 | Joint-lowest Pool A at 11 invariants; proved+admitted sum unbounded. |
| `ternary_gemm.t27` | Pool B | W242 | 91 | 93 | 12 | 13 | **Oldest Pool B** at 12 invariants; first-element identity missing. |
| `ternary_mac.t27` | Pool B | W243 | 94 | 96 | 12 | 13 | Second-oldest Pool B at 12 invariants; zero-weight identity missing. |
| `prm.t27` | CODER | W236 | 33 | 36 | 6 | 7 | **Oldest CODER** at 6 invariants, 11 waves untouched; reward upper bound missing. |

### 3.2 Tests Added

**bram_weights.t27**
1. `bram_weights_write_weight_changes_value` — Write then read back shows new value.
2. `bram_weights_flatten_addr_first_element_zero` — Flatten at (0,0) returns 0.

**formal.t27**
1. `formal_strings_equal_same` — Equal strings returns true.
2. `formal_count_proved_single_proved` — Single proved obligation returns count=1.

**ternary_gemm.t27**
1. `ternary_gemm_2x2_identity_weights_first_elem` — Identity weights preserve first element.
2. `get_elem_2x2_oob_returns_zero` — OOB access returns 0.

**ternary_mac.t27**
1. `ternary_mac_zero_acc_zero_activation` — Zero acc + zero activation = 0.
2. `ternary_decode_zero_weight` — Decode of code 0 returns 0.

**prm.t27**
1. `prm_reward_syntax_perfect_match` — Perfect syntax match gives positive score.
2. `prm_compute_step_reward_empty_golden_zero` — Empty golden string yields score 0.
3. `prm_reward_lint_no_mul_high_score` — Lint of adder (no multiply) gives positive score.

### 3.3 Invariants Added

1. `bram_weight_row_count_equals_depth` — Row count equals bank depth.
2. `formal_count_proved_plus_admitted_bounded_by_len` — Proved + admitted ≤ len(pos).
3. `ternary_gemm_2x2_output_first_elem_identity` — Identity weights preserve a[0].
4. `ternary_mac_zero_weight_identity` — Zero-weight code leaves accumulator unchanged.
5. `prm_compute_step_reward_bounded` — Step reward ≤ 1.0.

---

## 4. Verification Results

| Phase | Result |
|-------|--------|
| Parse | 570 passed, 0 failed |
| Typecheck | 570 passed, 0 failed |
| GF16 Conformance | OK |
| Gen Zig | 570 passed, 0 failed |
| Gen Rust | 570 passed, 0 failed |
| Gen Verilog | 570 passed, 0 failed |
| Gen C | 570 passed, 0 failed |
| Seal Verify | 570 passed, 0 failed |
| Fixed Point | 0 divergences |

**TOTAL: 570/570 PASS**

---

## 5. Competitive Positioning Update

- **New competitors:** 0 (stable plateau at 231 — **fifteenth zero-entrant wave** overall, fourteenth consecutive since W234 disruption).
- **Total tracked:** 231 (unchanged).
- **Emerging threat:** [Sparkle HDL](https://github.com/Verilean/sparkle) (GitHub Jun 2026) — Lean 4 standalone HDL with 102 formal theorems, RISC-V SoC, BitNet accelerator. Threat level: **MEDIUM-HIGH (stable)**. No new commits or releases since W246 discovery.
- **CktFormalizer v3:** arXiv 2605.07782v3 (May 2026) confirms active development of Lean 4 dependently-typed HDL.
- **Three-front convergence stable:**
  1. **Ternary silicon:** VitaLLM v2 (May 2026), Geens LUT-generator, TOM, T-SAR. No new hardware entrants.
  2. **Formal-verification arms race:** Sparkle HDL (Jun 2026) and CktFormalizer v3 (May 2026) are the notable entrants. Veri-Sure, EquivFusion, AutoINV, HierSVA, Interpretable HW Gen stable. Cluster deepening but no equilibrium break.
  3. **E₈/H₄ spectral unification:** Morató SGUP-600cell v5 (Zenodo Apr 2026) stable. Gray (arXiv Apr 2026) stable. Martinetti (arXiv Mar 2026) stable. No new submissions.
- **ASIC timeline:** manhvu/Balanced_Ternary ~24 weeks remaining. No updates.
- **Dormancy alerts:** t81dev/ternary-fabric 4 months dormant. TheusHen/ternary-ibex 9 months dormant.
- **Tier movements:** None.

---

## 6. Key Observations

1. **IGLA RACE Variant A active:** +11 tests (2 Pool A: bram_weights, formal; 2 Pool B: ternary_gemm, ternary_mac) + CODER depth push (prm, +3 tests, +1 invariant). 570/570 PASS with 5 seal regenerations.
2. **Pool A floor raised:** bram_weights 11→12, formal 11→12. **All Pool A specs now ≥12 invariants** (confirmed; after W246, cordic_top and gemm were raised; now bram_weights and formal complete the set).
3. **Pool B floor raised:** ternary_gemm 12→13, ternary_mac 12→13. Pool B now spans 12–13.
4. **CODER floor raised:** prm 6→7. **All CODER specs now ≥7 invariants** (first time). bench_proxy, tokenizer, training, arch at 7; prm now at 7.
5. **Fifteen-wave competitive calm:** W233 (0), W234 (+2), W235–W247 (0 each). Absolute record extended.
6. **Sparkle HDL stable:** No new activity since W246 discovery. Threat remains MEDIUM-HIGH but not escalating.
7. **No new scientific urgency:** No new arXiv papers in any front since W244. CktFormalizer v3 is the only new development.
8. **Engineering health:** Suite passes consistently at 570/570. Structural floors verified: Pool A ≥12, Pool B ≥12, CODER ≥7.
9. **CODER milestone:** All CODER specs ≥7 is a new structural ceiling. Next soft target: Raise all CODER specs to ≥8 by W255.
10. **prm age:** prm.t27 last edited W236 — **11 waves untouched**. Oldest CODER spec brought forward, demonstrating rotation heuristic health.

---

*Report generated by Trinity Queen Agent | Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>*
