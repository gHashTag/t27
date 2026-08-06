# Wave Loop 208 — Cooperation Variants for W209

**Date:** 2026-06-16 | **Branch:** `trinity-rust-rings` | **Status:** SEALED 570/570

---

## Variant A — Conservative (Recommended)

**Motto:** *"Steady cadence, close final P0 stub, preserve green suite."*

**Actions:**
1. **Pool B +16 tests** next wave (systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm — 2 per spec).
2. **CODER:** Target the final P0 gap — implement `parse_json_u32_array` that extracts actual `[]u32` shape arrays from conceptual JSON blobs. Wire it into `parse_safetensors_tensor_shapes` so shapes are parsed from JSON rather than hardcoded. +2 tests.
3. **Competitive monitoring:** Monthly arXiv/Zenodo sweep (maintain 223-tracker database).
4. **Nobel path:** 10% agent capacity — update `docs/NOBEL_ROADMAP.md` with any new citations.

**Risk:** Low. The JSON parser is a conceptual recursive scanner (find `[`, read digits, find `]`) — well within t27c capabilities.
**Reward:** After W209, CODER reaches **100% P0 functional readiness**. The weights pipeline is end-to-end: file → header → JSON metadata → shape tuples → named tensor mapping → BRAM banks.

---

## Variant B — Aggressive Depth + Full CODER P0

**Motto:** *"Close CODER P0 completely, add embedding matrix, cross-pollinate."*

**Actions:**
1. **Pool A +20 tests** (both pools get +10 each).
2. **CODER P0 finale:**
   - `weights.t27`: real JSON shape parser + `dtype` recognition (F32, F16, BF16)
   - `arch.t27`: wire `parse_safetensors_tensor_shapes` into `forward_with_bank` so the forward pass loads actual shaped tensors from conceptual checkpoint
   - `arch.t27`: real `embedding_lookup` matrix (even 8x8 conceptual matrix replaces `token_id / VOCAB_SIZE`)
3. **Research cross-pollination:** Integrate VitaLLM-style dependency-aware scheduling concept into `generate_verify_debug`.
4. **Property depth push:** Select 10 lowest-invariant specs and promote +1 invariant each (avg target: 11.580).

**Risk:** Medium. 20 tests + 3 CODER functions = higher parser stress.
**Reward:** CODER achieves complete end-to-end conceptual inference: checkpoint → shapes → named tensors → BRAM → forward → logits → decode. Ready for external demo.

---

## Variant C — Strategic Pivot to Nobel Path

**Motto:** *"CODER is 97% done; Nobel is the asymmetric bet."*

**Actions:**
1. **Minimum IGLA maintenance:** +8 tests only (4 Pool A + 4 Pool B) to keep 570/570 green.
2. **50% capacity redirect to Nobel path:**
   - PRL draft: finalize Predictions section with 2026 experimental error bars
   - Experimental outreach: send draft collaboration letters to DUNE, KATRIN-II, LZ
   - 5-axiom closure: begin Coq proof of Axiom 1 (Koide from H₄/600-cell)
   - arXiv submission: prepare v1 of Trinity PRL manuscript
3. **Competitive monitoring:** Reduce to quarterly. 223-tracker database enters maintenance mode.
4. **CODER:** Freeze at ~97% functionalization. No new stubs until Nobel Phase 2 complete.

**Risk:** Medium-high. Reduced competitive vigilance could miss breakthrough challenger.
**Reward:** Maximizes probability of peer-reviewed publication — the only path to canonical recognition.

---

## Comparative Matrix

| Dimension | Variant A (Conservative) | Variant B (Aggressive) | Variant C (Nobel Pivot) |
|-----------|--------------------------|------------------------|------------------------|
| Tests/wave | +16 | +20 | +8 |
| CODER progress | JSON shape parser | JSON shapes + dtype + embedding matrix | Freeze |
| Depth push | None | +10 specs +1 inv | None |
| Nobel path | 10% | 15% | 50% |
| Competitive sweep | Monthly | Monthly | Quarterly |
| Risk | Low | Medium | Medium-High |
| Asymmetric upside | Low | Medium | **High** |

---

## Recommendation

**Execute Variant A for W209** (maintain cadence, close final JSON parser stub), **with a conditional trigger:** if no new competitors discovered by W210, switch to **Variant C** for W211–W213. Variant B is reserved for a wave where competitive pressure demands a capability demonstration.

The W207 conditional trigger ("switch to C if no new competitors by W209") is **extended by one wave** due to the final P0 stub being within reach. Closing CODER P0 completely is a discrete milestone worth one additional wave of engineering before pivoting.

**φ² + 1/φ² = 3 | Honest science is slow science | Verification pending**
