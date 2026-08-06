# Wave Loop 372 — Three Cooperation Variants

**Prepared:** 2026-07-02 from Wave Loop 371 close-out  
**Tracking:** Follow-up to [#1260](https://github.com/gHashTag/t27/issues/1260)

The variants below follow the standard triad: **Variant A** (safe/minimal), **Variant B** (balanced — recommended), and **Variant C** (aggressive). All assume the QMTech Wukong V1 / XC7A100T remains the FPGA target and that `trinity-rust-rings` stays the working branch unless a broader master merge is explicitly authorized.

---

## Variant A — Conservative / Documentation-Heavy

**Theme:** Close out open documentation and low-risk gen-verilog fixes while keeping proof growth minimal.

### Deliverables
- Update `GEN_VERILOG_DEFECTS_REPRO.md` with the W371 keyword-escape verification results and add a new defect entry for the `let` destructuring lowering gap observed in `cordic.t27` / `cordic_top.t27`.
- Fix **one additional keyword edge case** if found during a broader yosys sweep (e.g., module top names that collide with keywords).
- Write a formal tutorial (`docs/lean/TERNARY_INFERENCE_TUTORIAL.md`) explaining how the generic ∀ proofs are constructed and how to avoid Lean keyword collisions at depth 48+.
- Add one small new theorem: `ternaryMacZeroWeightQuattuordecupleClosureAlt` (alternate 6+1+7 shape) to show the closure lattice generalizes.
- Update all stale `docs/reports/FPGA_EVIDENCE_*.md` to reference the latest `dlc10` output.

### Metrics
- Generic ∀: 228 → **229** (+1)
- Pool A floor: 112 (unchanged)
- CODER minimum: 102 (unchanged)
- Pool B depth: 130 (unchanged)
- Integration depth: 111 (unchanged)
- Tests: +2 to +6
- Invariants: +1 to +3
- Conformance target: 551/551 PASS

### Risk
- **Low.** No parser churn; almost no seal risk. Documentation is the largest effort.

### Cooperation ask
- Review and merge documentation PR.
- Provide FPGA board/cable access if available.

---

## Variant B — Balanced / Recommended

**Theme:** Continue the established cadence: 48-variable plus accumulation, 47-variable minus lattice, depth-25 cancellation, zero-weight 15-closure, and one safe gen-verilog sub-fix.

### Deliverables
1. **IGLA CODER+RACE spec blocks** — W372 blocks across all 27 specs (+54 tests, +27 invariants).
2. **Lean 4 proof-lattice extension**:
   - `ternaryMacAccumulateFortyEightPlusGeneric` (`a+b+...+as+au+av+aw`) — watch for keyword collisions at 48 variables (`aw` is safe; `at` already skipped).
   - `ternaryMacAccumulateFortySevenMinusGeneric` (`-(a+b+...+av)`).
   - `ternaryMacQuinvigintupleCancellationGeneric` (`mac^25(x,a,[.plus,.minus,...]) = mac(x,a,.plus)`).
   - `ternaryMacZeroWeightQuindecupleClosureGeneric` (7 zero + 1 plus + 8 zero or alternate 8+1+7 shape — choose whichever builds faster).
3. **Safe gen-verilog sub-fix** — fix the `let` destructuring lowering gap observed in `cordic.t27` / `cordic_top.t27`, or extend keyword escaping to local variables / struct fields if the `let` fix is too broad. Pick the narrower of the two.
4. **FPGA retry** — attempt `dlc10 idcode`, `dlc10 sram`, or `dlc10 reload` if hardware becomes available.
5. **Memory + reports** — write W372 report and three W373 cooperation variants.

### Metrics
- Generic ∀: 228 → **232** (+4)
- Pool A floor: 112 → **113**
- CODER minimum: 102 → **103**
- Pool B depth: 130 → **131**
- Integration depth: 111 → **112**
- Tests: +54
- Invariants: +27
- Conformance target: **555/555 PASS** (551 current + 4 new artifacts; actual spec count may grow slightly)

### Risk
- **Moderate.** 48 variables approach the practical Lean elaboration boundary; if timeout exceeds ~10 s, drop the plus accumulation and keep the other three theorems.
- A `let` destructuring fix may require mass resealing; script it and run full suite before claiming PASS.

### Cooperation ask
- Approve the planned gen-verilog sub-fix (`let` destructuring vs. keyword-escape extension).
- Confirm whether board/cable will be available during W372.

---

## Variant C — Aggressive / Lateral Expansion

**Theme:** Hit **236 generic ∀**, land a second backend sub-fix, and start a new proof-lattice dimension: signed/negative-weight MAC theorems.

### Deliverables
1. **Skip 48-variable plus and go to 49** if Variant B proves the boundary is still soft:
   - `ternaryMacAccumulateFortyNinePlusGeneric`
   - `ternaryMacAccumulateFortyEightMinusGeneric`
   - `ternaryMacSesvigintupleCancellationGeneric` (depth-26)
   - `ternaryMacZeroWeightSexdecupleClosureGeneric` (16-closure)
   - Introduce **one signed-weight theorem** as a new lattice dimension: `ternaryMacSignedWeightBaseGeneric`.
2. **Two gen-verilog sub-fixes** — `let` destructuring + keyword escaping for local/struct identifiers, if both are safe and independent.
3. **Bitstream regeneration** — if hardware arrives, regenerate `ternary_mac_demo_top.bit` with any new Verilog fixes and attempt flash.
4. **Tooling hardening** — add a non-shell CI smoke gate using the Rust runner (`t27c suite --repo-root .`) and update `OWNERS.md` for `bootstrap/src/compiler.rs`.

### Metrics
- Generic ∀: 228 → **236** (+8)
- Pool A floor: 112 → **114**
- CODER minimum: 102 → **104**
- Pool B depth: 130 → **132**
- Integration depth: 111 → **113**
- Tests: +54 to +108
- Invariants: +27 to +54
- Conformance target: **559/559 PASS**

### Risk
- **High.** 49-variable generic theorem may exceed Lean elaboration budget; signed-weight dimension may require auxiliary lemmas that do not close in one wave; two backend fixes may interact and break seals across unrelated specs.

### Cooperation ask
- Authorize extended CPU/CI time for larger Lean builds.
- Explicitly approve touching `master`-only #1245 fixes for merge or selective cherry-pick.
- Confirm board/cable availability.

---

## Recommendation

**Choose Variant B.** It preserves the 4-theorem-per-wave cadence that has produced 31 consecutive zero-IGLA-failure waves, includes one safe backend fix to keep chipping at gen-verilog gaps, and leaves room to downgrade to Variant A if the 48-variable theorem times out.

---

*phi² + 1/phi² = 3 | TRINITY*
