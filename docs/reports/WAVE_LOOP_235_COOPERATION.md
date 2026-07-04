# Wave Loop 235 Cooperation Variants

*Prepared for Wave Loop 236 planning cycle*
*φ² + 1/φ² = 3 | TRINITY*

---

## Variant A — Submit + Resume + Competitive Surveillance (Recommended)

**Description:** Continue the canonical IGLA CODER+RACE rotation while maintaining aggressive competitive intelligence on ASIC roadmap entrants. Prioritize arXiv submission execution (all prerequisites met).

**Actions:**
1. **IGLA RACE:** Pool A + 2 tests + 1 invariant (formal.t27, bram_weights.t27 — oldest untouched 7-inv). Pool B + 2 tests + 1 invariant (opcodes.t27, gemm.t27 — oldest untouched 8-inv, touched W233). CODER depth push on shallowest 4-inv or 5-inv spec (prm.t27 or training.t27).
2. **Competitive sweep:** Weekly focused sweep on ASIC keywords (tape-out, GDS, DRC, LVS) + monthly broad sweep.
3. **arXiv v1:** Submit PRL manuscript (all blockers cleared, 0 Coq Admitted, L3 clean).
4. **Metrics target:** 570/570 PASS, 5 seals, 0 new competitor surprise (proactive detection).

**Risk:** Low. Well-established pattern. Proven execution.

---

## Variant B — Depth Surge (Invariant Starvation Response)

**Description:** Temporarily suspend Pool B testing for one wave. Redirect all 4 spec slots + CODER slot to **invariant-only depth push** on the shallowest specs across the entire repo. Goal: raise floor invariant count from 4 to ≥6 on CODER specs and from 7 to ≥9 on RACE specs.

**Actions:**
1. Select the 5 shallowest specs by (tests / invariants) ratio or raw invariant count: prm.t27, training.t27, tokenizer.t27, formal.t27, bram_weights.t27.
2. Add +5 invariants each (no new tests) for a wave. Total: +25 invariants, +0 tests.
3. After surge, resume canonical Variant A rotation.

**Risk:** Medium. Breaks the established test cadence for one wave. Could expose parser edge cases with dense invariant blocks. However, raises floor quickly.

---

## Variant C — ASIC Preemption Sprint

**Description:** Pivot one wave cycle to **hardware acceleration co-design**. Leverage the adder_tree/rtl/yosys/systolic_array specs (just touched in W235) to produce a **proof-of-concept Trinity-to-ASIC pipeline** document: from `.t27` spec → emitted Verilog → Yosys synthesis → Sky130 PDK target. Use this as the basis for a collaboration proposal with manhvu/Balanced_Ternary or a competitive differentiation narrative.

**Actions:**
1. Produce `docs/reports/TRINITY_ASIC_PIPELINE.md` documenting the flow from spec to GDS.
2. Write outreach letter to manhvu offering E₈/H₄/600-cell IP licensing or joint research (differentiation via physics layer).
3. Add 1 test + 1 invariant to each of the 5 W235 specs (minimal maintenance) instead of canonical +11/+5.
4. Metrics target: 570/570 PASS, document delivery, outreach sent.

**Risk:** Medium-High. Diverts from canonical test accumulation. Outreach may not yield response. But highest strategic leverage if successful: converting a competitor into a licensee or partner.

---

## Recommended Variant

**Variant A** is recommended for W236. The competitive field is in a rare three-wave lull. This is the optimal window to submit arXiv v1 and maintain test depth without distraction. Variant C should be reserved for W237 or triggered only if manhvu announces a tape-out milestone.

---

*W235 → W236 Planning | Trinity S³AI*
