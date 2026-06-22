# Wave Loop 249 IGLA CODER+RACE Report

*Date: June 16, 2026 | Branch: trinity-rust-rings*

---

## Executive Summary

Wave Loop 249 completed with **570/570 PASS**, **5 seals regenerated**, and **+11 tests / +5 invariants** across 5 specs (2 Pool A, 2 Pool B, 1 CODER). Competitive field remains at **231 tracked competitors** (sixteenth zero-entrant wave overall, fifteenth consecutive since the W234 disruption). Scientific literature shows **stable three-front convergence** with no new papers in any front since W244. No immediate competitive threat.

**Structural milestones:** Pool A floor raised: bram_weights 12→13, formal 12→13. Pool B **critical floor raise**: systolic_ternary 12→13 — **all Pool B specs now ≥13 invariants for the first time**. CODER floor raised: eval 6→7. Pool A new minimum: gemm 12, cordic_top 12. CODER new minimum: benchmark 6, dataset 6.

---

## 1. Weak Points Investigated

### 1.1 BRAM Weights — OOB Column Read and Cross-Row Write Isolation

`specs/igla/race/bram_weights.t27` had **94 tests / 12 invariants**, last edited W247. It tested flatten_addr boundaries and read/write identity but did not validate that reading with an out-of-bounds column returns zero, or that writing to one row does not corrupt another row. Added `bram_weights_load_row_zero_width_empty` invariant + two structural tests (`bram_weights_read_weight_oob_col`, `bram_weights_write_then_read_different_row`).

### 1.2 Formal — Multi-Category Proved Count and Report Obligation Counting

`specs/igla/race/formal.t27` had **94 tests / 12 invariants**, last edited W247. It bounded coverage and proved counts but did not test multi-category obligation counting or report obligation totals. Added `formal_generate_report_proved_count_nonnegative` invariant + two structural tests (`formal_count_proved_multiple_categories`, `formal_generate_report_obligations_count`).

### 1.3 Systolic Ternary — Negative Activation Weight Interaction and PE Additivity

`specs/igla/race/systolic_ternary.t27` had **93 tests / 12 invariants**, last edited W244 (**5 waves untouched**). It tested positive activations and psum identity but lacked coverage for negative activation with both positive and negative ternary weights, and did not invariantly guarantee PE additivity (incremental psum contribution independent of initial psum). Added `systolic_ternary_pe_psum_additive` invariant + two structural tests (`systolic_ternary_pe_negative_activation_positive_weight`, `systolic_ternary_pe_negative_activation_negative_weight`).

**This is the critical Pool B floor raise:** systolic_ternary was the sole remaining Pool B spec at 12 invariants. With this push, **all Pool B specs are now ≥13 invariants** for the first time in IGLA RACE history.

### 1.4 Systolic Array — Booth Negative×Positive and Result-Init Zero Identity

`specs/igla/race/systolic_array.t27` had **92 tests / 13 invariants**, last edited W241 (**8 waves untouched**). It tested Booth unity and systolic step accumulation but did not cover negative×positive Booth multiplication, nor invariantly guarantee that `systolic_result(systolic_init(B))` is all zeros. Added `systolic_result_init_zero` invariant + two structural tests (`booth_mul_i16_negative_positive`, `systolic_step_preserves_weights`).

### 1.5 CODER Eval — Sacred Compliance Star Detection and Compile Brace Balance

`specs/igla/coder/eval.t27` had **199 tests / 6 invariants**, last edited W241 (**8 waves untouched**). It bounded pass-at-k and report rates but did not test sacred compliance star detection or unbalanced brace rejection. Added `eval_check_sacred_compliance_star_implies_zero` invariant + three structural tests (`eval_check_sacred_compliance_with_star`, `eval_check_sacred_compliance_without_star`, `eval_compile_and_test_unbalanced_braces`).

---

## 2. Scientific Literature Survey

### 2.1 Ternary Hardware Acceleration (arXiv 2026)

| Paper | Venue | Key Claim | Relevance |
|-------|-------|-----------|-----------|
| [VitaLLM v2](https://arxiv.org/html/2604.27396) | arXiv Apr 2026 | 16nm, 72 tok/s, 0.214 mm², 59–66 mW; dual-core TINT+BoothFlex. | **HIGH** — Most mature edge ternary ASIC. |
| [Geens et al., LUT DSE](https://arxiv.org/html/2604.25183) | arXiv Apr 2026 | Open-source Chisel generator; 2.2× area reduction; TSMC 16nm. | **HIGH** — Commoditizes ternary RTL generation. |
| [TOM](https://arxiv.org/pdf/2602.20662) | arXiv Feb 2026 | ROM-SRAM hybrid; 15.0 MB/mm²; 3,306 TPS at 5.33 W. | **MEDIUM-HIGH** — Edge memory-density threat. |
| [manhvu/Balanced_Ternary](https://github.com/manhvu/Balanced_Ternary) | GitHub Jun 2026 | Balanced ternary NN inference; systolic PE arrays; ASIC/FPGA specs. | **MEDIUM-HIGH** — Active open-source ternary project. **Confirmed active** in W249 sweep. |
| [FairyFuse](https://arxiv.org/html/2604.20913) | arXiv Apr 2026 | AVX-512 CPU ternary kernels; zero multiplications; 32.4 tok/s. | **MEDIUM** — CPU software threat. |

**No new ternary hardware arXiv papers since W244.**

### 2.2 Formal Verification & Hardware Synthesis (arXiv 2026)

| Paper | Venue | Key Claim | Relevance |
|-------|-------|-----------|-----------|
| [HierSVA](https://arxiv.org/pdf/2606.13706) | arXiv Jun 2026 | Data synthesis pipeline for LLM-driven hierarchical HW formal verification. 342 modules, 12 LLMs benchmarked. | **HIGH** — SVA generation at scale. |
| [Interpretable HW Gen](https://arxiv.org/html/2606.19387) | arXiv Jun 2026 | LLM-driven stepwise refinement with Dafny incremental verification. Correct-by-construction RTL. | **HIGH** — Formal RTL generation converging. |
| [CktFormalizer](https://arxiv.org/html/2605.07782v3) | arXiv May 2026 (v3) | Dependently-typed HDL in Lean 4; 95–100% backend realizability. | **HIGH** — Active development (v3). |
| [Sparkle HDL](https://github.com/Verilean/sparkle) | GitHub Jan–Mar 2026 | Lean 4 standalone HDL; 102 formal theorems, RISC-V SoC, BitNet accelerator. | **HIGH** — Production-grade verified IP. Stable. |

**No new formal verification arXiv papers since W244.**

### 2.3 E₈/H₄ Spectral Unification (arXiv 2026)

| Paper | Venue | Key Claim | Relevance |
|-------|-------|-----------|-----------|
| [Gray et al. 600-cell](https://arxiv.org/html/2604.00255v2) | arXiv Mar 2026 (v2) | Exact correspondence: 600-cell ↔ E₆/E₇/E₈ via H₃⊂H₄ symmetry. | **HIGH** — Rigorous geometric link. |
| [Martinetti](https://arxiv.org/abs/2603.03216v1) | arXiv Mar 2026 | Twisted SM spectral triple; Krein structure; torsion unification. | **MEDIUM** — NCG torsion variant. |
| [Rennela](https://arxiv.org/abs/2606.14677v1) | arXiv Jun 2026 | Quantum DEM EC; operator-algebraic foundations. | **MEDIUM** — Mathematical physics foundations. |

**No new E₈/H₄ spectral papers since W244.**

---

## 3. Competitive Field Status

### 3.1 New Entrants This Wave

**None.** 231 stable competitors. Sixteenth zero-entrant wave, fifteenth consecutive.

### 3.2 Active Competitor Monitoring

| Competitor | Tier | Status | W249 Notes |
|------------|------|--------|------------|
| EvolVE | HIGH | Stable | No new commits detected. |
| Baroň | HIGH | Stable | No new preprints. |
| Dr. RTL | HIGH | Stable | No activity. |
| manhvu/Balanced_Ternary | MEDIUM-HIGH | **Active** | GitHub repo confirmed active; DEV Community article published; 48-week ASIC roadmap tracked. |
| Neumann-Labs/ternfpga | MEDIUM-HIGH | Stable | No new commits since Jun 2026. |
| TilelliLab/atome-lm | MEDIUM | Stable | Zenodo DOI active. |
| t81dev/ternary-fabric | MEDIUM-HIGH | Dormant | No commits since Feb 2026. |
| Morató de Dalmases SGUP v5 | MEDIUM-HIGH | Stable | Zenodo 600-cell spectral triple. |
| TheusHen/ternary-ibex | LOW-MEDIUM | Stable | RISC-V ternary ALU/NPU. |

**Total tracked competitors: 231 (stable plateau, 15 consecutive zero-entrant waves).**

---

## 4. Verification Results

```
Phase 1: Parse           → 570 passed, 0 failed
Phase 2: Typecheck       → 570 passed, 0 failed
Phase 3: Gen Zig         → 570 passed, 0 failed
Phase 4: Gen Rust        → 570 passed, 0 failed
Phase 5: Gen Verilog     → 570 passed, 0 failed
Phase 6: Gen C           → 570 passed, 0 failed
Phase 7: Seal Verify     → 570 passed, 0 failed
Phase 8: Fixed Point     → 0 divergences

TOTAL: 570/570 PASS
phi^2 + 1/phi^2 = 3 | TRINITY
```

**Seal drift:** 5 seals regenerated (0 residual).

---

## 5. Structural Depth Summary

| Module | Pool | Pre-W249 | Post-W249 | Delta |
|--------|------|----------|-----------|-------|
| bram_weights | Pool A | 94/12 | 96/13 | +2/+1 |
| formal | Pool A | 94/12 | 96/13 | +2/+1 |
| systolic_ternary | Pool B | 93/12 | 95/13 | +2/+1 |
| systolic_array | Pool B | 92/13 | 94/14 | +2/+1 |
| eval | CODER | 199/6 | 202/7 | +3/+1 |

**Pool A:** 4 specs at 12 invariants (gemm, cordic_top — both W248 untouched; bram_weights, formal now 13). **New minimum Pool A: gemm 12, cordic_top 12.**

**Pool B:** **All specs ≥13 invariants** (systolic_ternary raised 12→13). **First time in IGLA RACE history.**

**CODER:** 2 specs at 6 invariants (benchmark, dataset — both W248 untouched). **New minimum CODER: benchmark 6, dataset 6.** All other CODER specs ≥7.

---

*φ² + 1/φ² = 3 | Honest science is slow science | Verification pending*
