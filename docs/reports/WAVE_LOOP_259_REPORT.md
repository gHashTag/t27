# Wave Loop 259 IGLA CODER+RACE — Variant A Pool A/B Floor Elimination + CODER Depth +11 Tests +5 Invariants + 231 Stable Plateau (26th Zero-Entrant Wave, 25th Consecutive) + Pool A 4→2 specs at 13 + Pool B Uniform ≥14 + CODER arch 7→8 + 5 Seals + Report/Cooperation for W260

**Date:** 2026-06-16  
**Branch:** `trinity-rust-rings`  
**Suite Result:** 570/570 PASS (Parse, Typecheck, GF16, Gen Zig, Gen Rust, Gen Verilog, Gen C, Seal Verify, Fixed Point — all clean)  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Competitive Sweep

| Metric | Value |
|--------|-------|
| Total competitors | **231** (stable) |
| New competitors | **0** |
| Zero-entrant streak | **26 waves** (absolute record) |
| Consecutive zero-entrant | **25 waves** (absolute record) |
| Last new entrant | W248 (manhvu/Balanced_Ternary) |
| manhvu/Balanced_Ternary | Confirmed active (GitHub, 48-week ASIC roadmap, no tape-out). Threat: **MEDIUM-HIGH (stable)** |
| Sparkle HDL | Stable, no new activity since W246. Threat: **MEDIUM-HIGH (stable)** |
| t81dev/ternary-fabric | Dormant (5 months, no commits since Feb 2026) |
| TheusHen/ternary-ibex | Dormant (10 months, no commits since Sep 2025) |
| New arXiv papers | **0 truly new since W258** — Singh 2606.12477, Rocq-to-Metal 2606.02651, Planning to Hammer 2606.17981 already tracked |

### Scientific Convergence (stable)
- **Ternary silicon:** TUM atomic-scale systolic array (balanced ternary + systolic), VitaLLM v2, Geens LUT-generator, TOM, T-SAR, manhvu/Balanced_Ternary — no new June papers.
- **Formal-verification arms race:** Sparkle HDL + CktFormalizer v3, HierSVA, Interpretable HW Gen, S-two AIR, "Planning to Hammer" — stable.
- **E₈/H₄ spectral unification:** Morató SGUP-600cell v5, Gray, Martinetti, Singh (arXiv 2606.12477) — stable.

### New Competitors Discovered (unchanged since W258)
- Neumann-Labs/ternfpga (June 2026, no formal proofs). LOW threat.
- shepherdscientific/ternarycore (April 2026, no formal verification). LOW threat.

---

## 2. Structural Changes (5 specs touched)

### Pool A — Floor Elimination (2 of 4 specs raised)
| Spec | Tests Before | Tests After | Inv Before | Inv After | Last Touched |
|------|-------------|-------------|------------|-----------|--------------|
| bram_weights | 98 | **100** | 13 | **14** | W256 (2 waves untouched) |
| cordic_fixed | 97 | **99** | 13 | **14** | W255 (3 waves untouched) |

- **bram_weights**: +2 tests (`bram_weights_write_then_read_same_addr`, `bram_weights_load_row_first_row`) +1 invariant (`bram_weights_write_then_read_identity`).
- **cordic_fixed**: +2 tests (`cordic_fixed_sin_zero_angle`, `cordic_fixed_cos_zero_angle_one`) +1 invariant (`cordic_fixed_sin_zero_angle_zero`).

**Pool A remaining at 13:** cordic_top, formal (2 specs). Previously 4 specs at 13.

### Pool B — Floor Elimination + Depth Push
| Spec | Tests Before | Tests After | Inv Before | Inv After | Last Touched |
|------|-------------|-------------|------------|-----------|--------------|
| systolic_ternary | 99 | **101** | 13 | **14** | W252 (**6 waves untouched**) |
| ternary_gemm | 95 | **97** | 14 | **15** | W252 (**6 waves untouched**) |

- **systolic_ternary**: +2 tests (`systolic_ternary_pe_zero_activation_identity`, `systolic_ternary_pe_positive_weight_one`) +1 invariant (`systolic_ternary_pe_zero_activation_preserves_psum`).
- **ternary_gemm**: +2 tests (`ternary_gemm_2x2_identity_weights`, `get_elem_2x2_first_row_second_col`) +1 invariant (`ternary_gemm_2x2_identity_weights_inv`).

**ALL Pool B specs now ≥14 invariants** — systolic_ternary was the final spec at 13.

### CODER — Depth Push
| Spec | Tests Before | Tests After | Inv Before | Inv After | Last Touched |
|------|-------------|-------------|------------|-----------|--------------|
| arch | 105 | **108** | 7 | **8** | W246 (**12 waves untouched**) |

- **arch**: +3 tests (`arch_empty_kv_cache_key_cache_empty`, `arch_sqrt_approx_zero_is_zero`, `arch_sin_approx_zero_is_zero`) +1 invariant (`relu_zero_is_zero`).

**CODER new minimum: benchmark 7, pipeline 7, prm 7, tokenizer 7, training 7** (5 specs at 7; arch raised to 8).

---

## 3. Invariant Count Summary

| Category | Pre-W259 Minimum | Post-W259 Minimum |
|----------|-----------------|-------------------|
| Pool A | bram_weights 13, cordic_fixed 13, cordic_top 13, formal 13 | **cordic_top 13, formal 13** (2 specs at 13; bram_weights and cordic_fixed raised to 14) |
| Pool B | systolic_ternary 13 | **ALL ≥14** (systolic_ternary raised 13→14; first time uniform ≥14) |
| CODER | arch 7, benchmark 7, pipeline 7, prm 7, tokenizer 7, training 7 | **benchmark 7, pipeline 7, prm 7, tokenizer 7, training 7** (arch raised 7→8) |

- +11 tests added across 5 specs
- +5 invariants added
- 5 seals regenerated
- 570/570 PASS

---

## 4. Weaknesses Addressed

1. **Pool A floor narrowed**: 4 specs at 13 → 2 specs at 13 (bram_weights and cordic_fixed raised to 14).
2. **Pool B uniform ≥14**: systolic_ternary raised 13→14 after 6-wave dormancy — final Pool B spec below 14 eliminated.
3. **CODER depth**: arch raised 7→8 after 12-wave dormancy. 5 CODER specs remain at 7.
4. **Competitive moat**: 26-wave zero-entrant streak maintained. Trinity's invariant depth continues to outpace all 231 tracked competitors.

---

## 5. Seal Regeneration Log

| Spec | Seal File |
|------|-----------|
| bram_weights | `.trinity/seals/race_igla-race-bram-weights.json` |
| cordic_fixed | `.trinity/seals/race_igla-race-cordic-fixed.json` |
| systolic_ternary | `.trinity/seals/race_igla-race-systolic-ternary.json` |
| ternary_gemm | `.trinity/seals/race_igla-race-ternary-gemm.json` |
| arch | `.trinity/seals/coder_igla-coder-arch.json` |

---

## 6. Execution Plan Completed

```
Phase 1: OBSERVE    → Issue #259 identified, variant A selected
Phase 2: PLAN       → 5 specs targeted: 2 Pool A + 2 Pool B + 1 CODER
Phase 3: DELEGATE   → Implementation on bram_weights, cordic_fixed, systolic_ternary, ternary_gemm, arch
Phase 4: VERIFY     → 570/570 PASS, 5 seals regenerated
Phase 5: SYNTHESIZE → Structural milestone: Pool B uniform ≥14, Pool A narrowed to 2 at 13
Phase 6: LEARN      → Pattern saved to memory
```

**Phase complete: VERIFY**
→ Phase 5: SYNTHESIZE → Phase 6: LEARN
