# Wave Loop 371 — Three Cooperation Variants

**Prepared:** 2026-07-02 from Wave Loop 370 close-out  
**Tracking:** Follow-up to [#1259](https://github.com/gHashTag/t27/issues/1259)

The variants below follow the standard triad: **Variant A** (safe/minimal), **Variant B** (balanced — recommended), and **Variant C** (aggressive). All assume the QMTech Wukong V1 / XC7A100T remains the FPGA target and that `trinity-rust-rings` stays the working branch unless a broader master merge is explicitly authorized.

---

## Variant A — Conservative / Documentation-Heavy

**Theme:** Close out open documentation and low-risk gen-verilog fixes while keeping proof growth minimal.

### Deliverables
- Update `GEN_VERILOG_DEFECTS_REPRO.md` with the B1 verification results and remaining 4 defect reproductions.
- Fix **defect 3** if it is a one-line parser change (safe, no new shell scripts).
- Write a formal tutorial (`docs/lean/TERNARY_INFERENCE_TUTORIAL.md`) explaining how the generic ∀ proofs are constructed and how to avoid Lean keyword collisions.
- Add one small new theorem: `ternaryMacZeroWeightTredecupleClosureAlt` (alternate 6+1+6 shape) to show the closure lattice generalizes.
- Update all stale `docs/reports/FPGA_EVIDENCE_*.md` to reference the latest `dlc10` output.

### Metrics
- Generic ∀: 224 → **225** (+1)
- Pool A floor: 111 (unchanged)
- CODER minimum: 101 (unchanged)
- Pool B depth: 129 (unchanged)
- Integration depth: 110 (unchanged)
- Tests: +2 to +6
- Invariants: +1 to +3
- Conformance target: 549/549 PASS

### Risk
- **Low.** No parser churn; almost no seal risk. Documentation is the largest effort.

### Cooperation ask
- Review and merge documentation PR.
- Provide FPGA board/cable access if available.

---

## Variant B — Balanced / Recommended

**Theme:** Continue the established cadence: 47-variable plus accumulation, 46-variable minus lattice, depth-24 cancellation, zero-weight 14-closure, and one safe gen-verilog sub-fix.

### Deliverables
1. **IGLA CODER+RACE spec blocks** — W371 blocks across all 27 specs (+54 tests, +27 invariants).
2. **Lean 4 proof-lattice extension**:
   - `ternaryMacAccumulateFortySevenPlusGeneric` (`a+b+...+as+au+av`) — watch for the 47th variable (`av`) and keyword collisions.
   - `ternaryMacAccumulateFortySixMinusGeneric` (`-(a+b+...+at)` with `at` skipped).
   - `ternaryMacQuattuorvigintupleCancellationGeneric` (`mac^24(x,a,[.plus,.minus,...]) = mac(x,a,.plus)`).
   - `ternaryMacZeroWeightQuattuordecupleClosureGeneric` (7 zero + 1 plus + 7 zero or alternate 6+1+7 shape — choose whichever builds faster).
3. **Safe gen-verilog sub-fix** — select the next lowest-risk defect from `GEN_VERILOG_DEFECTS_REPRO.md` (likely defect 2 or 3) and fix it in `bootstrap/src/compiler.rs` or the lowering pass with no new shell scripts.
4. **FPGA retry** — attempt `dlc10 idcode`, `dlc10 sram`, or `dlc10 reload` if hardware becomes available.
5. **Memory + reports** — write W371 report and three W372 cooperation variants.

### Metrics
- Generic ∀: 224 → **228** (+4)
- Pool A floor: 111 → **112**
- CODER minimum: 101 → **102**
- Pool B depth: 129 → **130**
- Integration depth: 110 → **111**
- Tests: +54
- Invariants: +27
- Conformance target: **553/553 PASS** (549 current + 4 new Lean/build artifacts; actual spec count may grow slightly)

### Risk
- **Moderate.** 47 variables approach the practical Lean elaboration boundary; if timeout exceeds ~8 s, drop the plus accumulation and keep the other three theorems.
- Another gen-verilog fix may require mass resealing; script it and run full suite before claiming PASS.

### Cooperation ask
- Approve the planned gen-verilog sub-fix (which defect).
- Confirm whether board/cable will be available during W371.

---

## Variant C — Aggressive / Lateral Expansion

**Theme:** Hit **232 generic ∀**, land a second backend sub-fix, and start a new proof-lattice dimension: signed/negative-weight MAC theorems.

### Deliverables
1. **Skip 47-variable plus and go to 48** if Variant B proves the boundary is still soft:
   - `ternaryMacAccumulateFortyEightPlusGeneric`
   - `ternaryMacAccumulateFortySevenMinusGeneric`
   - `ternaryMacQuinvigintupleCancellationGeneric` (depth-25)
   - `ternaryMacZeroWeightQuindecupleClosureGeneric` (15-closure)
   - Introduce **one signed-weight theorem** as a new lattice dimension: `ternaryMacSignedWeightBaseGeneric`.
2. **Two gen-verilog sub-fixes** — defect 2 and defect 4, if both are safe and independent.
3. **Bitstream regeneration** — if hardware arrives, regenerate `ternary_mac_demo_top.bit` with any new Verilog fixes and attempt flash.
4. **Tooling hardening** — add a non-shell CI smoke gate using the Rust runner (`t27c suite --repo-root .`) and update `OWNERS.md` for `bootstrap/src/compiler.rs`.

### Metrics
- Generic ∀: 224 → **232** (+8)
- Pool A floor: 111 → **113**
- CODER minimum: 101 → **103**
- Pool B depth: 129 → **131**
- Integration depth: 110 → **112**
- Tests: +54 to +108
- Invariants: +27 to +54
- Conformance target: **557/557 PASS**

### Risk
- **High.** 48-variable generic theorem may exceed Lean elaboration budget; signed-weight dimension may require auxiliary lemmas that do not close in one wave; two backend fixes may interact and break seals across unrelated specs.

### Cooperation ask
- Authorize extended CPU/CI time for larger Lean builds.
- Explicitly approve touching `master`-only #1245 fixes for merge or selective cherry-pick.
- Confirm board/cable availability.

---

## Recommendation

**Choose Variant B.** It preserves the 4-theorem-per-wave cadence that has produced 30 consecutive zero-IGLA-failure waves, includes one safe backend fix to keep chipping at #1245, and leaves room to downgrade to Variant A if the 47-variable theorem times out.

---

*phi² + 1/phi² = 3 | TRINITY*
