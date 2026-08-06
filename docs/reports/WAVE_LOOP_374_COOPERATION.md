# Wave Loop 375 — Three Cooperation Variants

**Prepared:** 2026-07-01 from Wave Loop 374 close-out  
**Tracking:** Follow-up to [#1263](https://github.com/gHashTag/t27/issues/1263)

The variants below follow the standard triad: **Variant A** (safe/minimal), **Variant B** (balanced — recommended), and **Variant C** (aggressive). All assume the QMTech Wukong V1 / XC7A100T remains the FPGA target and that `trinity-rust-rings` stays the working branch unless a broader `master` merge is explicitly authorized.

---

## Variant A — Conservative / Tooling Hardening

**Theme:** Keep proof growth minimal and invest the wave in backend/tooling hardening.

### Deliverables
- Update `GEN_VERILOG_DEFECTS_REPRO.md` with W374 module-level keyword escape results.
- Add a second scratch spec that exercises keyword collision in **enum variant** names (e.g., an enum variant named `task`).
- Audit remaining `gen_verilog_*` sites for any other identifier that is escaped in isolation and then concatenated, producing invalid mid-identifier backslashes.
- Add one small theorem: `ternaryMacZeroWeightSeptendecupleClosureAlt` (alternate 7+1+10 shape) to show closure generalizes.
- Refresh `docs/reports/FPGA_EVIDENCE_*.md` index and retry `dlc10 idcode`.

### Metrics
- Generic ∀: 240 → **241** (+1)
- Pool A floor: 116 (unchanged)
- CODER minimum: 106 (unchanged)
- Pool B depth: 134 (unchanged)
- Integration depth: 115 (unchanged)
- Tests: +2 to +6
- Invariants: +1 to +3
- Conformance target: 554/554 PASS

### Risk
- **Low.** Minimal proof-lattice stress; tooling-only changes can be reverted easily.

### Cooperation ask
- Review and merge documentation/tooling PR.
- Confirm whether board/cable will be available during W375.

---

## Variant B — Balanced / Recommended

**Theme:** Continue the established cadence: 51-variable plus accumulation, 50-variable minus lattice, depth-28 cancellation, zero-weight 18-closure, and one safe gen-verilog sub-fix.

### Deliverables
1. **IGLA CODER+RACE spec blocks** — W375 blocks across all 27 specs (+54 tests, +27 invariants).
2. **Lean 4 proof-lattice extension**:
   - `ternaryMacAccumulateFiftyOnePlusGeneric` (`a+b+...+as+au+av+aw+ax+ay+az`) — watch elaboration budget at depth 51.
   - `ternaryMacAccumulateFiftyMinusGeneric` (`-(a+b+...+ay)`).
   - `ternaryMacDuodecimvigintupleCancellationGeneric` (`mac^28(x,a,[.plus,.minus,...]) = x`) — even depth identity cancellation.
   - `ternaryMacZeroWeightOctodecupleClosureGeneric` (9 zero + 1 plus + 9 zero or 8+1+10 shape — choose whichever builds faster).
3. **Safe gen-verilog sub-fix** — pick the narrower of:
   - Extend keyword escaping to enum variant declarations (e.g., `enum E { task }`), or
   - Add a scratch repro for `let` destructuring and begin a narrow lowering path (emit scalar reg declarations for each bound name).
4. **FPGA retry** — attempt `dlc10 idcode` / `dlc10 sram` if hardware becomes available.
5. **Memory + reports** — write W375 report and three W376 cooperation variants.

### Metrics
- Generic ∀: 240 → **244** (+4)
- Pool A floor: 116 → **117**
- CODER minimum: 106 → **107**
- Pool B depth: 134 → **135**
- Integration depth: 115 → **116**
- Tests: +54
- Invariants: +27
- Conformance target: **558/558 PASS** (554 current + 4 new artifacts)

### Risk
- **Moderate.** Depth 51 may approach the practical Lean elaboration boundary; if timeout exceeds ~15 s, drop the plus accumulation and keep the other three theorems.
- A `let` destructuring fix may require AST-level tuple-pattern support; prefer the enum variant escape if it is narrower.

### Cooperation ask
- Approve the planned gen-verilog sub-fix scope.
- Confirm whether board/cable will be available during W375.

---

## Variant C — Aggressive / Lateral Expansion

**Theme:** Hit **248 generic ∀**, land a second backend sub-fix, and open a new proof-lattice dimension: mixed-weight non-trivial activation theorems.

### Deliverables
1. **Skip 51-variable plus and go to 52** if the W375 boundary proves soft:
   - `ternaryMacAccumulateFiftyTwoPlusGeneric`
   - `ternaryMacAccumulateFiftyOneMinusGeneric`
   - `ternaryMacNovemvigintupleCancellationGeneric` (depth-29 residual)
   - `ternaryMacZeroWeightNovemdecupleClosureGeneric` (19-closure)
   - Introduce **one mixed-weight theorem** as a new lattice dimension: `ternaryMacMixedWeightDistributiveGeneric` (e.g., plus then zero then minus reordering).
2. **Two gen-verilog sub-fixes** — enum variant keyword escape + a narrow `let` destructuring lowering, if both are safe and independent.
3. **Bitstream regeneration** — if hardware arrives, regenerate `ternary_mac_demo_top.bit` with any new Verilog fixes and attempt flash.
4. **Tooling hardening** — add a CI smoke gate using the Rust runner (`t27c suite`) and update `OWNERS.md` for `bootstrap/src/compiler.rs`.

### Metrics
- Generic ∀: 240 → **248** (+8)
- Pool A floor: 116 → **118**
- CODER minimum: 106 → **108**
- Pool B depth: 134 → **136**
- Integration depth: 115 → **117**
- Tests: +54 to +108
- Invariants: +27 to +54
- Conformance target: **562/562 PASS**

### Risk
- **High.** 52-variable theorem may exceed Lean elaboration budget; mixed-weight dimension may require auxiliary lemmas; two backend fixes may interact and break seals across unrelated specs.

### Cooperation ask
- Authorize extended CPU/CI time for larger Lean builds.
- Explicitly approve touching `master`-only #1245 fixes for merge or selective cherry-pick.
- Confirm board/cable availability.

---

## Recommendation

**Choose Variant B.** It preserves the 4-theorem-per-wave cadence that has produced 34 consecutive zero-IGLA-failure waves, includes one safe backend fix to keep chipping at gen-verilog gaps, and leaves room to downgrade to Variant A if the 51-variable theorem times out.

---

*phi² + 1/phi² = 3 | TRINITY*
