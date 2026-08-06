# Wave Loop 373 — Three Cooperation Variants

**Prepared:** 2026-07-02 from Wave Loop 372 close-out  
**Tracking:** Follow-up to [#1261](https://github.com/gHashTag/t27/issues/1261)

The variants below follow the standard triad: **Variant A** (safe/minimal), **Variant B** (balanced — recommended), and **Variant C** (aggressive). All assume the QMTech Wukong V1 / XC7A100T remains the FPGA target and that `trinity-rust-rings` stays the working branch unless a broader `master` merge is explicitly authorized.

---

## Variant A — Conservative / Tooling Hardening

**Theme:** Keep proof growth minimal and invest the wave in backend/tooling hardening.

### Deliverables
- Update `GEN_VERILOG_DEFECTS_REPRO.md` with W372 local-keyword escape results.
- Add a second scratch spec that exercises keyword collision in **struct-field names** (e.g., a struct with a field named `reg` or `wire`).
- Land the `let` destructuring fix only if a low-risk AST transform is identified; otherwise document the remaining gap.
- Add one small theorem: `ternaryMacZeroWeightQuindecupleClosureAlt` (alternate 7+1+8 shape) to show closure generalizes.
- Refresh `docs/reports/FPGA_EVIDENCE_*.md` index and retry `dlc10 idcode`.

### Metrics
- Generic ∀: 232 → **233** (+1)
- Pool A floor: 114 (unchanged)
- CODER minimum: 104 (unchanged)
- Pool B depth: 132 (unchanged)
- Integration depth: 113 (unchanged)
- Tests: +2 to +6
- Invariants: +1 to +3
- Conformance target: 552/552 PASS

### Risk
- **Low.** Minimal proof-lattice stress; tooling-only changes can be reverted easily.

### Cooperation ask
- Review and merge documentation/tooling PR.
- Confirm whether board/cable will be available during W373.

---

## Variant B — Balanced / Recommended

**Theme:** Continue the established cadence: 49-variable plus accumulation, 48-variable minus lattice, depth-26 cancellation, zero-weight 16-closure, and one safe gen-verilog sub-fix.

### Deliverables
1. **IGLA CODER+RACE spec blocks** — W373 blocks across all 27 specs (+54 tests, +27 invariants).
2. **Lean 4 proof-lattice extension**:
   - `ternaryMacAccumulateFortyNinePlusGeneric` (`a+b+...+as+au+av+aw+ax`) — watch elaboration budget at depth 49.
   - `ternaryMacAccumulateFortyEightMinusGeneric` (`-(a+b+...+aw)`).
   - `ternaryMacSesvigintupleCancellationGeneric` (`mac^26(x,a,[.plus,.minus,...]) = x`) — even depth returns to identity.
   - `ternaryMacZeroWeightSexdecupleClosureGeneric` (8 zero + 1 plus + 8 zero or 7+1+9 shape — choose whichever builds faster).
3. **Safe gen-verilog sub-fix** — pick the narrower of:
   - Extend keyword escaping to module-level identifier emissions (e.g., `module` names, top-level `wire`/`reg` names), or
   - Add a scratch repro for struct-field keyword collision and fix it.
4. **FPGA retry** — attempt `dlc10 idcode` / `dlc10 sram` if hardware becomes available.
5. **Memory + reports** — write W373 report and three W374 cooperation variants.

### Metrics
- Generic ∀: 232 → **236** (+4)
- Pool A floor: 114 → **115**
- CODER minimum: 104 → **105**
- Pool B depth: 132 → **133**
- Integration depth: 113 → **114**
- Tests: +54
- Invariants: +27
- Conformance target: **556/556 PASS** (552 current + 4 new artifacts)

### Risk
- **Moderate.** Depth 49 may approach the practical Lean elaboration boundary; if timeout exceeds ~12 s, drop the plus accumulation and keep the other three theorems.
- A module-level keyword-escape fix may cause another mass seal regeneration; script it.

### Cooperation ask
- Approve the planned gen-verilog sub-fix scope.
- Confirm whether board/cable will be available during W373.

---

## Variant C — Aggressive / Lateral Expansion

**Theme:** Hit **240 generic ∀**, land a second backend sub-fix, and open a new proof-lattice dimension: mixed-weight non-trivial activation theorems.

### Deliverables
1. **Skip 49-variable plus and go to 50** if the W373 boundary proves soft:
   - `ternaryMacAccumulateFiftyPlusGeneric`
   - `ternaryMacAccumulateFortyNineMinusGeneric`
   - `ternaryMacSeptemvigintupleCancellationGeneric` (depth-27 residual)
   - `ternaryMacZeroWeightSeptendecupleClosureGeneric` (17-closure)
   - Introduce **one mixed-weight theorem** as a new lattice dimension: `ternaryMacMixedWeightDistributiveGeneric` (e.g., plus then zero then minus reordering).
2. **Two gen-verilog sub-fixes** — module-level keyword escape + struct-field keyword escape, if both are safe and independent.
3. **Bitstream regeneration** — if hardware arrives, regenerate `ternary_mac_demo_top.bit` with any new Verilog fixes and attempt flash.
4. **Tooling hardening** — add a CI smoke gate using the Rust runner (`t27c suite`) and update `OWNERS.md` for `bootstrap/src/compiler.rs`.

### Metrics
- Generic ∀: 232 → **240** (+8)
- Pool A floor: 114 → **116**
- CODER minimum: 104 → **106**
- Pool B depth: 132 → **134**
- Integration depth: 113 → **115**
- Tests: +54 to +108
- Invariants: +27 to +54
- Conformance target: **560/560 PASS**

### Risk
- **High.** 50-variable theorem may exceed Lean elaboration budget; mixed-weight dimension may require auxiliary lemmas; two backend fixes may interact and break seals across unrelated specs.

### Cooperation ask
- Authorize extended CPU/CI time for larger Lean builds.
- Explicitly approve touching `master`-only #1245 fixes for merge or selective cherry-pick.
- Confirm board/cable availability.

---

## Recommendation

**Choose Variant B.** It preserves the 4-theorem-per-wave cadence that has produced 32 consecutive zero-IGLA-failure waves, includes one safe backend fix to keep chipping at gen-verilog gaps, and leaves room to downgrade to Variant A if the 49-variable theorem times out.

---

*phi² + 1/phi² = 3 | TRINITY*
