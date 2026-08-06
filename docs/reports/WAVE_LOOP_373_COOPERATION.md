# Wave Loop 374 — Three Cooperation Variants

**Prepared:** 2026-07-01 from Wave Loop 373 close-out  
**Tracking:** Follow-up to [#1262](https://github.com/gHashTag/t27/issues/1262)

The variants below follow the standard triad: **Variant A** (safe/minimal), **Variant B** (balanced — recommended), and **Variant C** (aggressive). All assume the QMTech Wukong V1 / XC7A100T remains the FPGA target and that `trinity-rust-rings` stays the working branch unless a broader `master` merge is explicitly authorized.

---

## Variant A — Conservative / Tooling Hardening

**Theme:** Keep proof growth minimal and invest the wave in backend/tooling hardening.

### Deliverables
- Update `GEN_VERILOG_DEFECTS_REPRO.md` with W373 struct-field keyword escape results.
- Add a second scratch spec that exercises keyword collision in **module-level** identifiers (e.g., a top-level `wire`/`reg` name or a module named `task`).
- Audit remaining `gen_verilog_*` sites for any other identifier that is escaped in isolation and then concatenated, producing invalid mid-identifier backslashes.
- Add one small theorem: `ternaryMacZeroWeightSexdecupleClosureAlt` (alternate 7+1+9 shape) to show closure generalizes.
- Refresh `docs/reports/FPGA_EVIDENCE_*.md` index and retry `dlc10 idcode`.

### Metrics
- Generic ∀: 236 → **237** (+1)
- Pool A floor: 115 (unchanged)
- CODER minimum: 105 (unchanged)
- Pool B depth: 133 (unchanged)
- Integration depth: 114 (unchanged)
- Tests: +2 to +6
- Invariants: +1 to +3
- Conformance target: 553/553 PASS

### Risk
- **Low.** Minimal proof-lattice stress; tooling-only changes can be reverted easily.

### Cooperation ask
- Review and merge documentation/tooling PR.
- Confirm whether board/cable will be available during W374.

---

## Variant B — Balanced / Recommended

**Theme:** Continue the established cadence: 50-variable plus accumulation, 49-variable minus lattice, depth-27 cancellation, zero-weight 17-closure, and one safe gen-verilog sub-fix.

### Deliverables
1. **IGLA CODER+RACE spec blocks** — W374 blocks across all 27 specs (+54 tests, +27 invariants).
2. **Lean 4 proof-lattice extension**:
   - `ternaryMacAccumulateFiftyPlusGeneric` (`a+b+...+as+au+av+aw+ax+ay`) — watch elaboration budget at depth 50.
   - `ternaryMacAccumulateFortyNineMinusGeneric` (`-(a+b+...+ax)`).
   - `ternaryMacSeptemvigintupleCancellationGeneric` (`mac^27(x,a,[.plus,.minus,...]) = mac(x,a,.plus)`) — odd depth residual cancellation.
   - `ternaryMacZeroWeightSeptendecupleClosureGeneric` (8 zero + 1 plus + 9 zero or 9+1+8 shape — choose whichever builds faster).
3. **Safe gen-verilog sub-fix** — pick the narrower of:
   - Extend keyword escaping to module-level identifier emissions (e.g., top-level `wire`/`reg` names), or
   - Add a scratch repro for keyword collision inside array indexing / enum values and fix it.
4. **FPGA retry** — attempt `dlc10 idcode` / `dlc10 sram` if hardware becomes available.
5. **Memory + reports** — write W374 report and three W375 cooperation variants.

### Metrics
- Generic ∀: 236 → **240** (+4)
- Pool A floor: 115 → **116**
- CODER minimum: 105 → **106**
- Pool B depth: 133 → **134**
- Integration depth: 114 → **115**
- Tests: +54
- Invariants: +27
- Conformance target: **557/557 PASS** (553 current + 4 new artifacts)

### Risk
- **Moderate.** Depth 50 may approach the practical Lean elaboration boundary; if timeout exceeds ~15 s, drop the plus accumulation and keep the other three theorems.
- A module-level keyword-escape fix may cause another mass seal regeneration; script it.

### Cooperation ask
- Approve the planned gen-verilog sub-fix scope.
- Confirm whether board/cable will be available during W374.

---

## Variant C — Aggressive / Lateral Expansion

**Theme:** Hit **244 generic ∀**, land a second backend sub-fix, and open a new proof-lattice dimension: mixed-weight non-trivial activation theorems.

### Deliverables
1. **Skip 50-variable plus and go to 51** if the W374 boundary proves soft:
   - `ternaryMacAccumulateFiftyOnePlusGeneric`
   - `ternaryMacAccumulateFiftyMinusGeneric`
   - `ternaryMacDuodecimvigintupleCancellationGeneric` (depth-28 identity)
   - `ternaryMacZeroWeightOctodecupleClosureGeneric` (18-closure)
   - Introduce **one mixed-weight theorem** as a new lattice dimension: `ternaryMacMixedWeightDistributiveGeneric` (e.g., plus then zero then minus reordering).
2. **Two gen-verilog sub-fixes** — module-level keyword escape + array-index/enum keyword escape, if both are safe and independent.
3. **Bitstream regeneration** — if hardware arrives, regenerate `ternary_mac_demo_top.bit` with any new Verilog fixes and attempt flash.
4. **Tooling hardening** — add a CI smoke gate using the Rust runner (`t27c suite`) and update `OWNERS.md` for `bootstrap/src/compiler.rs`.

### Metrics
- Generic ∀: 236 → **244** (+8)
- Pool A floor: 115 → **117**
- CODER minimum: 105 → **107**
- Pool B depth: 133 → **135**
- Integration depth: 114 → **116**
- Tests: +54 to +108
- Invariants: +27 to +54
- Conformance target: **561/561 PASS**

### Risk
- **High.** 51-variable theorem may exceed Lean elaboration budget; mixed-weight dimension may require auxiliary lemmas; two backend fixes may interact and break seals across unrelated specs.

### Cooperation ask
- Authorize extended CPU/CI time for larger Lean builds.
- Explicitly approve touching `master`-only #1245 fixes for merge or selective cherry-pick.
- Confirm board/cable availability.

---

## Recommendation

**Choose Variant B.** It preserves the 4-theorem-per-wave cadence that has produced 33 consecutive zero-IGLA-failure waves, includes one safe backend fix to keep chipping at gen-verilog gaps, and leaves room to downgrade to Variant A if the 50-variable theorem times out.

---

*phi² + 1/phi² = 3 | TRINITY*
